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
) -> dict[str, Any]:
    return {
        "schema_version": SNAPSHOT_SCHEMA,
        "language": language,
        "personality": personality or "default",
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
