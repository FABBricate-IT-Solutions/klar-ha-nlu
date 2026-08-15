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
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers import intent
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .const import (
    CONF_ASSIST_FILTER,
    CONF_FALLBACK_AGENT,
    CONF_LANGUAGES,
    CONF_URL,
    DEFAULT_ASSIST_FILTER,
    DEFAULT_URL,
    LANGUAGE_VARIANTS,
    SUPPORTED_LANGUAGES,
)

_LOGGER = logging.getLogger(__name__)

_CHAT_ONLY = {
    "de": (
        "Du antwortest nur im Gespräch. Steuere keine Geräte "
        "und rufe keine Home-Assistant-Werkzeuge auf."
    ),
    "en": (
        "Reply in conversation only. Do not control devices "
        "and do not call Home Assistant tools."
    ),
}

_UNREACHABLE = {
    "de": "Klar antwortet gerade nicht.",
    "en": "Klar is not responding right now.",
}

_DONE = {"de": "Erledigt.", "en": "Done."}

_ACTION = {
    "HassTurnOn": {"de": "Schalte {where} ein.", "en": "Turn on {where}."},
    "HassTurnOff": {"de": "Schalte {where} aus.", "en": "Turn off {where}."},
    "HassToggle": {"de": "Schalte {where} um.", "en": "Toggle {where}."},
    "HassLightSet": {"de": "Setze {where}.", "en": "Set {where}."},
}

_DE_ENGINE = ("Schalte", "Frage", "Setze", "Sag mir", "Meinst du")

_TIMER_INTENTS = {
    "HassStartTimer",
    "HassIncreaseTimer",
    "HassDecreaseTimer",
    "HassCancelTimer",
    "HassPauseTimer",
}


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    async_add_entities([KlarConversationEntity(hass, entry)])


def _timer_slots(slots: dict[str, Any]) -> dict[str, Any]:
    if "duration" in slots:
        duration = slots.pop("duration")
        if "minutes" not in slots and "hours" not in slots and "seconds" not in slots:
            slots["minutes"] = duration
    slots.pop("entity_id", None)
    slots.pop("domain", None)
    return slots


def _list_slots(
    hass: HomeAssistant, name: str, slots: dict[str, Any]
) -> tuple[str, dict[str, Any]]:
    if name.startswith("HassShoppingList"):
        name = name.replace("HassShoppingList", "HassList")
    entity = slots.pop("entity_id", None)
    slots.pop("domain", None)
    entity_id = str((entity or {}).get("value") or "")
    if entity_id:
        state = hass.states.get(entity_id)
        if state is not None:
            slots["name"] = {"value": state.name}
    slots.setdefault("name", {"value": "shopping_list"})
    return name, slots


def _home_intents(intents: list[Any]) -> list[dict[str, Any]]:
    out: list[dict[str, Any]] = []
    for item in intents:
        if not isinstance(item, dict) or not item.get("name") or item["name"] == "Unknown":
            continue
        if item["name"] == "HassGetState" and not _get_state_has_target(item):
            continue
        out.append(item)
    return out


def _get_state_has_target(item: dict[str, Any]) -> bool:
    return any(
        isinstance(slot, dict)
        and slot.get("name") in {"area", "entity_id", "name", "device_class", "domain"}
        for slot in (item.get("slots") or [])
    )


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


def _plain_speech(handled: Any) -> str:
    speech = getattr(handled, "speech", None) or {}
    plain = speech.get("plain") if isinstance(speech, dict) else None
    if isinstance(plain, dict):
        text = str(plain.get("speech") or "").strip()
        if text:
            return text
    as_dict = getattr(handled, "as_dict", None)
    if callable(as_dict):
        data = as_dict()
        nested = (data.get("speech") or {}).get("plain") or {}
        return str(nested.get("speech") or "").strip()
    return ""


def _is_query(handled: Any, name: str) -> bool:
    rtype = getattr(handled, "response_type", None)
    value = getattr(rtype, "value", rtype)
    return str(value) == "query_answer" or name in {
        "HassGetState",
        "HassClimateGetTemperature",
    }


def _state_value(state: Any) -> tuple[str, str]:
    attrs = getattr(state, "attributes", None) or {}
    if not isinstance(attrs, dict):
        attrs = {}
    unit = str(attrs.get("unit_of_measurement") or attrs.get("temperature_unit") or "")
    name = str(attrs.get("friendly_name") or "")
    name = name or str(getattr(state, "name", None) or getattr(state, "entity_id", ""))
    raw = attrs.get("current_temperature")
    if raw is None or raw == "":
        raw = getattr(state, "state", "")
    elif not unit:
        unit = "°C"
    value = str(raw).replace(".", ",")
    spoken = f"{value} {unit}".strip() if unit else value
    return name, spoken


def _query_speech(handled: Any, pack: str) -> str:
    states = list(getattr(handled, "matched_states", None) or [])
    if not states:
        states = list(getattr(handled, "unmatched_states", None) or [])
    parts: list[str] = []
    for state in states[:4]:
        name, spoken = _state_value(state)
        if not name or not spoken:
            continue
        if pack == "en":
            parts.append(f"{name} is {spoken.replace(',', '.')}.")
        else:
            parts.append(f"{name}: {spoken}.")
    return " ".join(parts)


def _where(handled: Any, item: dict) -> str:
    names = [
        str(getattr(target, "name", None) or getattr(target, "id", "") or "")
        for target in getattr(handled, "success_results", None) or []
    ]
    names = [name for name in names if name]
    if names:
        return ", ".join(dict.fromkeys(names))
    slots = {
        slot["name"]: slot["value"]
        for slot in item.get("slots") or []
        if isinstance(slot, dict) and slot.get("name")
    }
    return str(slots.get("area") or slots.get("name") or slots.get("entity_id") or "")


def _speech_from_handled(handled: Any, pack: str, item: dict) -> str | None:
    text = _plain_speech(handled)
    if text:
        return text
    name = str(item.get("name") or "")
    if _is_query(handled, name):
        query = _query_speech(handled, pack)
        if query:
            return query
    where = _where(handled, item) or ("home" if pack == "en" else "Zuhause")
    template = (_ACTION.get(name) or {}).get(pack)
    if template:
        return template.format(where=where)
    query = _query_speech(handled, pack)
    return query or None


def _engine_ok(speech: str, pack: str) -> bool:
    if not speech:
        return False
    if pack != "en":
        return True
    return not any(marker in speech for marker in _DE_ENGINE)


class KlarConversationEntity(ConversationEntity):
    _attr_name = "Klar NLU"
    _attr_supported_features = conversation.ConversationEntityFeature.CONTROL

    def __init__(self, hass: HomeAssistant, entry: ConfigEntry) -> None:
        self.hass = hass
        self._entry = entry
        self._attr_unique_id = entry.entry_id

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
        speech = engine_speech if _engine_ok(engine_speech, pack) else _DONE[pack]
        intents = _home_intents(payload.get("intents") or [])
        clarify = bool(payload.get("clarify"))
        conversation_id = payload.get("conversation_id") or user_input.conversation_id

        if not clarify and not intents and not payload.get("unreachable"):
            fallback = await self._fallback(user_input, chat_log, pack)
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

        chat_log.async_add_assistant_content_without_tools(
            AssistantContent(agent_id=user_input.agent_id, content=speech)
        )
        response = intent.IntentResponse(language=user_input.language or pack)
        response.async_set_speech(speech)
        return ConversationResult(
            conversation_id=conversation_id,
            response=response,
            continue_conversation=clarify,
        )

    async def _fallback(
        self,
        user_input: ConversationInput,
        chat_log: ChatLog,
        pack: str,
    ) -> ConversationResult | None:
        agent_id = self._fallback_agent_id()
        if not agent_id:
            return None
        extra = getattr(user_input, "extra_system_prompt", None)
        only = _CHAT_ONLY[pack]
        prompt = f"{extra}\n{only}" if extra else only
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
                extra_system_prompt=prompt,
            )
        except Exception as err:  # noqa: BLE001 — other agent is a system boundary
            _LOGGER.warning("LLM-Fallback fehlgeschlagen: %s", err)
            return None
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
        }
        try:
            async with aiohttp.ClientSession() as session:
                async with session.post(
                    url,
                    json=body,
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
        return _speech_from_handled(handled, pack, {**item, "name": name})

    async def _handle_intent(
        self, user_input: ConversationInput, item: dict, pack: str
    ) -> str | None:
        name = item.get("name")
        if not name:
            return None
        slots = {s["name"]: {"value": s["value"]} for s in item.get("slots") or []}
        if name in {
            "HassListAddItem",
            "HassListCompleteItem",
            "HassShoppingListAddItem",
            "HassShoppingListCompleteItem",
        }:
            name, slots = _list_slots(self.hass, name, slots)
        if name in _TIMER_INTENTS:
            slots = _timer_slots(slots)
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
        return await self._invoke_intent(user_input, name, slots, pack, item)
