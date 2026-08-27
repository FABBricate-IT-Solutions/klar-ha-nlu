"""Last listed calendar events, keyed by Assist conversation_id."""

from __future__ import annotations

import time
from typing import Any

_TTL_SEC = 30 * 60
_MAX_EVENTS = 8
_STORE: dict[str, tuple[float, list[dict[str, str]]]] = {}


def remember(conversation_id: str | None, events: list[dict[str, str]]) -> None:
    if not conversation_id:
        return
    _STORE[conversation_id] = (time.monotonic(), events[:_MAX_EVENTS])
    stale = [key for key, (stamp, _) in _STORE.items() if time.monotonic() - stamp > _TTL_SEC]
    for key in stale:
        _STORE.pop(key, None)


def last_events(conversation_id: str | None) -> list[dict[str, str]]:
    if not conversation_id:
        return []
    stamp, events = _STORE.get(conversation_id, (0.0, []))
    if time.monotonic() - stamp > _TTL_SEC:
        _STORE.pop(conversation_id, None)
        return []
    return list(events)


def as_record(entity_id: str, event: dict[str, Any]) -> dict[str, str]:
    start = event.get("start") or event.get("start_date_time") or event.get("start_date") or ""
    end = event.get("end") or event.get("end_date_time") or event.get("end_date") or ""
    if isinstance(start, dict):
        start = start.get("dateTime") or start.get("date") or ""
    if isinstance(end, dict):
        end = end.get("dateTime") or end.get("date") or ""
    return {
        "uid": str(event.get("uid") or event.get("id") or ""),
        "recurrence_id": str(event.get("recurrence_id") or ""),
        "summary": str(event.get("summary") or event.get("title") or "").strip(),
        "start": str(start),
        "end": str(end),
        "entity_id": entity_id,
    }


def match_events(events: list[dict[str, str]], summary: str) -> list[dict[str, str]]:
    needle = summary.casefold().strip()
    if not needle:
        return list(events)
    exact = [item for item in events if item.get("summary", "").casefold() == needle]
    if exact:
        return exact
    return [item for item in events if needle in item.get("summary", "").casefold()]
