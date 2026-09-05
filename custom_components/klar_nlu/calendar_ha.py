"""Native calendar list/create/delete/move and pack-local speech."""

from __future__ import annotations

from collections.abc import Callable
from datetime import datetime, timedelta
from typing import Any

from homeassistant.core import HomeAssistant

try:
    from . import calendar_entity, calendar_say, calendar_session
except ImportError:
    import calendar_entity  # type: ignore[no-redef]
    import calendar_say  # type: ignore[no-redef]
    import calendar_session  # type: ignore[no-redef]

try:
    from .speech_render import try_engine_speech
except ImportError:
    try:
        from speech_render import try_engine_speech
    except ImportError:
        async def try_engine_speech(*_args: Any, **_kwargs: Any) -> str | None:
            return None

CALENDAR_INTENTS = {
    "KlarGetCalendarEvents",
    "KlarCreateCalendarEvent",
    "KlarDeleteCalendarEvent",
    "KlarMoveCalendarEvent",
    "KlarNoMusicPlayer",
}


def _slot(item: dict, name: str) -> str:
    for raw in item.get("slots") or []:
        if isinstance(raw, dict) and raw.get("name") == name:
            return str(raw.get("value") or "")
    return ""


def _when_bounds(item: dict, hass: HomeAssistant) -> tuple[datetime, datetime, bool]:
    now = datetime.now(calendar_entity.timezone(hass)).replace(second=0, microsecond=0)
    day = _slot(item, "day")
    hour_raw = _slot(item, "hour")
    in_days = _slot(item, "in_days")
    start = now.replace(hour=0, minute=0)
    if day == "tomorrow":
        start = start + timedelta(days=1)
    elif in_days.isdigit():
        start = start + timedelta(days=int(in_days))
    all_day = not hour_raw
    if hour_raw.isdigit():
        start = start.replace(hour=max(0, min(23, int(hour_raw))), minute=0)
        return start, start + timedelta(hours=1), False
    return start, start + timedelta(days=1), all_day


def _calendars(hass: HomeAssistant, item: dict, exposed: Callable[[str], bool]) -> list[str]:
    wanted = _slot(item, "entity_id")
    ids = [wanted] if wanted.startswith("calendar.") else []
    if not ids:
        ids = [state.entity_id for state in hass.states.async_all("calendar") if exposed(state.entity_id)]
    return [entity_id for entity_id in ids if exposed(entity_id) and hass.states.get(entity_id) is not None]


def _pick(conversation_id: str | None, summary: str, records: list[dict[str, str]]) -> list[dict[str, str]]:
    pooled = calendar_session.last_events(conversation_id) + records
    seen: set[tuple[str, str]] = set()
    unique: list[dict[str, str]] = []
    for item in pooled:
        key = (item.get("entity_id", ""), item.get("uid", "") or item.get("start", "") or item.get("summary", ""))
        if key in seen:
            continue
        seen.add(key)
        unique.append(item)
    return calendar_session.match_events(unique, summary)


def _with_slots(item: dict, extra: list[dict[str, str]]) -> dict[str, Any]:
    return {**item, "slots": [*(item.get("slots") or []), *extra]}


async def _speak(
    hass: HomeAssistant,
    pack: str,
    item: dict,
    *,
    calendar_events: list[dict[str, Any]] | None = None,
    extra_slots: list[dict[str, str]] | None = None,
) -> str | None:
    spoken_item = _with_slots(item, extra_slots or [])
    return await try_engine_speech(
        hass,
        pack,
        "default",
        spoken_item,
        calendar_events=calendar_events,
    )


async def handle_calendar_intent(
    hass: HomeAssistant,
    item: dict,
    pack: str,
    exposed: Callable[[str], bool],
    conversation_id: str | None = None,
) -> tuple[bool, str | None, str | None]:
    name = str(item.get("name") or "")
    if name == "KlarNoMusicPlayer":
        return True, await _speak(hass, pack, item), None
    if name == "KlarCreateCalendarEvent":
        return await _create(hass, item, pack, exposed, conversation_id)
    if name == "KlarGetCalendarEvents":
        return await _list(hass, item, pack, exposed, conversation_id)
    if name == "KlarDeleteCalendarEvent":
        return await _delete(hass, item, pack, exposed, conversation_id)
    if name == "KlarMoveCalendarEvent":
        return await _move(hass, item, pack, exposed, conversation_id)
    return False, None, "unsupported_calendar"


async def _list(
    hass: HomeAssistant,
    item: dict,
    pack: str,
    exposed: Callable[[str], bool],
    conversation_id: str | None = None,
) -> tuple[bool, str | None, str | None]:
    targets = _calendars(hass, item, exposed)
    if not targets:
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "cue", "value": "none"}]), None
    day = _slot(item, "day")
    start = end = None
    if day or _slot(item, "in_days"):
        start, end, _ = _when_bounds(item, hass)
    events, records = await calendar_entity.collect(hass, targets, start, end)
    calendar_session.remember(conversation_id, records)
    snapshot = [_event_snapshot(event, pack, hass) for event in events]
    return True, await _speak(hass, pack, item, calendar_events=snapshot), None


async def _delete(
    hass: HomeAssistant,
    item: dict,
    pack: str,
    exposed: Callable[[str], bool],
    conversation_id: str | None,
) -> tuple[bool, str | None, str | None]:
    if _slot(item, "need") == "which":
        return True, await _speak(hass, pack, item), None
    targets = _calendars(hass, item, exposed)
    if not targets:
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "cue", "value": "none"}]), None
    _, records = await calendar_entity.collect(hass, targets)
    hits = _pick(conversation_id, _slot(item, "summary"), records)
    if len(hits) != 1:
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "need", "value": "which"}]), None
    hit = hits[0]
    if not hit.get("uid"):
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "cue", "value": "no_uid"}]), None
    if not await calendar_entity.delete_event(hass, hit):
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "cue", "value": "readonly"}]), None
    leftover = [row for row in calendar_session.last_events(conversation_id) if row.get("uid") != hit.get("uid")]
    calendar_session.remember(conversation_id, leftover)
    return True, await _speak(
        hass, pack, item, extra_slots=[{"name": "summary", "value": hit.get("summary", "")}]
    ), None


async def _move(
    hass: HomeAssistant,
    item: dict,
    pack: str,
    exposed: Callable[[str], bool],
    conversation_id: str | None,
) -> tuple[bool, str | None, str | None]:
    need = _slot(item, "need")
    if need == "when":
        return True, await _speak(hass, pack, item), None
    if need == "which":
        return True, await _speak(hass, pack, item), None
    targets = _calendars(hass, item, exposed)
    if not targets:
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "cue", "value": "none"}]), None
    _, records = await calendar_entity.collect(hass, targets)
    hits = _pick(conversation_id, _slot(item, "summary"), records)
    if len(hits) != 1:
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "need", "value": "which"}]), None
    hit = hits[0]
    start, end, all_day = _when_bounds(item, hass)
    data = _event_data(hit.get("summary") or _slot(item, "summary"), start, end, all_day)
    if not await calendar_entity.move_event(hass, hit, data, start, end, all_day):
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "cue", "value": "readonly"}]), None
    when = calendar_say.when_from_bounds(start, all_day, pack, hass)
    return True, await _speak(
        hass,
        pack,
        item,
        extra_slots=[{"name": "summary", "value": data["summary"]}, {"name": "when", "value": when}],
    ), None


async def _create(
    hass: HomeAssistant,
    item: dict,
    pack: str,
    exposed: Callable[[str], bool],
    conversation_id: str | None,
) -> tuple[bool, str | None, str | None]:
    need = _slot(item, "need")
    if need == "title":
        return True, await _speak(hass, pack, item), None
    if need == "when":
        return True, await _speak(hass, pack, item), None
    summary = _slot(item, "summary")
    if not summary:
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "need", "value": "title"}]), None
    targets = _calendars(hass, item, exposed)
    if not targets:
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "cue", "value": "none"}]), None
    start, end, all_day = _when_bounds(item, hass)
    data = _event_data(summary, start, end, all_day)
    if not await calendar_entity.create_event(hass, targets[0], data):
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "cue", "value": "readonly"}]), None
    _, records = await calendar_entity.collect(hass, targets[:1])
    calendar_session.remember(conversation_id, records)
    when = calendar_say.when_from_bounds(start, all_day, pack, hass)
    return True, await _speak(
        hass,
        pack,
        item,
        extra_slots=[{"name": "summary", "value": summary}, {"name": "when", "value": when}],
    ), None


def _event_snapshot(event: dict[str, Any], pack: str, hass: HomeAssistant) -> dict[str, str]:
    summary = str(event.get("summary") or event.get("title") or "").strip()
    start = calendar_say._as_start(event)
    if start is None:
        return {"summary": summary, "start": ""}
    when = calendar_say.when_label(start, calendar_say.is_all_day(event, start), pack, hass)
    return {"summary": summary, "start": when}


def _event_data(summary: str, start: datetime, end: datetime, all_day: bool) -> dict[str, str]:
    data = {"summary": summary}
    if all_day:
        data["start_date"] = start.date().isoformat()
        data["end_date"] = end.date().isoformat()
        return data
    data["start_date_time"] = start.isoformat()
    data["end_date_time"] = end.isoformat()
    return data
