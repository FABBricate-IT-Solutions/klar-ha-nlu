"""Timer helpers when conversation timers have no satellite device."""

from __future__ import annotations

import logging
from typing import Any

from homeassistant.core import HomeAssistant

from .dispatch_result import IntentStepResult, fail, ok
from .intents import timer_slots
from .speech_render import spoken_after_execute

_LOGGER = logging.getLogger(__name__)

_DURATION_KEYS = ("hours", "minutes", "seconds")
_SERVICES = {
    "HassStartTimer": "start",
    "HassIncreaseTimer": "change",
    "HassDecreaseTimer": "change",
    "HassCancelTimer": "cancel",
    "HassPauseTimer": "pause",
}


def timer_has_name(slots: dict[str, Any]) -> bool:
    for key in ("entity_id", "timer_name", "name"):
        text = _slot_text(slots.get(key))
        if text and "abstract" not in text:
            return True
    return False


def duration_text(slots: dict[str, Any]) -> str | None:
    hours = _slot_int(slots.get("hours")) or 0
    minutes = _slot_int(slots.get("minutes")) or 0
    seconds = _slot_int(slots.get("seconds")) or 0
    if hours == minutes == seconds == 0:
        return None
    return f"{hours:02d}:{minutes:02d}:{seconds:02d}"


def prepare_timer_slots(name: str, slots: dict[str, Any]) -> tuple[dict[str, Any], str | None]:
    named = timer_has_name(slots)
    shaped = timer_slots(slots)
    if name == "HassStartTimer" and not any(key in shaped for key in _DURATION_KEYS) and not named:
        return shaped, "missing_timer_duration"
    if name in {"HassIncreaseTimer", "HassDecreaseTimer"} and duration_text(shaped) is None:
        return shaped, "missing_timer_duration"
    return shaped, None


async def run_timer_helper(
    hass: HomeAssistant,
    name: str,
    slots: dict[str, Any],
    pack: str,
    item: dict,
) -> IntentStepResult:
    service = _SERVICES.get(name)
    if service is None:
        return fail("timer_unsupported")
    entity_id = pick_timer_helper(hass, slots, name)
    if not entity_id:
        return fail("timer_not_found")
    data: dict[str, Any] = {"entity_id": entity_id}
    if name in {"HassStartTimer", "HassIncreaseTimer", "HassDecreaseTimer"}:
        duration = duration_text(slots)
        if duration is None:
            return fail("missing_timer_duration")
        data["duration"] = f"-{duration}" if name == "HassDecreaseTimer" else duration
    try:
        await hass.services.async_call("timer", service, data, blocking=True)
    except Exception as err:  # noqa: BLE001 — helper services are a boundary
        _LOGGER.debug("Timer-Helfer %s nicht ausgeführt: %s", entity_id, err)
        return fail(str(err) or "timer_helper_failed")
    spoken = await spoken_after_execute(hass, pack, "default", {**item, "name": name})
    return ok(spoken) if spoken else fail("speech_missing")


def pick_timer_helper(hass: HomeAssistant, slots: dict[str, Any], name: str) -> str | None:
    timers = list(hass.states.async_all("timer"))
    if not timers:
        return None
    needle = _slot_text(slots.get("name")) or _slot_text(slots.get("timer_name"))
    if needle:
        matches = [state for state in timers if _name_hit(state, needle)]
        if len(matches) == 1:
            return matches[0].entity_id
        if len(matches) > 1:
            return None
    if len(timers) == 1:
        return timers[0].entity_id
    if name == "HassStartTimer":
        return None
    active = [state for state in timers if state.state == "active"]
    if len(active) == 1:
        return active[0].entity_id
    paused = [state for state in timers if state.state == "paused"]
    if name != "HassPauseTimer" and len(paused) == 1:
        return paused[0].entity_id
    return None


def _slot_text(raw: Any) -> str:
    value = raw.get("value") if isinstance(raw, dict) else raw
    return str(value or "").strip()


def _slot_int(raw: Any) -> int | None:
    text = _slot_text(raw)
    if not text:
        return None
    try:
        return int(float(text))
    except ValueError:
        return None


def _name_hit(state: Any, needle: str) -> bool:
    folded = needle.casefold()
    name = str(getattr(state, "name", "") or "").casefold()
    entity_id = str(getattr(state, "entity_id", "") or "").casefold()
    tail = entity_id.rsplit(".", 1)[-1].replace("_", " ")
    return folded in {name, entity_id, tail} or folded in name or folded in tail
