"""Build the post-execute HA state snapshot. Engine interpolates; Python does not invent values."""

from __future__ import annotations

from typing import Any

SNAPSHOT_SCHEMA = "1"
MAX_ENTITIES = 32
MAX_EVENTS = 16
MAX_QUEUE = 8
MAX_ATTR = 256
ALLOWED_ATTRS = (
    "current_temperature",
    "temperature",
    "temperature_unit",
    "unit_of_measurement",
    "hvac_action",
    "hvac_mode",
    "volume_level",
    "is_volume_muted",
    "media_title",
    "media_artist",
    "media_album_name",
)


def build_snapshot(
    *,
    language: str,
    personality: str,
    now: str,
    intent: dict[str, Any],
    outcome: str,
    entities: list[dict[str, Any]] | None = None,
    calendar_events: list[dict[str, Any]] | None = None,
    media_queue: list[dict[str, Any]] | None = None,
    unit_system: str = "metric",
) -> dict[str, Any]:
    return {
        "schema_version": SNAPSHOT_SCHEMA,
        "language": language,
        "personality": personality or "default",
        "unit_system": "imperial" if unit_system == "imperial" else "metric",
        "now": now,
        "intent": {
            "name": str(intent.get("name") or ""),
            "slots": [
                {"name": str(slot.get("name") or ""), "value": str(slot.get("value") or "")}
                for slot in (intent.get("slots") or [])
                if isinstance(slot, dict) and slot.get("name")
            ][:16],
        },
        "outcome": outcome,
        "entities": [_entity(row) for row in (entities or [])[:MAX_ENTITIES]],
        "calendar_events": [
            {"summary": str(row.get("summary") or "")[:128], "start": str(row.get("start") or "")[:64]}
            for row in (calendar_events or [])[:MAX_EVENTS]
            if isinstance(row, dict)
        ],
        "media_queue": [
            {"title": str(row.get("title") or "")[:128]}
            for row in (media_queue or [])[:MAX_QUEUE]
            if isinstance(row, dict)
        ],
    }


def _entity(row: dict[str, Any]) -> dict[str, Any]:
    raw_attrs = row.get("attributes") if isinstance(row.get("attributes"), dict) else {}
    attrs: dict[str, Any] = {}
    for key in ALLOWED_ATTRS:
        if key not in raw_attrs:
            continue
        value = raw_attrs[key]
        if isinstance(value, str):
            attrs[key] = value[:MAX_ATTR]
        elif isinstance(value, (int, float, bool)) or value is None:
            attrs[key] = value
    return {
        "entity_id": str(row.get("entity_id") or "")[:128],
        "name": str(row.get("name") or "")[:128],
        "domain": str(row.get("domain") or "")[:32],
        "state": str(row.get("state") or "")[:64],
        "area": _opt(row.get("area")),
        "area_name": _opt(row.get("area_name")),
        "device_class": _opt(row.get("device_class")),
        "attributes": attrs,
    }


def _opt(value: Any) -> str | None:
    if value is None:
        return None
    text = str(value).strip()
    return text[:128] if text else None


def entities_from_handled(
    handled: Any,
    item: dict[str, Any] | None = None,
    hass: Any = None,
) -> list[dict[str, Any]]:
    states = list(getattr(handled, "matched_states", None) or [])
    if not states:
        states = list(getattr(handled, "unmatched_states", None) or [])
    rows = [_state_row(state) for state in states]
    rows = [row for row in rows if row]
    if not rows:
        slots = {
            str(slot.get("name") or ""): str(slot.get("value") or "")
            for slot in (item or {}).get("slots") or []
            if isinstance(slot, dict)
        }
        entity_id = slots.get("entity_id") or ""
        if entity_id:
            domain = entity_id.split(".", 1)[0]
            rows = [
                {
                    "entity_id": entity_id,
                    "name": slots.get("name") or "",
                    "domain": domain,
                    "state": "",
                    "area": slots.get("area") or None,
                    "area_name": slots.get("area_name") or None,
                    "attributes": {},
                }
            ]
    return hydrate_from_hass(hass, rows)


def hydrate_from_hass(hass: Any, rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    states = getattr(hass, "states", None) if hass is not None else None
    getter = getattr(states, "get", None)
    if not callable(getter):
        return rows
    out: list[dict[str, Any]] = []
    for row in rows:
        if row.get("state") and row.get("name") and row.get("attributes"):
            out.append(row)
            continue
        entity_id = str(row.get("entity_id") or "")
        state = getter(entity_id) if entity_id else None
        live = _state_row(state) if state is not None else None
        out.append(live or row)
    return out


def entity_from_state(state: Any) -> dict[str, Any] | None:
    return _state_row(state)


def _state_row(state: Any) -> dict[str, Any] | None:
    entity_id = str(getattr(state, "entity_id", "") or "")
    if not entity_id:
        return None
    attrs = getattr(state, "attributes", None) or {}
    if not isinstance(attrs, dict):
        attrs = {}
    name = str(attrs.get("friendly_name") or getattr(state, "name", "") or "")
    domain = entity_id.split(".", 1)[0]
    return {
        "entity_id": entity_id,
        "name": name,
        "domain": domain,
        "state": str(getattr(state, "state", "") or ""),
        "attributes": attrs,
    }
