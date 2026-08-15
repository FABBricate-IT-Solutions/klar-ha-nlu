from __future__ import annotations

import logging
from typing import Any

import aiohttp
from homeassistant.components import conversation
from homeassistant.components.conversation import (
    AssistantContent,
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
from homeassistant.helpers.device_registry import DeviceInfo
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .const import (
    CONF_ASSIST_FILTER,
    CONF_FALLBACK_AGENT,
    CONF_LANGUAGES,
    CONF_PERSONALITY,
    CONF_REFINE_PROMPT,
    CONF_REFINE_SPEECH,
    CONF_TOKEN,
    CONF_URL,
    DEFAULT_ASSIST_FILTER,
    DEFAULT_URL,
    DOMAIN,
    LANGUAGE_VARIANTS,
    resolve_personality,
    SUPPORTED_LANGUAGES,
)
from .fallback import (
    agent_has_home_control,
    can_use_fallback_agent,
    chat_only_prompt,
    news_followup_prompt,
    news_prompt,
)
from .intents import (
    ENTITY_SERVICES,
    LIST_INTENTS,
    TIMER_INTENTS,
    area_label,
    home_intents,
    item_slots,
    list_slots,
    timer_slots,
)
from .news import announce, asked_for_more, compose_speech, fetch_headlines, nudge
from .refine import async_refine_speech, should_refine
from .speech import from_handled, style

_LOGGER = logging.getLogger(__name__)

_UNREACHABLE = {
    "de": "Klar antwortet gerade nicht.",
    "en": "Klar is not responding right now.",
}

_DONE = {"de": "Erledigt.", "en": "Done."}


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


def _pack(language: str | None, enabled: list[str] | None = None) -> str:
    if language:
        code = language.replace("_", "-").split("-", 1)[0].lower()
        if code in SUPPORTED_LANGUAGES:
            return code
    if enabled:
        return enabled[0]
    return "de"


def _enabled_packs(entry: ConfigEntry) -> list[str]:
    raw = entry.options.get(CONF_LANGUAGES)
    if not isinstance(raw, list) or not raw:
        return list(SUPPORTED_LANGUAGES)
    packs = [code for code in raw if code in SUPPORTED_LANGUAGES]
    return packs or list(SUPPORTED_LANGUAGES)


def _advertise(packs: list[str]) -> list[str]:
    out: list[str] = []
    for pack in packs:
        out.extend(LANGUAGE_VARIANTS.get(pack, (pack,)))
    return out


class KlarConversationEntity(ConversationEntity):
    _attr_name = "Klar NLU"
    _attr_supported_features = conversation.ConversationEntityFeature.CONTROL

    def __init__(self, hass: HomeAssistant, entry: ConfigEntry) -> None:
        self.hass = hass
        self._entry = entry
        self._attr_unique_id = entry.entry_id
        self._attr_device_info = DeviceInfo(
            identifiers={(DOMAIN, entry.entry_id)},
            name="Klar NLU",
            manufacturer="FABBricate IT Solutions",
        )

    @property
    def supported_languages(self) -> list[str]:
        return _advertise(_enabled_packs(self._entry))

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
            return True

    async def _async_handle_message(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
    ) -> ConversationResult:
        pack = _pack(user_input.language, _enabled_packs(self._entry))
        payload = await self._parse(
            user_input.text, user_input.conversation_id, pack
        )
        engine_speech = str(payload.get("speech") or "")
        speech = engine_speech or _DONE[pack]
        intents = home_intents(payload.get("intents") or [])
        clarify = bool(payload.get("clarify"))
        conversation_id = payload.get("conversation_id") or user_input.conversation_id
        personality = self._personality()

        if payload.get("briefing"):
            if payload.get("chat"):
                briefing = await self._briefing(
                    user_input, chat_log, pack, engine_speech, conversation_id
                )
                if briefing is not None:
                    return briefing
            else:
                return self._spoken(
                    user_input, chat_log, pack, engine_speech or _DONE[pack], conversation_id, False
                )

        if not clarify and not intents and not payload.get("unreachable"):
            fallback = await self._fallback(
                user_input, chat_log, pack, bool(payload.get("chat"))
            )
            if fallback is not None:
                return fallback

        names = {item.get("name") for item in intents}
        spoken: list[str] = []
        for item in intents:
            if item.get("name") == "HassVacuumReturnToBase" and "HassGetState" in names:
                continue
            ha_speech = await self._handle_intent(user_input, item, pack)
            if ha_speech:
                spoken.append(ha_speech)
        if spoken:
            speech = " ".join(spoken)
        if not clarify:
            speech = style(speech, personality, pack)
        agent_id = self._fallback_agent_id()
        home_turn = bool(intents) and not clarify
        if should_refine(
            bool(self._entry.options.get(CONF_REFINE_SPEECH)),
            agent_id,
            speech,
            home_turn,
        ):
            refined = await async_refine_speech(
                self.hass,
                str(agent_id),
                self._agent_controls_home(str(agent_id)),
                speech,
                user_input.context,
                user_input.language,
                pack,
                personality,
                str(self._entry.options.get(CONF_REFINE_PROMPT) or ""),
            )
            if refined:
                speech = style(refined, personality, pack)

        return self._spoken(user_input, chat_log, pack, speech, conversation_id, clarify)

    def _spoken(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
        pack: str,
        speech: str,
        conversation_id: str | None,
        continue_conversation: bool,
    ) -> ConversationResult:
        chat_log.async_add_assistant_content_without_tools(
            AssistantContent(agent_id=user_input.agent_id, content=speech)
        )
        response = intent.IntentResponse(language=user_input.language or pack)
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
        result = await self._fallback(user_input, chat_log, pack, True, prompt, False)
        llm = _speech_from_result(result) if result is not None else ""
        extra_nudge = nudge(pack) if intro and not asked_for_more(llm) else ""
        spoken = compose_speech(intro, llm, extra_nudge, announced) or intro or _DONE[pack]
        if result is None:
            return self._spoken(user_input, chat_log, pack, spoken, conversation_id, True)
        result.response.async_set_speech(spoken)
        return self._spoken(
            user_input,
            chat_log,
            pack,
            spoken,
            result.conversation_id or conversation_id,
            True,
        )

    async def _fallback(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
        pack: str,
        chat: bool = False,
        prompt: str | None = None,
        record: bool = True,
    ) -> ConversationResult | None:
        agent_id = self._fallback_agent_id()
        if not agent_id:
            return None
        if not can_use_fallback_agent(self._agent_controls_home(agent_id), chat):
            _LOGGER.warning("LLM-Fallback %s hat Assist-Werkzeuge — übersprungen", agent_id)
            return None
        extra = getattr(user_input, "extra_system_prompt", None)
        system = prompt or chat_only_prompt(pack, extra if isinstance(extra, str) else None)
        try:
            result = await conversation.async_converse(
                self.hass,
                user_input.text,
                user_input.conversation_id,
                user_input.context,
                language=user_input.language,
                agent_id=agent_id,
                device_id=user_input.device_id,
                satellite_id=getattr(user_input, "satellite_id", None),
                extra_system_prompt=system,
            )
        except Exception as err:  # noqa: BLE001 — other agent is a system boundary
            _LOGGER.warning("LLM-Fallback fehlgeschlagen: %s", err)
            return None
        if not record:
            return result
        last = chat_log.content[-1] if chat_log.content else None
        if getattr(last, "role", None) != "assistant":
            speech = _speech_from_result(result)
            if speech:
                chat_log.async_add_assistant_content_without_tools(
                    AssistantContent(agent_id=user_input.agent_id, content=speech)
                )
        return result

    async def _parse(
        self, text: str, conversation_id: str | None, language: str | None
    ) -> dict[str, Any]:
        url = f"{self._url}/api/parse"
        pack = _pack(language, _enabled_packs(self._entry))
        body: dict[str, Any] = {
            "text": text,
            "conversation_id": conversation_id,
            "language": pack,
            "personality": self._personality(),
        }
        try:
            session = async_get_clientsession(self.hass)
            async with session.post(
                url,
                json=body,
                headers=self._headers(),
                timeout=aiohttp.ClientTimeout(total=5),
            ) as resp:
                resp.raise_for_status()
                return await resp.json()
        except Exception as err:  # noqa: BLE001 — boundary to the local engine
            _LOGGER.warning("Klar nicht erreichbar: %s", err)
            return {
                "speech": _UNREACHABLE[pack],
                "intents": [],
                "unreachable": True,
            }

    async def _invoke_intent(
        self,
        user_input: ConversationInput,
        name: str,
        slots: dict[str, Any],
        pack: str,
        item: dict,
    ) -> str | None:
        try:
            handled = await intent.async_handle(
                self.hass,
                "klar_nlu",
                name,
                slots,
                user_input.text,
                user_input.context,
                user_input.language or pack,
                assistant=self._assistant(),
            )
        except Exception as err:  # noqa: BLE001 — HA intent system is a boundary
            _LOGGER.debug("Intent %s nicht ausgeführt: %s", name, err)
            return None
        return from_handled(handled, pack, {**item, "name": name})

    async def _run_entity(
        self,
        name: str,
        entity_id: str,
        slots: dict[str, Any],
        pack: str,
        item: dict,
    ) -> str | None:
        if "." not in entity_id or self.hass.states.get(entity_id) is None:
            return None
        if not self._exposed(entity_id):
            return None
        domain = entity_id.split(".", 1)[0]
        data: dict[str, Any] = {"entity_id": entity_id}
        if name == "HassLightSet" and domain == "light":
            service = "turn_on"
            if (bri := slots.get("brightness", {}).get("value")) is not None:
                try:
                    data["brightness_pct"] = max(0, min(100, int(bri)))
                except (TypeError, ValueError):
                    pass
            if color := slots.get("color", {}).get("value"):
                data["color_name"] = str(color)
        else:
            service = ENTITY_SERVICES.get(name)
            if not service:
                return None
        try:
            await self.hass.services.async_call(domain, service, data, blocking=True)
        except Exception as err:  # noqa: BLE001 — HA services are a boundary
            _LOGGER.debug("Gerät %s nicht geschaltet: %s", entity_id, err)
            return None
        state = self.hass.states.get(entity_id)
        attrs = getattr(state, "attributes", None) or {}
        pretty = ""
        if isinstance(attrs, dict):
            pretty = str(attrs.get("friendly_name") or "")
        pretty = pretty or str(getattr(state, "name", None) or "")
        spoken = {**item, "name": name, "slots": [*(item.get("slots") or []), {"name": "name", "value": pretty}]}
        return from_handled(None, pack, spoken)

    async def _handle_intent(
        self, user_input: ConversationInput, item: dict, pack: str
    ) -> str | None:
        name = item.get("name")
        if not name:
            return None
        slots = item_slots(item)
        if "entity_id" in slots:
            entity_id = str(slots["entity_id"].get("value") or "")
            spoken = await self._run_entity(name, entity_id, slots, pack, item)
            if spoken:
                return spoken
            state = self.hass.states.get(entity_id)
            if state is not None:
                slots["name"] = {"value": state.name}
            if "." in entity_id:
                slots.setdefault("domain", {"value": entity_id.split(".", 1)[0]})
            slots.pop("area", None)
        if name in LIST_INTENTS:
            name, slots = list_slots(self.hass, name, slots)
        if name in TIMER_INTENTS:
            slots = timer_slots(slots)
            if name == "HassStartTimer" and not any(
                key in slots for key in ("hours", "minutes", "seconds")
            ):
                return None
        if name == "HassGetState" and slots.get("device_class", {}).get("value") == "temperature":
            slots.pop("domain", None)
            speech = await self._invoke_intent(user_input, name, slots, pack, item)
            if speech:
                return speech
            climate = {key: val for key, val in slots.items() if key != "device_class"}
            return await self._invoke_intent(
                user_input, "HassClimateGetTemperature", climate, pack, item
            )
        if name == "HassGetState" and "area" in slots and "entity_id" not in slots:
            label = area_label(self.hass, str(slots["area"].get("value") or ""))
            if label:
                item = {
                    **item,
                    "slots": [*(item.get("slots") or []), {"name": "area_name", "value": label}],
                }
        return await self._invoke_intent(user_input, name, slots, pack, item)
