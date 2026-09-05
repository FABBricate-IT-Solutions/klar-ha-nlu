"""Run a parsed Klar intent through Home Assistant."""

from __future__ import annotations

import logging
from collections.abc import Callable
from types import SimpleNamespace
from typing import Any

from homeassistant.components.conversation import ConversationInput
from homeassistant.core import HomeAssistant
from homeassistant.helpers import entity_registry, intent

from .calendar_ha import CALENDAR_INTENTS, handle_calendar_intent
from .dispatch_media import (
    MEDIA_SERVICES,
    media_missing,
    music_assistant_player,
    run_mass,
    start_idle_music,
)
from .dispatch_result import IntentStepResult, fail as _fail, ok as _ok
from .floor_query import place_get_state
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
from .lang_select import speak_tag
from .speech_render import spoken_after_execute
from .speech_snapshot import entity_from_state

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
        if state is None or not exposed(entity_id) or media_missing(state):
            return _fail("media_status_unavailable")
        extra = [row] if (row := entity_from_state(state)) else None
        spoken = await spoken_after_execute(
            hass,
            pack,
            "default",
            {**item, "name": name},
            SimpleNamespace(matched_states=[state], response_type="query_answer"),
            extra_entities=extra,
        )
        return _ok(spoken) if spoken else _fail("media_status_unavailable")
    if name == "HassGetState" and not entity_id:
        spoken = place_get_state(hass, slots, pack, exposed)
        if spoken:
            return _ok(spoken)
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
    spoken = await spoken_after_execute(
        hass,
        pack,
        "default",
        {**item, "name": "HassClimateGetTemperature"},
        SimpleNamespace(
            matched_states=states,
            unmatched_states=[],
            success_results=[],
            response_type="query_answer",
        ),
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


def intent_query_text(user_input: ConversationInput, name: str, slots: dict[str, Any]) -> str:
    """HA weather GetState reads the utterance for 'tomorrow'. Keep that only for weather."""
    entity_id = str(slots.get("entity_id", {}).get("value") or "")
    domain = str(slots.get("domain", {}).get("value") or "")
    if name == "HassGetState" and (domain == "weather" or entity_id.startswith("weather.")):
        return user_input.text
    return name


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
            intent_query_text(user_input, name, slots),
            user_input.context,
            speak_tag(pack),
            assistant=assistant,
        )
    except Exception as err:  # noqa: BLE001 — HA intent system is a boundary
        _LOGGER.debug("Intent %s nicht ausgeführt: %s", name, err)
        return _fail(str(err) or name)
    return _ok(await spoken_after_execute(hass, pack, "default", {**item, "name": name}, handled))


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
    extra = [row] if (row := entity_from_state(state)) else None
    return _ok(await spoken_after_execute(hass, pack, "default", spoken, extra_entities=extra))


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
