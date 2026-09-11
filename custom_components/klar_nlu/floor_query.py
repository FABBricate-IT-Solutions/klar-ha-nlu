"""Speak GetState for a floor or area instead of one mixed HA result."""

from __future__ import annotations

from collections.abc import Callable
from typing import Any

from homeassistant.helpers import area_registry, device_registry, entity_registry, floor_registry

from .intents import _area_hit, resolve_area
from .speech_status import _infra_state, empty_status_speech, rooms_status_speech

_STATUS_DOMAINS = (
    "light",
    "switch",
    "binary_sensor",
    "sensor",
    "climate",
    "vacuum",
    "cover",
    "lock",
    "fan",
    "media_player",
)


def place_status_rooms(
    hass: Any,
    slots: dict[str, Any],
    exposed: Callable[[str], bool],
) -> list[tuple[str, list[Any]]] | None:
    """Collect floor/area states. None = not a place query."""
    if str(slots.get("device_class", {}).get("value") or "") == "temperature":
        return None
    if slots.get("floor"):
        return floor_status_rooms(
            hass,
            str(slots.get("floor", {}).get("value") or ""),
            str(slots.get("domain", {}).get("value") or ""),
            exposed,
        )
    if slots.get("area"):
        return area_status_rooms(
            hass,
            str(slots.get("area", {}).get("value") or ""),
            str(slots.get("domain", {}).get("value") or ""),
            exposed,
        )
    return None


def place_get_state(
    hass: Any,
    slots: dict[str, Any],
    pack: str,
    exposed: Callable[[str], bool],
    unit_system: str = "metric",
) -> str:
    rooms = place_status_rooms(hass, slots, exposed)
    if rooms is None:
        return ""
    return rooms_status_speech(rooms, pack, unit_system) or empty_status_speech(pack)


def floor_get_state(
    hass: Any,
    slots: dict[str, Any],
    pack: str,
    exposed: Callable[[str], bool],
    unit_system: str = "metric",
) -> str:
    floor_key = str(slots.get("floor", {}).get("value") or "")
    domain = str(slots.get("domain", {}).get("value") or "")
    rooms = floor_status_rooms(hass, floor_key, domain, exposed)
    return rooms_status_speech(rooms, pack, unit_system)


def area_get_state(
    hass: Any,
    slots: dict[str, Any],
    pack: str,
    exposed: Callable[[str], bool],
    unit_system: str = "metric",
) -> str:
    area_key = str(slots.get("area", {}).get("value") or "")
    domain = str(slots.get("domain", {}).get("value") or "")
    rooms = area_status_rooms(hass, area_key, domain, exposed)
    return rooms_status_speech(rooms, pack, unit_system)


def floor_status_rooms(
    hass: Any,
    floor_key: str,
    domain: str,
    exposed: Callable[[str], bool],
) -> list[tuple[str, list[Any]]]:
    floor = resolve_floor(hass, floor_key)
    if floor is None:
        return []
    floor_id = str(getattr(floor, "floor_id", None) or getattr(floor, "id", "") or "")
    if not floor_id:
        return []
    wanted = _wanted_domains(domain)
    rooms: list[tuple[str, list[Any]]] = []
    for area in areas_on_floor(hass, floor_id):
        area_id = str(getattr(area, "id", None) or getattr(area, "area_id", "") or "")
        states = states_in_area(hass, area_id, wanted, exposed)
        if states:
            rooms.append((str(getattr(area, "name", None) or area_id), states))
    return rooms


def area_status_rooms(
    hass: Any,
    area_key: str,
    domain: str,
    exposed: Callable[[str], bool],
) -> list[tuple[str, list[Any]]]:
    area = resolve_area(hass, area_key)
    if area is None:
        return []
    area_id = str(getattr(area, "id", None) or getattr(area, "area_id", "") or "")
    if not area_id:
        return []
    states = states_in_area(hass, area_id, _wanted_domains(domain), exposed)
    if not states:
        return []
    return [(str(getattr(area, "name", None) or area_id), states)]


def states_in_area(
    hass: Any,
    area_id: str,
    wanted: set[str],
    exposed: Callable[[str], bool],
) -> list[Any]:
    return [
        state
        for state in hass.states.async_all()
        if str(state.entity_id).split(".", 1)[0] in wanted
        and exposed(str(state.entity_id))
        and entity_area_id(hass, str(state.entity_id)) == area_id
    ]


def resolve_floor(hass: Any, key: str) -> Any | None:
    if not key:
        return None
    floors = floor_registry.async_get(hass)
    getter = getattr(floors, "async_get_floor", None)
    if callable(getter):
        found = getter(key)
        if found is not None:
            return found
    for item in _floor_items(floors):
        labels = [
            getattr(item, "floor_id", None) or getattr(item, "id", ""),
            getattr(item, "name", ""),
            *list(getattr(item, "aliases", None) or []),
        ]
        if any(_area_hit(str(label), key) for label in labels if label):
            return item
    return None


def areas_on_floor(hass: Any, floor_id: str) -> list[Any]:
    areas = [
        area
        for area in area_registry.async_get(hass).async_list_areas()
        if getattr(area, "floor_id", None) == floor_id
    ]
    areas.sort(key=lambda area: str(getattr(area, "name", None) or getattr(area, "id", "")).casefold())
    return areas


def entity_area_id(hass: Any, entity_id: str) -> str:
    entry = entity_registry.async_get(hass).async_get(entity_id)
    if entry is None:
        return ""
    if getattr(entry, "area_id", None):
        return str(entry.area_id)
    device_id = getattr(entry, "device_id", None)
    if not device_id:
        return ""
    device = device_registry.async_get(hass).async_get(device_id)
    return str(getattr(device, "area_id", None) or "") if device is not None else ""


def _wanted_domains(domain: str) -> set[str]:
    return {domain} if domain in _STATUS_DOMAINS else set(_STATUS_DOMAINS)


def floor_temperature_rooms(
    hass: Any,
    floor_key: str,
    exposed: Callable[[str], bool],
) -> list[tuple[str, list[Any]]]:
    """One temperature reading per area on the floor. Sensor wins over climate."""
    rooms: list[tuple[str, list[Any]]] = []
    for area_name, states in floor_status_rooms(hass, floor_key, "", exposed):
        picked = _pick_temp_state(states)
        if picked is not None:
            rooms.append((area_name, [picked]))
    return rooms


def area_temperature_context(hass: Any, exposed: Callable[[str], bool]) -> list[dict[str, Any]]:
    """Jinja `rooms` for house-rule templates: area_id, name, floor, temp."""
    rows: list[dict[str, Any]] = []
    for area in area_registry.async_get(hass).async_list_areas():
        area_id = str(getattr(area, "id", None) or getattr(area, "area_id", "") or "")
        if not area_id:
            continue
        picked = _pick_temp_state(states_in_area(hass, area_id, {"sensor", "climate"}, exposed))
        rows.append(
            {
                "area_id": area_id,
                "name": str(getattr(area, "name", None) or area_id),
                "floor": str(getattr(area, "floor_id", None) or ""),
                "temp": _temp_number(picked) if picked is not None else None,
            }
        )
    rows.sort(key=lambda row: str(row.get("name") or "").casefold())
    return rows


def _pick_temp_state(states: list[Any]) -> Any | None:
    usable = [state for state in states if _usable_temp(state)]
    sensors = [state for state in usable if _temp_class(state) == "temperature"]
    if sensors:
        return sensors[0]
    climates = [state for state in usable if str(getattr(state, "entity_id", "")).startswith("climate.")]
    return climates[0] if climates else None


def _usable_temp(state: Any) -> bool:
    if _infra_state(state):
        return False
    raw = str(getattr(state, "state", "") or "").lower()
    if raw in {"unavailable", "unknown"}:
        return False
    return _temp_number(state) is not None


def _temp_class(state: Any) -> str:
    attrs = getattr(state, "attributes", None) or {}
    return str(attrs.get("device_class") or "").lower() if isinstance(attrs, dict) else ""


def _temp_number(state: Any) -> str | None:
    attrs = getattr(state, "attributes", None) or {}
    raw = attrs.get("current_temperature") if isinstance(attrs, dict) else None
    if raw in (None, ""):
        raw = getattr(state, "state", None)
    try:
        value = float(raw)
    except (TypeError, ValueError):
        return None
    if value == int(value):
        return str(int(value))
    text = f"{value:.1f}".rstrip("0").rstrip(".")
    return text


def _floor_items(registry: Any) -> list[Any]:
    floors = getattr(registry, "floors", None)
    if floors is None:
        return []
    values = getattr(floors, "values", None)
    if callable(values):
        return list(values())
    if isinstance(floors, dict):
        return list(floors.values())
    return list(floors)
