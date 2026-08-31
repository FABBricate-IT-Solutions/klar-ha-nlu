"""Run a parsed Klar intent through Home Assistant."""

from __future__ import annotations

import logging
from collections.abc import Callable
from dataclasses import dataclass
from types import SimpleNamespace
from typing import Any

from homeassistant.components.conversation import ConversationInput
from homeassistant.core import HomeAssistant
from homeassistant.helpers import entity_registry, intent

from .calendar_ha import CALENDAR_INTENTS, handle_calendar_intent
from .lang_select import speak_tag
from .intents import (
    ENTITY_SERVICES,
    LIST_INTENTS,
    MASS_INTENTS,
    TIMER_INTENTS,
    area_label,
    item_slots,
    list_slots,
    resolve_area,
    timer_slots,
)
from .speech import from_handled, media_state_speech, queue_speech

_LOGGER = logging.getLogger(__name__)

# HA has no native intent for these Music Assistant / mute / relative-volume actions.
_SERVICE_ONLY = {
    "HassSetVolumeRelative",
    "HassMediaPause",
    "HassMediaUnpause",
    "HassMediaNext",
    "HassMediaPrevious",
    "HassMediaPlayerMute",
    "HassMediaPlayerUnmute",
}
_HA_INTENT_ALIASES = {
    "HassMediaNext": "HassNext",
    "HassMediaPrevious": "HassPrevious",
}


@dataclass(frozen=True)
class IntentStepResult:
    ok: bool
    speech: str | None = None
    error: str | None = None


def _ok(speech: str | None) -> IntentStepResult:
    if speech:
        return IntentStepResult(True, speech=speech)
    return IntentStepResult(False, error="empty_speech")


def _fail(error: str) -> IntentStepResult:
    return IntentStepResult(False, error=error[:256])


async def handle_intent(
    hass: HomeAssistant,
    user_input: ConversationInput,
    item: dict,
    pack: str,
    assistant: str | None,
    exposed: Callable[[str], bool],
) -> IntentStepResult:
    name = item.get("name")
    if not name:
        return _fail("missing_intent")
    if name in CALENDAR_INTENTS:
        ok, speech, error = await handle_calendar_intent(
            hass, item, pack, exposed, getattr(user_input, "conversation_id", None)
        )
        return _ok(speech) if ok else _fail(error or "calendar_failed")
    slots = item_slots(item)
    entity_id = str(slots.get("entity_id", {}).get("value") or "")
    if name == "HassTurnOn" and tv_request(getattr(user_input, "text", "")):
        state = hass.states.get(entity_id) if entity_id else None
        if not entity_id or not tv_named(entity_id, state):
            return _fail("media_unavailable")
    if name == "HassMediaSearchAndPlay" and music_assistant_player(hass, entity_id):
        query = str(slots.get("media_id", {}).get("value") or slots.get("search_query", {}).get("value") or "")
        if query:
            slots = {**slots, "media_id": {"value": query}}
            item = {**item, "name": "MassPlayMedia"}
            return await run_mass(hass, "MassPlayMedia", slots, pack, item, exposed)
    if name in MASS_INTENTS:
        return await run_mass(hass, name, slots, pack, item, exposed)
    media_status = str(slots.get("media_status", {}).get("value") or "")
    if name == "HassGetState" and media_status:
        # Custom now-playing speech; HA has no media_status intent.
        state = hass.states.get(entity_id) if entity_id.startswith("media_player.") else None
        if (
            state is None
            or not exposed(entity_id)
            or media_missing(state)
        ):
            return _fail("media_status_unavailable")
        return _ok(media_state_speech(state, media_status, pack))
    if name == "HassMediaUnpause" and entity_id:
        started = await start_idle_music(hass, entity_id, pack, item, exposed)
        if started is not None:
            return started
    if name in _SERVICE_ONLY:
        if not entity_id:
            return _fail("missing_entity")
        return await run_entity(hass, name, entity_id, slots, pack, item, exposed)
    if name == "HassLightSet" and not entity_id:
        return _fail("missing_entity")
    if entity_id and (name in ENTITY_SERVICES or name in {"HassLightSet", "HassClimateSetTemperature"}):
        return await run_entity(hass, name, entity_id, slots, pack, item, exposed)
    if name == "HassClimateSetTemperature":
        return await climate_set(hass, slots, pack, item, exposed)
    if entity_id:
        if not exposed(entity_id):
            return _fail("entity_not_exposed")
        state = hass.states.get(entity_id)
        if state is None:
            return _fail("entity_unavailable")
        domain = entity_id.split(".", 1)[0]
        if domain == "media_player" and media_missing(state):
            return _fail("media_unavailable")
        slots["name"] = {"value": state.name}
        slots.setdefault("domain", {"value": domain})
        slots.pop("area", None)
    if name in LIST_INTENTS:
        name, slots = list_slots(hass, name, slots)
    if name in TIMER_INTENTS:
        slots = timer_slots(slots)
        if name == "HassStartTimer" and not any(key in slots for key in ("hours", "minutes", "seconds")):
            return _fail("missing_timer_duration")
    if name == "HassClimateGetTemperature":
        return await climate_query(hass, user_input, item, slots, pack, assistant, exposed)
    if name == "HassGetState" and slots.get("device_class", {}).get("value") == "temperature":
        slots.pop("domain", None)
        first = await invoke_intent(hass, user_input, name, slots, pack, item, assistant)
        if first.ok:
            return first
        return await climate_query(hass, user_input, item, slots, pack, assistant, exposed)
    if "area" in slots and "entity_id" not in slots:
        slots, item = bind_area_name(hass, slots, item)
    return await invoke_intent(hass, user_input, _HA_INTENT_ALIASES.get(name, name), slots, pack, item, assistant)


def bind_area_name(hass: HomeAssistant, slots: dict[str, Any], item: dict) -> tuple[dict[str, Any], dict]:
    area_id = str(slots.get("area", {}).get("value") or "")
    if not area_id:
        return slots, item
    label = area_label(hass, area_id)
    if not label:
        return slots, item
    slots = {**slots, "area": {"value": label}}
    existing = item.get("slots") or []
    if any(isinstance(slot, dict) and slot.get("name") == "area_name" for slot in existing):
        return slots, item
    return slots, {**item, "slots": [*existing, {"name": "area_name", "value": label}]}


async def climate_query(
    hass: HomeAssistant,
    user_input: ConversationInput,
    item: dict,
    slots: dict[str, Any],
    pack: str,
    assistant: str | None,
    exposed: Callable[[str], bool],
) -> IntentStepResult:
    entity_id = str(slots.get("entity_id", {}).get("value") or "")
    state = hass.states.get(entity_id) if entity_id else None
    shaped = {key: val for key, val in slots.items() if key not in {"entity_id", "domain", "device_class"}}
    if entity_id and state is not None:
        shaped["name"] = {"value": state.name}
    elif "area" in shaped:
        shaped, item = bind_area_name(hass, shaped, item)
    first = await invoke_intent(hass, user_input, "HassClimateGetTemperature", shaped, pack, item, assistant)
    if first.ok:
        return first
    states: list[Any] = []
    if state is not None and entity_id and exposed(entity_id):
        states = [state]
    else:
        area_key = str(slots.get("area", {}).get("value") or "")
        states = [item_state for item_state in climate_states_in_area(hass, area_key) if exposed(item_state.entity_id)]
    if not states:
        return first
    spoken = from_handled(
        SimpleNamespace(
            matched_states=states,
            unmatched_states=[],
            success_results=[],
            response_type="query_answer",
        ),
        pack,
        {**item, "name": "HassClimateGetTemperature"},
    )
    return _ok(spoken) if spoken else first


def climate_states_in_area(hass: HomeAssistant, area_key: str) -> list[Any]:
    if not area_key:
        return []
    area = resolve_area(hass, area_key)
    if area is None:
        return []
    registry = entity_registry.async_get(hass)
    found: list[Any] = []
    for state in hass.states.async_all("climate"):
        entry = registry.async_get(state.entity_id)
        if entry is not None and entry.area_id == area.id:
            found.append(state)
    return found


async def invoke_intent(
    hass: HomeAssistant,
    user_input: ConversationInput,
    name: str,
    slots: dict[str, Any],
    pack: str,
    item: dict,
    assistant: str | None,
) -> IntentStepResult:
    try:
        handled = await intent.async_handle(
            hass,
            "klar_nlu",
            name,
            slots,
            user_input.text,
            user_input.context,
            speak_tag(pack),
            assistant=assistant,
        )
    except Exception as err:  # noqa: BLE001 — HA intent system is a boundary
        _LOGGER.debug("Intent %s nicht ausgeführt: %s", name, err)
        return _fail(str(err) or name)
    return _ok(from_handled(handled, pack, {**item, "name": name}))


async def run_entity(
    hass: HomeAssistant,
    name: str,
    entity_id: str,
    slots: dict[str, Any],
    pack: str,
    item: dict,
    exposed: Callable[[str], bool],
) -> IntentStepResult:
    state = hass.states.get(entity_id)
    if "." not in entity_id or state is None:
        return _fail("entity_unavailable")
    if not exposed(entity_id):
        return _fail("entity_not_exposed")
    domain = entity_id.split(".", 1)[0]
    if domain == "media_player" and media_missing(state):
        return _fail("media_unavailable")
    data: dict[str, Any] = {"entity_id": entity_id}
    if name == "HassLightSet" and domain == "light":
        service = "turn_on"
        data.update(light_turn_on(slots))
    elif name == "HassClimateSetTemperature" and domain == "climate":
        service = "set_temperature"
        extra = climate_set_data(slots, state)
        if extra is None:
            return _fail("invalid_temperature")
        data.update(extra)
    elif domain == "media_player" and name in MEDIA_SERVICES:
        service = MEDIA_SERVICES[name]
        if name == "HassSetVolume":
            try:
                data["volume_level"] = max(0.0, min(1.0, float(slots["volume_level"]["value"]) / 100.0))
            except (KeyError, TypeError, ValueError):
                return _fail("invalid_volume")
        elif name == "HassSetVolumeRelative":
            step = str(slots.get("volume_step", {}).get("value") or "")
            service = "volume_down" if step == "down" else "volume_up"
        elif name in {"HassMediaPlayerMute", "HassMediaPlayerUnmute"}:
            data["is_volume_muted"] = name == "HassMediaPlayerMute"
    else:
        service = ENTITY_SERVICES.get(name)
        if not service:
            return _fail("unsupported_service")
    try:
        await hass.services.async_call(domain, service, data, blocking=True)
    except Exception as err:  # noqa: BLE001 — HA services are a boundary
        _LOGGER.debug("Gerät %s nicht geschaltet: %s", entity_id, err)
        return _fail(str(err) or "service_failed")
    state = hass.states.get(entity_id)
    attrs = getattr(state, "attributes", None) or {}
    pretty = ""
    if isinstance(attrs, dict):
        pretty = str(attrs.get("friendly_name") or "")
    pretty = pretty or str(getattr(state, "name", None) or "")
    spoken = {**item, "name": name, "slots": [*(item.get("slots") or []), {"name": "name", "value": pretty}]}
    return _ok(from_handled(None, pack, spoken))


MEDIA_SERVICES = {
    "HassMediaPause": "media_pause",
    "HassMediaUnpause": "media_play",
    "HassMediaNext": "media_next_track",
    "HassMediaPrevious": "media_previous_track",
    "HassMediaPlayerMute": "volume_mute",
    "HassMediaPlayerUnmute": "volume_mute",
    "HassSetVolume": "volume_set",
    "HassSetVolumeRelative": "volume_up",
}


# Music Assistant has no native Home Assistant intents; these stay as service calls.
async def run_mass(
    hass: HomeAssistant,
    name: str,
    slots: dict[str, Any],
    pack: str,
    item: dict,
    exposed: Callable[[str], bool],
) -> IntentStepResult:
    entity_id = str(slots.get("entity_id", {}).get("value") or "")
    state = hass.states.get(entity_id) if entity_id.startswith("media_player.") else None
    if (
        state is None
        or not exposed(entity_id)
        or media_missing(state)
    ):
        return _fail("mass_target_unavailable")
    if name == "MassFavorite":
        button = favorite_button(hass, entity_id)
        if not button:
            return _fail("favorite_button_missing")
        try:
            await hass.services.async_call("button", "press", {"entity_id": button}, blocking=True)
        except Exception as err:  # noqa: BLE001 — HA services are a boundary
            _LOGGER.debug("Favorit für %s nicht gesetzt: %s", entity_id, err)
            return _fail(str(err) or "favorite_failed")
        return _ok(from_handled(None, pack, {**item, "name": name}))
    try:
        if name == "MassGetQueue":
            response = await call_with_response(hass, "music_assistant", "get_queue", {}, {"entity_id": entity_id})
            return _ok(queue_speech(response, state, pack))
        if name == "MassTransferQueue":
            data = clean_service_data(slots, ["source_player", "auto_play"])
            source_player = str(data.get("source_player") or "")
            source_state = (
                hass.states.get(source_player)
                if source_player.startswith("media_player.")
                else None
            )
            if (
                source_state is None
                or source_player == entity_id
                or not exposed(source_player)
                or media_missing(source_state)
            ):
                return _fail("transfer_source_unavailable")
            await hass.services.async_call("music_assistant", "transfer_queue", data, blocking=True, target={"entity_id": entity_id})
        elif name == "MassPlayMedia":
            data = clean_service_data(slots, ["media_id", "media_type", "artist", "album", "enqueue", "radio_mode", "username"])
            if "radio_mode" in data:
                data["radio_mode"] = str(data["radio_mode"]).lower() == "true"
            await hass.services.async_call("music_assistant", "play_media", data, blocking=True, target={"entity_id": entity_id})
        else:
            return _fail("unsupported_mass_intent")
    except Exception as err:  # noqa: BLE001 — Music Assistant is a service boundary
        _LOGGER.debug("Music Assistant Intent %s fehlgeschlagen: %s", name, err)
        return _fail(str(err) or "mass_failed")
    return _ok(from_handled(None, pack, {**item, "name": name}))


async def call_with_response(
    hass: HomeAssistant,
    domain: str,
    service: str,
    data: dict[str, Any],
    target: dict[str, Any],
) -> Any:
    try:
        return await hass.services.async_call(domain, service, data, blocking=True, target=target, return_response=True)
    except TypeError:
        await hass.services.async_call(domain, service, data, blocking=True, target=target)
        return None


def clean_service_data(slots: dict[str, Any], names: list[str]) -> dict[str, Any]:
    data: dict[str, Any] = {}
    for name in names:
        value = slots.get(name, {}).get("value")
        if value not in (None, ""):
            data[name] = value
    return data


def media_missing(state: Any) -> bool:
    return str(getattr(state, "state", "")).lower() in {"unavailable", "unknown"}


def light_turn_on(slots: dict[str, Any]) -> dict[str, Any]:
    data: dict[str, Any] = {}
    if (bri := slots.get("brightness", {}).get("value")) is not None:
        try:
            data["brightness_pct"] = max(0, min(100, int(bri)))
        except (TypeError, ValueError):
            pass
    step = str(slots.get("brightness_step", {}).get("value") or "")
    if step in {"up", "down"}:
        data["brightness_step_pct"] = 15 if step == "up" else -15
    color = str(slots.get("color", {}).get("value") or "")
    key = color.casefold().replace(" ", "").replace("ß", "ss")
    if key in {"warmwhite", "warmweiss"}:
        data["color_temp_kelvin"] = 2700
    elif color:
        data["color_name"] = color
    return data


def climate_set_data(slots: dict[str, Any], state: Any) -> dict[str, Any] | None:
    try:
        data = {"temperature": float(slots.get("temperature", {}).get("value"))}
    except (TypeError, ValueError):
        return None
    attrs = getattr(state, "attributes", None) or {}
    mode = str((attrs.get("hvac_mode") if isinstance(attrs, dict) else None) or getattr(state, "state", "") or "")
    if mode.casefold() in {"off", "idle", "frost"}:
        data["hvac_mode"] = "heat"
    return data


async def climate_set(
    hass: HomeAssistant,
    slots: dict[str, Any],
    pack: str,
    item: dict,
    exposed: Callable[[str], bool],
) -> IntentStepResult:
    area_key = str(slots.get("area", {}).get("value") or "")
    states = [state for state in climate_states_in_area(hass, area_key) if exposed(state.entity_id)]
    if not states:
        return _fail("climate_unavailable")
    last = _fail("climate_unavailable")
    for state in states:
        last = await run_entity(hass, "HassClimateSetTemperature", state.entity_id, slots, pack, item, exposed)
        if last.ok:
            return last
    return last


async def start_idle_music(
    hass: HomeAssistant,
    entity_id: str,
    pack: str,
    item: dict,
    exposed: Callable[[str], bool],
) -> IntentStepResult | None:
    state = hass.states.get(entity_id) if entity_id.startswith("media_player.") else None
    if state is None or not exposed(entity_id) or media_missing(state):
        return None
    if str(getattr(state, "state", "")).lower() not in {"idle", "off", "on", "standby"}:
        return None
    if music_assistant_player(hass, entity_id):
        query = "Musik" if pack == "de" or pack.startswith("de-") else "music"
        return await run_mass(
            hass,
            "MassPlayMedia",
            {"entity_id": {"value": entity_id}, "media_id": {"value": query}},
            pack,
            {**item, "name": "MassPlayMedia", "slots": [*(item.get("slots") or []), {"name": "media_id", "value": query}]},
            exposed,
        )
    device_id = alexa_device_id(hass, entity_id)
    if not device_id:
        return None
    command = "spiel Musik" if pack == "de" or pack.startswith("de-") else "play music"
    try:
        await hass.services.async_call(
            "alexa_devices",
            "send_text_command",
            {"device_id": device_id, "text_command": command},
            blocking=True,
        )
    except Exception as err:  # noqa: BLE001 — Alexa is a service boundary
        _LOGGER.debug("Alexa-Wiedergabe für %s fehlgeschlagen: %s", entity_id, err)
        return _fail(str(err) or "alexa_play_failed")
    spoken = {**item, "name": "HassMediaSearchAndPlay", "slots": [*(item.get("slots") or []), {"name": "name", "value": state.name}]}
    return _ok(from_handled(None, pack, spoken))


def tv_request(text: str) -> bool:
    folded = f" {(text or '').casefold()} "
    return "fernseher" in folded or "television" in folded or " tv " in folded or folded.startswith(" tv")


def tv_named(entity_id: str, state: Any) -> bool:
    attrs = getattr(state, "attributes", None) or {} if state is not None else {}
    name = str(attrs.get("friendly_name") or "") if isinstance(attrs, dict) else ""
    name = name or str(getattr(state, "name", "") or "")
    blob = f"{entity_id} {name}".casefold()
    return "tv" in blob or "fernseher" in blob or "television" in blob


def registry_entry(hass: HomeAssistant, entity_id: str) -> Any:
    try:
        return entity_registry.async_get(hass).async_get(entity_id)
    except Exception:  # noqa: BLE001 — registry is a system boundary
        return None


def alexa_device_id(hass: HomeAssistant, entity_id: str) -> str:
    entry = registry_entry(hass, entity_id)
    if entry is None or getattr(entry, "platform", None) != "alexa_devices":
        return ""
    return str(getattr(entry, "device_id", "") or "")


def music_assistant_player(hass: HomeAssistant, entity_id: str) -> bool:
    if not entity_id.startswith("media_player."):
        return False
    state = hass.states.get(entity_id)
    attrs = getattr(state, "attributes", None) or {}
    if isinstance(attrs, dict) and (
        attrs.get("mass_player_type") or "music assistant" in str(attrs.get("source") or "").lower()
    ):
        return True
    entry = registry_entry(hass, entity_id)
    return bool(entry is not None and getattr(entry, "platform", None) == "music_assistant")


def favorite_button(hass: HomeAssistant, player: str) -> str:
    base = player.split(".", 1)[-1].removesuffix("_2")
    for state in hass.states.async_all("button"):
        entity_id = str(state.entity_id)
        label = f"{entity_id} {getattr(state, 'name', '')}".lower()
        if ("favorisieren" in label or "favorite" in label) and (not base or base in entity_id):
            return entity_id
    return ""
