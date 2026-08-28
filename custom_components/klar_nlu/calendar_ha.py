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
    from .speech_locale import SPEECH_PACKS
except ImportError:
    try:
        from speech_locale import SPEECH_PACKS
    except ImportError:
        SPEECH_PACKS = {}

CALENDAR_INTENTS = {
    "KlarGetCalendarEvents",
    "KlarCreateCalendarEvent",
    "KlarDeleteCalendarEvent",
    "KlarMoveCalendarEvent",
    "KlarNoMusicPlayer",
}


def _pack(pack: str) -> dict[str, str]:
    return calendar_say.overlay(pack, SPEECH_PACKS.get(pack) or SPEECH_PACKS.get("en") or {})


def _fill(pack: str, key: str, **slots: str) -> str:
    if key in calendar_say.templates(pack):
        return calendar_say.fill(pack, key, **slots)
    template = str(_pack(pack).get(key) or _pack("en").get(key) or "")
    for name, value in slots.items():
        template = template.replace(f"{{{name}}}", value)
    return template.strip()


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


async def handle_calendar_intent(
    hass: HomeAssistant,
    item: dict,
    pack: str,
    exposed: Callable[[str], bool],
    conversation_id: str | None = None,
) -> tuple[bool, str | None, str | None]:
    name = str(item.get("name") or "")
    if name == "KlarNoMusicPlayer":
        return True, _fill(pack, "no_music_player"), None
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
        return True, _fill(pack, "calendar_none"), None
    events, records = await calendar_entity.collect(hass, targets)
    calendar_session.remember(conversation_id, records)
    return True, calendar_say.list_speech(events, pack, hass), None


async def _delete(
    hass: HomeAssistant,
    item: dict,
    pack: str,
    exposed: Callable[[str], bool],
    conversation_id: str | None,
) -> tuple[bool, str | None, str | None]:
    if _slot(item, "need") == "which":
        return True, _fill(pack, "calendar_which"), None
    targets = _calendars(hass, item, exposed)
    if not targets:
        return True, _fill(pack, "calendar_none"), None
    _, records = await calendar_entity.collect(hass, targets)
    hits = _pick(conversation_id, _slot(item, "summary"), records)
    if len(hits) != 1:
        return True, _fill(pack, "calendar_which"), None
    hit = hits[0]
    if not hit.get("uid"):
        return True, _fill(pack, "calendar_no_uid"), None
    if not await calendar_entity.delete_event(hass, hit):
        return True, _fill(pack, "calendar_readonly"), None
    leftover = [row for row in calendar_session.last_events(conversation_id) if row.get("uid") != hit.get("uid")]
    calendar_session.remember(conversation_id, leftover)
    return True, _fill(pack, "calendar_deleted", summary=hit.get("summary", "")), None


async def _move(
    hass: HomeAssistant,
    item: dict,
    pack: str,
    exposed: Callable[[str], bool],
    conversation_id: str | None,
) -> tuple[bool, str | None, str | None]:
    need = _slot(item, "need")
    if need == "when":
        return True, _fill(pack, "calendar_need_when"), None
    if need == "which":
        return True, _fill(pack, "calendar_which"), None
    targets = _calendars(hass, item, exposed)
    if not targets:
        return True, _fill(pack, "calendar_none"), None
    _, records = await calendar_entity.collect(hass, targets)
    hits = _pick(conversation_id, _slot(item, "summary"), records)
    if len(hits) != 1:
        return True, _fill(pack, "calendar_which"), None
    hit = hits[0]
    start, end, all_day = _when_bounds(item, hass)
    data = _event_data(hit.get("summary") or _slot(item, "summary"), start, end, all_day)
    if not await calendar_entity.move_event(hass, hit, data, start, end, all_day):
        return True, _fill(pack, "calendar_readonly"), None
    when = calendar_say.when_from_bounds(start, all_day, pack, hass)
    return True, _fill(pack, "calendar_moved", summary=data["summary"], when=when), None


async def _create(
    hass: HomeAssistant,
    item: dict,
    pack: str,
    exposed: Callable[[str], bool],
    conversation_id: str | None,
) -> tuple[bool, str | None, str | None]:
    need = _slot(item, "need")
    if need == "title":
        return True, _fill(pack, "calendar_need_title"), None
    if need == "when":
        return True, _fill(pack, "calendar_need_when"), None
    summary = _slot(item, "summary")
    if not summary:
        return True, _fill(pack, "calendar_need_title"), None
    targets = _calendars(hass, item, exposed)
    if not targets:
        return True, _fill(pack, "calendar_none"), None
    start, end, all_day = _when_bounds(item, hass)
    data = _event_data(summary, start, end, all_day)
    if not await calendar_entity.create_event(hass, targets[0], data):
        return True, _fill(pack, "calendar_readonly"), None
    _, records = await calendar_entity.collect(hass, targets[:1])
    calendar_session.remember(conversation_id, records)
    when = calendar_say.when_from_bounds(start, all_day, pack, hass)
    return True, _fill(pack, "calendar_created", summary=summary, when=when), None


def _event_data(summary: str, start: datetime, end: datetime, all_day: bool) -> dict[str, str]:
    data = {"summary": summary}
    if all_day:
        data["start_date"] = start.date().isoformat()
        data["end_date"] = end.date().isoformat()
        return data
    data["start_date_time"] = start.isoformat()
    data["end_date_time"] = end.isoformat()
    return data
