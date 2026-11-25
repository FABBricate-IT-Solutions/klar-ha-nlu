"""Native calendar list/create/delete/move and pack-local speech."""

from __future__ import annotations

from collections.abc import Callable
from datetime import date, datetime, timedelta
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

# More candidates than this are not read out; Klar only asks which one is meant.
MAX_NAMED = 3
_LISTING = {"name": "KlarGetCalendarEvents", "slots": []}

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


_WEEKDAYS = {
    "mon": 0,
    "tue": 1,
    "wed": 2,
    "thu": 3,
    "fri": 4,
    "sat": 5,
    "sun": 6,
    "monday": 0,
    "tuesday": 1,
    "wednesday": 2,
    "thursday": 3,
    "friday": 4,
    "saturday": 5,
    "sunday": 6,
}


def _when_bounds(item: dict, hass: HomeAssistant) -> tuple[datetime, datetime, bool] | None:
    """Start, end and all-day flag from the engine's slots.

    None when a slot holds something this code does not understand. The caller then
    asks for the date instead of silently writing the event for today.
    """
    now = datetime.now(calendar_entity.timezone(hass)).replace(second=0, microsecond=0)
    day = _slot(item, "day")
    hour_raw = _slot(item, "hour")
    in_days = _slot(item, "in_days")
    weekday = _WEEKDAYS.get(day.lower()) if day else None
    if day not in ("", "today", "tomorrow") and weekday is None:
        return None
    if in_days and not in_days.isdigit():
        return None
    if hour_raw and not (hour_raw.isdigit() and int(hour_raw) <= 23):
        return None
    start = now.replace(hour=0, minute=0)
    if day == "tomorrow":
        start = start + timedelta(days=1)
    elif weekday is not None:
        # Python Monday=0; advance to the next matching weekday (today counts).
        delta = (weekday - start.weekday()) % 7
        start = start + timedelta(days=delta)
    elif in_days:
        start = start + timedelta(days=int(in_days))
    if hour_raw:
        start = start.replace(hour=int(hour_raw), minute=0)
        return start, start + timedelta(hours=1), False
    return start, start + timedelta(days=1), True


def _already_over(hass: HomeAssistant, start: datetime, all_day: bool) -> bool:
    """A timed event that would start before now. Nobody adds an event to the past by voice."""
    return not all_day and start < datetime.now(calendar_entity.timezone(hass))


def _moved_bounds(
    hass: HomeAssistant, hit: dict[str, str], start: datetime, end: datetime, all_day: bool
) -> tuple[datetime, datetime, bool]:
    """The new times of a moved event: what the user did not change stays as it was."""
    old_start = calendar_entity.parse_stamp(hit.get("start", ""))
    old_end = calendar_entity.parse_stamp(hit.get("end", ""))
    if not isinstance(old_start, datetime):
        if all_day and isinstance(old_end, date) and isinstance(old_start, date) and old_end > old_start:
            return start, start + (old_end - old_start), True
        return start, end, all_day
    if old_start.tzinfo is None:
        old_start = old_start.replace(tzinfo=calendar_entity.timezone(hass))
    if all_day:
        # Only a day was named: the event keeps its time of day.
        local = old_start.astimezone(calendar_entity.timezone(hass))
        start = start.replace(hour=local.hour, minute=local.minute)
    if isinstance(old_end, datetime):
        if old_end.tzinfo is None:
            old_end = old_end.replace(tzinfo=calendar_entity.timezone(hass))
        if old_end > old_start:
            return start, start + (old_end - old_start), False
    return start, start + timedelta(hours=1), False


def _calendars(hass: HomeAssistant, item: dict, exposed: Callable[[str], bool]) -> list[str]:
    wanted = _slot(item, "entity_id")
    ids = [wanted] if wanted.startswith("calendar.") else []
    if not ids:
        ids = [state.entity_id for state in hass.states.async_all("calendar") if exposed(state.entity_id)]
    return [entity_id for entity_id in ids if exposed(entity_id) and hass.states.get(entity_id) is not None]


def _pick(conversation_id: str | None, summary: str, records: list[dict[str, str]]) -> list[dict[str, str]]:
    # What the calendar holds now comes first, so a remembered copy never wins over it.
    pooled = records + calendar_session.last_events(conversation_id)
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


async def _speak_naming(
    hass: HomeAssistant, pack: str, item: dict, events: list[dict[str, str]], extra_slots: list[dict[str, str]]
) -> str | None:
    """Say the events by name, then the sentence for ``item``.

    The engine's sentences for deleting and for "which event?" do not carry a title, so
    the events are read out the way a calendar listing reads them.
    """
    names = None
    if 0 < len(events) <= MAX_NAMED:
        names = await _speak(hass, pack, _LISTING, calendar_events=[_event_snapshot(row, pack, hass) for row in events])
    if names and names[-1] not in ".!?。":
        names += "."  # the engine joins listed events with ". " and leaves the end open
    said = await _speak(hass, pack, item, extra_slots=extra_slots)
    return " ".join(part for part in (names, said) if part) or None


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
    start = end = None
    bounds = _when_bounds(item, hass) if _slot(item, "day") or _slot(item, "in_days") else None
    if bounds is not None:
        start, end, _ = bounds
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
    summary = _slot(item, "summary")
    hits = _pick(conversation_id, summary, records)
    if not summary or len(hits) != 1:
        # Without a title Klar never guesses: it names what is there and asks.
        return True, await _speak_naming(hass, pack, item, hits, [{"name": "need", "value": "which"}]), None
    hit = hits[0]
    if not hit.get("uid"):
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "cue", "value": "no_uid"}]), None
    writable = calendar_entity.supports(hass, hit["entity_id"], calendar_entity.DELETE_EVENT)
    if not writable or not await calendar_entity.delete_event(hass, hit):
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "cue", "value": "readonly"}]), None
    leftover = [row for row in calendar_session.last_events(conversation_id) if row.get("uid") != hit.get("uid")]
    calendar_session.remember(conversation_id, leftover)
    return True, await _speak_naming(
        hass, pack, item, [hit], [{"name": "summary", "value": hit.get("summary", "")}]
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
    if not hit.get("uid"):
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "cue", "value": "no_uid"}]), None
    bounds = _when_bounds(item, hass)
    if bounds is None:
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "need", "value": "when"}]), None
    start, end, all_day = _moved_bounds(hass, hit, *bounds)
    if _already_over(hass, start, all_day):
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "need", "value": "when"}]), None
    data = _event_data(hit.get("summary") or _slot(item, "summary"), start, end, all_day)
    data.update({key: hit[key] for key in ("description", "location") if hit.get(key)})
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
    bounds = _when_bounds(item, hass)
    if bounds is None or _already_over(hass, bounds[0], bounds[2]):
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "need", "value": "when"}]), None
    start, end, all_day = bounds
    data = _event_data(summary, start, end, all_day)
    # The first calendar may be a read-only one (holidays, birthdays): write where writing works.
    writable = [target for target in targets if calendar_entity.supports(hass, target, calendar_entity.CREATE_EVENT)]
    if not writable or not await calendar_entity.create_event(hass, writable[0], data):
        return True, await _speak(hass, pack, item, extra_slots=[{"name": "cue", "value": "readonly"}]), None
    _, records = await calendar_entity.collect(hass, writable[:1])
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
