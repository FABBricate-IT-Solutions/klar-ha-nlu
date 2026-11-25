"""Typed (domain, intent) → Home Assistant service mapping.

Named-entity steps call the service by entity_id. A missing cell means Klar
refuses the step instead of inventing a service Home Assistant does not have.
"""

from __future__ import annotations

from typing import Any

# None = no agreed mapping; Klar must not invent a service for that cell.
DOMAIN_TURN: dict[str, dict[str, str | None]] = {
    "light": {"HassTurnOn": "turn_on", "HassTurnOff": "turn_off", "HassToggle": "toggle"},
    "switch": {"HassTurnOn": "turn_on", "HassTurnOff": "turn_off", "HassToggle": "toggle"},
    "fan": {"HassTurnOn": "turn_on", "HassTurnOff": "turn_off", "HassToggle": "toggle"},
    "climate": {"HassTurnOn": "turn_on", "HassTurnOff": "turn_off", "HassToggle": "toggle"},
    "media_player": {"HassTurnOn": "turn_on", "HassTurnOff": "turn_off", "HassToggle": "toggle"},
    "cover": {"HassTurnOn": "open_cover", "HassTurnOff": "close_cover", "HassToggle": "toggle"},
    "lock": {"HassTurnOn": "lock", "HassTurnOff": "unlock", "HassToggle": None},
    "valve": {"HassTurnOn": "open_valve", "HassTurnOff": "close_valve", "HassToggle": "toggle"},
    "lawn_mower": {"HassTurnOn": "start_mowing", "HassTurnOff": "dock", "HassToggle": None},
    "vacuum": {"HassTurnOn": "start", "HassTurnOff": "return_to_base", "HassToggle": None},
    "button": {"HassTurnOn": "press", "HassTurnOff": None, "HassToggle": None},
    "scene": {"HassTurnOn": "turn_on", "HassTurnOff": None, "HassToggle": None},
}

# Intents that always go through the entity-id service path when an entity is set.
ENTITY_INTENTS = {
    "HassTurnOn",
    "HassTurnOff",
    "HassToggle",
    "HassLightSet",
    "HassClimateSetTemperature",
    "HassFanSetSpeed",
    "HassSetPosition",
    "HassVacuumStart",
    "HassVacuumReturnToBase",
}


def turn_service(domain: str, name: str) -> str | None:
    """Service for a generic on/off/toggle intent, or None when unsupported."""
    row = DOMAIN_TURN.get(domain)
    if row is None:
        return None
    if name not in row:
        return None
    return row[name]


def build_service_call(
    domain: str,
    name: str,
    entity_id: str,
    slots: dict[str, Any],
    state: Any,
) -> tuple[str, dict[str, Any]] | None:
    """Return ``(service, data)`` for a named-entity intent, or None to refuse."""
    data: dict[str, Any] = {"entity_id": entity_id}
    if name == "HassLightSet" and domain == "light":
        data.update(light_turn_on(slots))
        return "turn_on", data
    if name == "HassClimateSetTemperature" and domain == "climate":
        extra = climate_set_data(slots, state)
        if extra is None:
            return None
        data.update(extra)
        return "set_temperature", data
    if name == "HassFanSetSpeed" and domain == "fan":
        pct = _slot_int(slots.get("percentage"))
        if pct is None:
            return None
        data["percentage"] = max(0, min(100, pct))
        return "turn_on", data
    if name == "HassSetPosition" and domain == "cover":
        position = _slot_int(slots.get("position"))
        if position is None:
            return None
        data["position"] = max(0, min(100, position))
        return "set_cover_position", data
    if name == "HassVacuumStart" and domain == "vacuum":
        return "start", data
    if name == "HassVacuumReturnToBase" and domain == "vacuum":
        return "return_to_base", data
    if name in {"HassTurnOn", "HassTurnOff", "HassToggle"}:
        service = turn_service(domain, name)
        if not service:
            return None
        return service, data
    return None


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


def climate_set_data(slots: dict[str, Any], state: Any = None) -> dict[str, Any] | None:
    """Temperature only — never force ``hvac_mode: heat`` (H-11).

    Relative ``temperature_step`` adds to the entity's current target (or current
    temperature) so "one degree warmer" is not stored as absolute 1 °C.
    """
    step_raw = slots.get("temperature_step", {}).get("value")
    if step_raw is not None and str(step_raw) != "":
        try:
            step = float(step_raw)
        except (TypeError, ValueError):
            return None
        attrs = getattr(state, "attributes", None) or {}
        base = None
        if isinstance(attrs, dict):
            for key in ("temperature", "current_temperature"):
                raw = attrs.get(key)
                if raw is None:
                    continue
                try:
                    base = float(raw)
                    break
                except (TypeError, ValueError):
                    continue
        if base is None:
            return None
        return {"temperature": base + step}
    try:
        return {"temperature": float(slots.get("temperature", {}).get("value"))}
    except (TypeError, ValueError):
        return None


def _slot_int(raw: Any) -> int | None:
    value = raw.get("value") if isinstance(raw, dict) else raw
    if value is None or value == "":
        return None
    try:
        return int(float(str(value)))
    except (TypeError, ValueError):
        return None
