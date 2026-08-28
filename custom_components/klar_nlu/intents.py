"""Home Assistant intent slot shaping for Klar."""

from __future__ import annotations

from typing import Any

from homeassistant.core import HomeAssistant
from homeassistant.helpers import area_registry

TIMER_INTENTS = {
    "HassStartTimer",
    "HassIncreaseTimer",
    "HassDecreaseTimer",
    "HassCancelTimer",
    "HassPauseTimer",
}

ENTITY_SERVICES = {
    "HassTurnOn": "turn_on",
    "HassTurnOff": "turn_off",
    "HassToggle": "toggle",
}

LIST_INTENTS = {
    "HassListAddItem",
    "HassListCompleteItem",
    "HassShoppingListAddItem",
    "HassShoppingListCompleteItem",
}

MASS_INTENTS = {
    "MassPlayMedia",
    "MassTransferQueue",
    "MassFavorite",
    "MassGetQueue",
}

ALLOWED_INTENTS = TIMER_INTENTS | set(ENTITY_SERVICES) | LIST_INTENTS | {
    "HassLightSet",
    "HassClimateSetTemperature",
    "HassClimateGetTemperature",
    "HassGetState",
    "HassMediaPause",
    "HassMediaUnpause",
    "HassMediaNext",
    "HassMediaPrevious",
    "HassMediaPlayerMute",
    "HassMediaPlayerUnmute",
    "HassSetVolume",
    "HassSetVolumeRelative",
    "HassMediaSearchAndPlay",
    *MASS_INTENTS,
    "HassFanSetSpeed",
    "HassVacuumStart",
    "HassVacuumReturnToBase",
    "HassSetPosition",
    "KlarGetCalendarEvents",
    "KlarCreateCalendarEvent",
    "KlarDeleteCalendarEvent",
    "KlarMoveCalendarEvent",
    "KlarNoMusicPlayer",
}


def timer_slots(slots: dict[str, Any]) -> dict[str, Any]:
    if "duration" in slots:
        duration = slots.pop("duration")
        if "minutes" not in slots and "hours" not in slots and "seconds" not in slots:
            slots["minutes"] = duration
    slots.pop("entity_id", None)
    slots.pop("domain", None)
    return slots


def list_slots(
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


def home_intents(intents: list[Any], registered: set[str] | None = None) -> list[dict[str, Any]]:
    allowed = ALLOWED_INTENTS | (registered or set())
    out: list[dict[str, Any]] = []
    for item in intents:
        if not isinstance(item, dict) or item.get("name") not in allowed:
            continue
        if item["name"] == "HassGetState" and not get_state_has_target(item):
            continue
        out.append(item)
    return out


def get_state_has_target(item: dict[str, Any]) -> bool:
    return any(
        isinstance(slot, dict)
        and slot.get("name") in {"area", "floor", "entity_id", "name", "device_class", "domain"}
        for slot in (item.get("slots") or [])
    )


def _fold_latin(text: str) -> str:
    return (
        text.replace("ä", "ae")
        .replace("ö", "oe")
        .replace("ü", "ue")
        .replace("Ä", "ae")
        .replace("Ö", "oe")
        .replace("Ü", "ue")
        .replace("ß", "ss")
        .casefold()
    )


def _fold_marks(text: str) -> str:
    return (
        text.replace("ä", "a")
        .replace("ö", "o")
        .replace("ü", "u")
        .replace("Ä", "a")
        .replace("Ö", "o")
        .replace("Ü", "u")
        .replace("ß", "ss")
        .casefold()
    )


def _umlaut_eq(left: str, right: str) -> bool:
    if left == right:
        return True
    i = j = 0
    while i < len(left) and j < len(right):
        if left[i] != right[j]:
            return False
        if left[i] in {"a", "o", "u"}:
            left_e = i + 1 < len(left) and left[i + 1] == "e"
            right_e = j + 1 < len(right) and right[j + 1] == "e"
            if left_e != right_e:
                if left_e:
                    i += 1
                else:
                    j += 1
        i += 1
        j += 1
    return i == len(left) and j == len(right)


def _area_hit(label: str, key: str) -> bool:
    text = str(label)
    want = key.casefold()
    return (
        text.casefold() == want
        or _fold_latin(text) == _fold_latin(key)
        or _fold_marks(text) == _fold_marks(key)
        or _umlaut_eq(_fold_latin(text), _fold_latin(key))
    )


def resolve_area(hass: HomeAssistant, key: str) -> Any | None:
    if not key:
        return None
    areas = area_registry.async_get(hass)
    found = areas.async_get_area(key)
    if found is not None:
        return found
    for item in areas.async_list_areas():
        labels = [item.id, item.name, *list(getattr(item, "aliases", None) or [])]
        if any(_area_hit(label, key) for label in labels):
            return item
    return None


def area_label(hass: HomeAssistant, area_id: str) -> str:
    if not area_id:
        return ""
    area = resolve_area(hass, area_id)
    return str(getattr(area, "name", None) or area_id)


def registered_intent_names(hass: HomeAssistant | None) -> set[str]:
    if hass is None:
        return set()
    try:
        from homeassistant.helpers import intent as ha_intent

        manager = ha_intent.async_get(hass)
    except Exception:  # noqa: BLE001 — intent manager is a system boundary
        return set()
    handlers = getattr(manager, "handlers", None) or getattr(manager, "_handlers", {}) or {}
    return {name for name in handlers if isinstance(name, str) and name not in ALLOWED_INTENTS}


def item_slots(item: dict) -> dict[str, Any]:
    return {
        str(raw["name"]): {"value": raw.get("value")}
        for raw in item.get("slots") or []
        if isinstance(raw, dict) and raw.get("name")
    }
