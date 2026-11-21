"""IANA timezone names synced from Home Assistant onto the engine."""

from __future__ import annotations

_MAX_LEN = 64


def time_zone_name(zone: object) -> str:
    text = str(zone or "").strip()
    if not text or len(text) > _MAX_LEN:
        return ""
    if any(not (ch.isascii() and (ch.isalnum() or ch in "/_+-")) for ch in text):
        return ""
    return text


def time_zone_needs_sync(current: object, zone: str) -> bool:
    if not zone:
        return False
    stored = current.get("time_zone") if isinstance(current, dict) else None
    return str(stored or "") != zone
