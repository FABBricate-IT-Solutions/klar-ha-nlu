from __future__ import annotations

import logging
from typing import Any

from homeassistant.components import conversation
from homeassistant.components.conversation import (
    ChatLog,
    ConversationEntity,
    ConversationInput,
    ConversationResult,
)
from homeassistant.components.homeassistant.exposed_entities import (
    async_should_expose,
)
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers import intent
from homeassistant.helpers.aiohttp_client import async_get_clientsession
from homeassistant.helpers import device_registry
from homeassistant.helpers.device_registry import DeviceInfo
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .const import (
    CONF_ALLOW_LLM_TOOLS,
    CONF_ASSIST_FILTER,
    CONF_CALENDAR_LLM,
    CONF_FALLBACK_AGENT,
    CONF_LANGUAGES,
    CONF_NLU_RAG,
    CONF_PERSONALITY,
    CONF_QUIET_ACK,
    CONF_REFINE_PROMPT,
    CONF_REFINE_SPEECH,
    CONF_TOKEN,
    CONF_URL,
    DEFAULT_ALLOW_LLM_TOOLS,
    DEFAULT_ASSIST_FILTER,
    DEFAULT_CALENDAR_LLM,
    DEFAULT_NLU_RAG,
    DEFAULT_QUIET_ACK,
    DEFAULT_URL,
    DOMAIN,
    engine_session_id,
    keeps_conversation,
    parse_session_id,
    resolve_personality,
)
from .engine_http import post_parse
from .lang_select import advertised_languages, default_pack, enabled_packs, resolve_pack, speak_tag
from .contracts import executable_intents
from .executor import execute_plan
from .fallback import (
    agent_has_home_control,
    append_llm_turn,
    can_use_fallback_agent,
    calendar_prompt,
    calendar_query_only,
    chat_only_prompt,
    history_prompt,
    llm_conversation_id,
    news_followup_prompt,
    news_prompt,
    with_personality,
    yarn_asks_permission,
    yarn_canned,
    yarn_nudge,
    yarn_prompt,
    yarn_request,
)
from .intents import home_intents, registered_intent_names
from .rag_tools import (
    act_payload,
    holds_klar_tool_prefix,
    leaks_klar_tools,
    parse_tool_reply,
    rag_prompt,
)
from .stream import stream_chat
from .news import announce, asked_for_more, compose_speech, fetch_headlines, nudge
from .policy_actions import (
    hit_and_payload,
    keeps_engine_chat,
    render_user_template,
    skips_llm_fallback,
)
from .refine import (
    async_finish_speech,
    emit_assistant_speech,
    isolated_conversation_id,
    llm_client_and_model,
    nested_llm_session,
    pop_complete_sentences,
    refine_prompt,
    skip_rewrite,
)
from .quiet import play_chime, quiet_ack_applies
from .sensor import remember_turn
from .speech import finish_clock_speech

_LOGGER = logging.getLogger(__name__)

_UNREACHABLE = {
    "de": "Klar antwortet gerade nicht.",
    "en": "Klar is not responding right now.",
}

_DONE = {"de": "Erledigt.", "en": "Done."}


def _cue(table: dict[str, str], pack: str, fallback: str) -> str:
    return table.get(pack) or fallback


def _ack_speech(engine_speech: str, pack: str, executed: bool) -> str:
    if engine_speech.strip():
        return engine_speech
    if executed:
        return _cue(_DONE, pack, "OK")
    return engine_speech


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    async_add_entities([KlarConversationEntity(hass, entry)])


def _speech_from_result(result: ConversationResult) -> str:
    speech = result.response.speech or {}
    plain = speech.get("plain") or {}
    return str(plain.get("speech") or "")


def _was_published(result: ConversationResult | None) -> bool:
    if result is None:
        return False
    speech = result.response.speech or {}
    plain = speech.get("plain") or {}
    extra = plain.get("extra_data")
    return bool(isinstance(extra, dict) and extra.get("klar_published"))


def _speech_result(pack: str, speech: str, published: bool) -> ConversationResult:
    response = intent.IntentResponse(language=speak_tag(pack))
    extra = {"klar_published": True} if published else None
    response.async_set_speech(speech, extra_data=extra)
    return ConversationResult(
        conversation_id=isolated_conversation_id(),
        response=response,
        continue_conversation=True,
    )


def _stream_hold(yarn: bool, rag: bool):
    if not yarn and not rag:
        return None

    def hold(speech: str) -> bool | None:
        stripped = speech.lstrip()
        if parse_tool_reply(stripped) or leaks_klar_tools(speech):
            return None
        if rag and holds_klar_tool_prefix(stripped):
            return False
        if yarn:
            if not pop_complete_sentences(speech)[0]:
                return False
            if yarn_asks_permission(speech):
                return None
        return True

    return hold


class KlarConversationEntity(ConversationEntity):
    _attr_name = "Klar NLU"
    _attr_supported_features = conversation.ConversationEntityFeature.CONTROL
    _attr_supports_streaming = True

    def __init__(self, hass: HomeAssistant, entry: ConfigEntry) -> None:
        self.hass = hass
        self._entry = entry
        self._attr_unique_id = entry.entry_id
        self._attr_device_info = DeviceInfo(
            identifiers={(DOMAIN, entry.entry_id)},
            name="Klar NLU",
            manufacturer="FABBricate IT Solutions",
        )

    def _packs(self) -> list[str]:
        return enabled_packs(
            self._entry.options.get(CONF_LANGUAGES),
            getattr(self.hass.config, "language", None),
        )

    def _request_pack(self, language: str | None) -> str:
        choice = self._entry.options.get(CONF_LANGUAGES)
        hass_language = getattr(self.hass.config, "language", None)
        if language:
            return resolve_pack(language, self._packs())
        return default_pack(choice, hass_language)

    @property
    def supported_languages(self) -> list[str]:
        return advertised_languages()

    @property
    def _url(self) -> str:
        return (
            self._entry.options.get(CONF_URL)
            or self._entry.data.get(CONF_URL)
            or DEFAULT_URL
        ).rstrip("/")

    def _fallback_agent_id(self) -> str | None:
        agent_id = self._entry.options.get(CONF_FALLBACK_AGENT)
        if not agent_id or agent_id == self.entity_id:
            return None
        return agent_id

    def _assistant(self) -> str | None:
        if self._entry.options.get(CONF_ASSIST_FILTER, DEFAULT_ASSIST_FILTER):
            return "conversation"
        return None

    def _personality(self) -> str:
        return resolve_personality(self._entry.options.get(CONF_PERSONALITY))

    def _token(self) -> str | None:
        stored = (self.hass.data.get(DOMAIN) or {}).get(self._entry.entry_id) or {}
        token = stored.get("token") or self._entry.options.get(CONF_TOKEN) or self._entry.data.get(CONF_TOKEN)
        return str(token) if token else None

    def _headers(self) -> dict[str, str]:
        token = self._token()
        return {"X-Klar-Token": token} if token else {}

    def _agent_controls_home(self, agent_id: str) -> bool:
        try:
            agent = conversation.async_get_agent(self.hass, agent_id)
        except Exception:  # noqa: BLE001 — other agent is a system boundary
            return True
        if agent is None:
            return True
        features = getattr(agent, "supported_features", 0)
        if callable(features):
            try:
                features = features()
            except Exception:  # noqa: BLE001 — agent API varies
                return True
        return agent_has_home_control(features)

    def _exposed(self, entity_id: str) -> bool:
        if not self._assistant():
            return True
        try:
            return bool(async_should_expose(self.hass, "conversation", entity_id))
        except Exception:  # noqa: BLE001 — expose store is a system boundary
            return False

    async def _async_handle_message(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
    ) -> ConversationResult:
        pack = self._request_pack(user_input.language)
        triggered = await self._sentence_triggers(user_input, chat_log, pack)
        if triggered is not None:
            return triggered
        payload = await self._parse(
            user_input.text, user_input.conversation_id, pack, user_input.device_id, getattr(user_input, "satellite_id", None)
        )
        engine_speech = str(payload.get("speech") or "")
        decision = payload.get("decision") if isinstance(payload.get("decision"), dict) else {}
        decision_type = str(decision.get("type") or "")
        intents = home_intents(executable_intents(payload), registered_intent_names(self.hass))
        speech = _ack_speech(engine_speech, pack, decision_type == "execute" and bool(intents))
        clarify = decision_type in {"clarify", "confirm"}
        chat = decision_type == "chat"
        conversation_id = payload.get("conversation_id") or user_input.conversation_id

        if payload.get("briefing"):
            if chat:
                briefing = await self._briefing(
                    user_input, chat_log, pack, engine_speech, conversation_id
                )
                if briefing is not None:
                    return briefing
            else:
                return await self._spoken(
                    user_input, chat_log, pack, _ack_speech(engine_speech, pack, False), conversation_id, False
                )

        retrieval = payload.get("retrieval") if isinstance(payload.get("retrieval"), dict) else None
        hit, action = hit_and_payload(payload)
        if hit == "template" and action:
            rendered = await render_user_template(self.hass, action, user_input.text)
            if rendered:
                speech = rendered
        if hit == "llm" and action and not clarify and not payload.get("unreachable"):
            prompt = yarn_prompt(pack, action, user_input.text) if yarn_request(user_input.text) else chat_only_prompt(pack, action, self._allow_llm_tools())
            fallback = await self._fallback(
                user_input, chat_log, pack, True, prompt, retrieval
            )
            replied = await self._after_fallback(
                user_input, chat_log, pack, fallback, speech, conversation_id
            )
            if replied is not None:
                return replied
        if (
            not skips_llm_fallback(hit)
            and not keeps_engine_chat(hit, chat, engine_speech)
            and not clarify
            and not intents
            and not payload.get("unreachable")
            and self._fallback_agent_id()
        ):
            fallback = await self._fallback(user_input, chat_log, pack, chat, retrieval=retrieval)
            replied = await self._after_fallback(
                user_input, chat_log, pack, fallback, speech, conversation_id
            )
            if replied is not None:
                return replied

        if decision_type == "execute" and intents:
            names = {item.get("name") for item in intents}
            plan = [
                item
                for item in intents
                if not (item.get("name") == "HassVacuumReturnToBase" and "HassGetState" in names)
            ]
            executed = await execute_plan(
                self.hass, user_input, plan, pack, self._assistant(), self._exposed
            )
            if executed.get("speech"):
                speech = str(executed["speech"])
            if executed.get("outcome") == "error":
                decision_type = "error"
            if (
                self._calendar_llm()
                and calendar_query_only(plan)
                and self._fallback_agent_id()
            ):
                extra = getattr(user_input, "extra_system_prompt", None)
                extra_s = extra if isinstance(extra, str) else None
                fallback = await self._fallback(
                    user_input, chat_log, pack, True, calendar_prompt(pack, speech, extra_s)
                )
                llm = _speech_from_result(fallback) if fallback is not None else ""
                if llm.strip() and "?" not in llm:
                    speech = llm
            if self._quiet_ack() and quiet_ack_applies(executed, plan):
                await play_chime(self.hass, user_input)
                return await self._spoken(
                    user_input, chat_log, pack, "", conversation_id, False, "chime"
                )
        return await self._spoken(
            user_input, chat_log, pack, speech, conversation_id, keeps_conversation(decision_type), decision_type
        )

    async def _after_fallback(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
        pack: str,
        fallback: ConversationResult | None,
        speech: str,
        conversation_id: str | None,
    ) -> ConversationResult | None:
        if fallback is None:
            return None
        tooled = await self._klar_tool_turn(user_input, chat_log, pack, fallback)
        if tooled is not None:
            return tooled
        llm = _speech_from_result(fallback)
        if yarn_request(user_input.text) and yarn_asks_permission(llm):
            llm = yarn_canned(pack, user_input.text)
        if leaks_klar_tools(llm):
            llm = speech
            return await self._spoken(user_input, chat_log, pack, llm, conversation_id, False)
        self._note_llm_turn(user_input, llm)
        return await self._spoken(
            user_input, chat_log, pack, llm, conversation_id,
            bool(getattr(fallback, "continue_conversation", False)), "chat",
            _was_published(fallback),
        )

    async def _spoken(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
        pack: str,
        speech: str,
        conversation_id: str | None,
        continue_conversation: bool,
        decision: str = "",
        published: bool = False,
    ) -> ConversationResult:
        agent_id = self._fallback_agent_id()
        if decision == "chat":
            speech = finish_clock_speech(speech, pack)
        if not skip_rewrite(decision):
            speech = await async_finish_speech(
                self.hass,
                bool(self._entry.options.get(CONF_REFINE_SPEECH)),
                agent_id,
                self._agent_controls_home(str(agent_id or "")),
                speech,
                user_input.context,
                speak_tag(pack),
                pack,
                self._personality(),
                str(self._entry.options.get(CONF_REFINE_PROMPT) or ""),
                self._allow_llm_tools(),
            )
        remember_turn(
            self.hass,
            self._entry.entry_id,
            user_input.text,
            speech,
            decision,
            self._preferred_area(user_input.device_id, getattr(user_input, "satellite_id", None)),
        )
        if speech.strip() and not published:
            await emit_assistant_speech(chat_log, user_input.agent_id, speech)
        response = intent.IntentResponse(language=speak_tag(pack))
        kinds = getattr(intent, "IntentResponseType", None)
        query = getattr(kinds, "QUERY_ANSWER", None) if kinds is not None else None
        if query is not None and decision in {"chat", "llm"}:
            response.response_type = query
        response.async_set_speech(speech)
        return ConversationResult(
            conversation_id=conversation_id,
            response=response,
            continue_conversation=continue_conversation,
        )

    async def _briefing(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
        pack: str,
        intro: str,
        conversation_id: str | None,
    ) -> ConversationResult | None:
        extra = getattr(user_input, "extra_system_prompt", None)
        extra_s = extra if isinstance(extra, str) else None
        announced = False
        if intro:
            announced = await announce(self.hass, user_input, intro)
            headlines = await fetch_headlines(self.hass, pack)
            prompt = news_prompt(pack, headlines, extra_s)
        else:
            prompt = news_followup_prompt(pack, extra_s)
        result = await self._fallback(user_input, chat_log, pack, True, prompt, publish=False)
        llm = _speech_from_result(result) if result is not None else ""
        extra_nudge = nudge(pack) if intro and not asked_for_more(llm) else ""
        spoken = compose_speech(intro, llm, extra_nudge, announced) or intro or _cue(_DONE, pack, "OK")
        self._note_llm_turn(user_input, spoken)
        return await self._spoken(user_input, chat_log, pack, spoken, conversation_id, True, "chat")

    def _nlu_rag(self) -> bool:
        return bool(self._entry.options.get(CONF_NLU_RAG, DEFAULT_NLU_RAG))

    def _quiet_ack(self) -> bool:
        return bool(self._entry.options.get(CONF_QUIET_ACK, DEFAULT_QUIET_ACK))

    def _calendar_llm(self) -> bool:
        return bool(self._entry.options.get(CONF_CALENDAR_LLM, DEFAULT_CALENDAR_LLM))

    def _allow_llm_tools(self) -> bool:
        return bool(self._entry.options.get(CONF_ALLOW_LLM_TOOLS, DEFAULT_ALLOW_LLM_TOOLS))

    async def async_reload(self, language: str | None = None) -> None:
        """Honor conversation.reload; registered intents are read live."""
        del language
        registered_intent_names(self.hass)

    async def _sentence_triggers(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
        pack: str,
    ) -> ConversationResult | None:
        handle = getattr(conversation, "async_handle_sentence_triggers", None)
        if handle is None:
            return None
        try:
            result = await handle(self.hass, user_input, chat_log)
        except TypeError:
            try:
                result = await handle(self.hass, user_input)
            except Exception:  # noqa: BLE001 — HA trigger API is a system boundary
                return None
        except Exception:  # noqa: BLE001 — HA trigger API is a system boundary
            return None
        if result is None:
            return None
        if isinstance(result, ConversationResult):
            return result
        speech = result if isinstance(result, str) else _speech_from_result(result)
        if not speech:
            return None
        return await self._spoken(user_input, chat_log, pack, speech, user_input.conversation_id, False, "trigger")

    async def _klar_tool_turn(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
        pack: str,
        fallback: ConversationResult,
    ) -> ConversationResult | None:
        if not self._nlu_rag():
            return None
        tool = parse_tool_reply(_speech_from_result(fallback))
        if tool is None:
            return None
        if tool.get("tool") == "klar.parse" and tool.get("text"):
            payload = await self._parse(
                str(tool["text"]), user_input.conversation_id, pack, user_input.device_id, getattr(user_input, "satellite_id", None)
            )
            if str((payload.get("decision") or {}).get("type") or "") == "execute":
                intents = home_intents(executable_intents(payload), registered_intent_names(self.hass))
                if intents:
                    executed = await execute_plan(self.hass, user_input, intents, pack, self._assistant(), self._exposed)
                    speech = str(executed.get("speech") or payload.get("speech") or _cue(_DONE, pack, "OK"))
                    return await self._spoken(
                        user_input, chat_log, pack, speech, payload.get("conversation_id"), False, "execute"
                    )
            speech = str(payload.get("speech") or "")
            return await self._spoken(user_input, chat_log, pack, speech, payload.get("conversation_id"), False)
        if tool.get("tool") == "klar.act" and tool.get("intent"):
            item = act_payload(str(tool["intent"]), tool.get("slots") or {})
            intents = home_intents([item], registered_intent_names(self.hass))
            if not intents:
                return None
            executed = await execute_plan(self.hass, user_input, intents, pack, self._assistant(), self._exposed)
            return await self._spoken(
                user_input,
                chat_log,
                pack,
                str(executed.get("speech") or _cue(_DONE, pack, "OK")),
                user_input.conversation_id,
                False,
                "execute",
            )
        return None

    async def _fallback(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
        pack: str,
        chat: bool = False,
        prompt: str | None = None,
        retrieval: dict[str, Any] | None = None,
        publish: bool = True,
    ) -> ConversationResult | None:
        agent_id = self._fallback_agent_id()
        if not agent_id:
            return None
        if not can_use_fallback_agent(
            self._agent_controls_home(agent_id), chat, self._allow_llm_tools()
        ):
            _LOGGER.warning("LLM-Fallback %s hat Assist-Werkzeuge — übersprungen", agent_id)
            return None
        extra_s = getattr(user_input, "extra_system_prompt", None)
        extra_s = extra_s if isinstance(extra_s, str) else None
        voice = refine_prompt(pack, self._personality(), str(self._entry.options.get(CONF_REFINE_PROMPT) or ""))
        if prompt:
            system = with_personality(prompt, voice)
        elif self._nlu_rag():
            system = rag_prompt(pack, retrieval, with_personality(extra_s, voice))
        elif yarn_request(user_input.text):
            system = yarn_prompt(pack, with_personality(extra_s, voice), user_input.text)
        else:
            system = chat_only_prompt(
                pack, with_personality(extra_s, voice), self._allow_llm_tools()
            )
        session_id = self._llm_session_id(user_input)
        prior = history_prompt(pack, self._llm_turns(session_id))
        if prior and not yarn_request(user_input.text):
            system = f"{system}\n\n{prior}"
        yarn = yarn_request(user_input.text)
        resolved = llm_client_and_model(self.hass, agent_id)
        if resolved is not None:
            streamed = await self._stream_fallback(
                resolved[0],
                resolved[1],
                user_input,
                pack,
                system,
                chat_log if publish else None,
                yarn,
            )
            if streamed is not None:
                return streamed
        try:
            result = await conversation.async_converse(
                self.hass,
                user_input.text,
                isolated_conversation_id(),
                user_input.context,
                **nested_llm_session(agent_id, speak_tag(pack), system),
            )
        except Exception as err:  # noqa: BLE001 — other agent is a system boundary
            _LOGGER.warning("LLM-Fallback fehlgeschlagen: %s", err)
            return None
        if result is not None and yarn and yarn_asks_permission(_speech_from_result(result)):
            try:
                result = await conversation.async_converse(
                    self.hass,
                    user_input.text,
                    isolated_conversation_id(),
                    user_input.context,
                    **nested_llm_session(agent_id, speak_tag(pack), yarn_nudge(pack, system)),
                )
            except Exception as err:  # noqa: BLE001 — other agent is a system boundary
                _LOGGER.warning("LLM-Fallback-Wiederholung fehlgeschlagen: %s", err)
        return result

    async def _stream_fallback(
        self,
        client: Any,
        model: str,
        user_input: ConversationInput,
        pack: str,
        system: str,
        chat_log: ChatLog | None,
        yarn: bool,
    ) -> ConversationResult | None:
        hold = _stream_hold(yarn, self._nlu_rag())
        try:
            speech, published = await stream_chat(
                client,
                model,
                user_input.text,
                system,
                chat_log,
                getattr(user_input, "agent_id", None),
                hold=hold,
            )
        except Exception as err:  # noqa: BLE001 — client shape varies by agent
            _LOGGER.warning("LLM-Stream fehlgeschlagen, converse: %s", err)
            return None
        if not speech:
            return None
        if yarn and yarn_asks_permission(speech) and not published:
            try:
                speech, published = await stream_chat(
                    client,
                    model,
                    user_input.text,
                    yarn_nudge(pack, system),
                    chat_log,
                    getattr(user_input, "agent_id", None),
                )
            except Exception as err:  # noqa: BLE001 — retry is best-effort
                _LOGGER.warning("LLM-Stream-Wiederholung fehlgeschlagen: %s", err)
                return None
            if not speech:
                return None
        return _speech_result(pack, speech, published and chat_log is not None)

    def _preferred_area(self, device_id: str | None, satellite_id: str | None = None) -> str | None:
        registry = device_registry.async_get(self.hass)
        for candidate in (device_id, satellite_id):
            if not candidate:
                continue
            device = registry.async_get(str(candidate))
            area = str(getattr(device, "area_id", "") or "") if device is not None else ""
            if area:
                return area
        return None

    async def _parse(
        self, text: str, conversation_id: str | None, language: str | None, device_id: str | None, satellite_id: str | None = None
    ) -> dict[str, Any]:
        pack = self._request_pack(language)
        body: dict[str, Any] = {
            "text": text,
            "conversation_id": parse_session_id(conversation_id, device_id, satellite_id),
            "language": pack,
            "personality": self._personality(),
        }
        if area := self._preferred_area(device_id, satellite_id):
            body["preferred_area"] = area
        if self._nlu_rag():
            body["nlu_rag"] = True
        payload, last_err = await post_parse(
            async_get_clientsession(self.hass), self._url, body, self._headers()
        )
        if payload is not None:
            return payload
        _LOGGER.warning("Klar nicht erreichbar: %s", last_err)
        return {
            "schema_version": "2.0",
            "speech": _cue(_UNREACHABLE, pack, "Klar is not responding right now."),
            "decision": {"type": "error", "code": "unreachable", "message": str(last_err or "")},
            "plan": None,
            "unreachable": True,
        }

    def _llm_session_id(self, user_input: ConversationInput) -> str:
        assist_id = str(user_input.conversation_id or "").strip()
        if assist_id:
            return llm_conversation_id(assist_id[:128])
        return llm_conversation_id(
            engine_session_id(user_input.device_id, getattr(user_input, "satellite_id", None))
        )

    def _llm_turns(self, session_id: str) -> list[tuple[str, str]]:
        store = self.hass.data.setdefault(DOMAIN, {}).setdefault("llm_turns", {})
        return list(store.get(session_id) or [])

    def _note_llm_turn(self, user_input: ConversationInput, speech: str) -> None:
        session_id = self._llm_session_id(user_input)
        store = self.hass.data.setdefault(DOMAIN, {}).setdefault("llm_turns", {})
        store[session_id] = append_llm_turn(store.get(session_id), user_input.text, speech)
