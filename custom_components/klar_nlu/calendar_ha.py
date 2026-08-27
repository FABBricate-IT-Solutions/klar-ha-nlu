"""Native calendar list/create and pack-local speech."""

from __future__ import annotations

from collections.abc import Callable
from datetime import datetime, timedelta
from typing import Any
from zoneinfo import ZoneInfo

from homeassistant.core import HomeAssistant

try:
    from . import calendar_session
except ImportError:
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
    return SPEECH_PACKS.get(pack) or SPEECH_PACKS.get("en") or {}


def _fill(pack: str, key: str, **slots: str) -> str:
    template = str(_pack(pack).get(key) or _pack("en").get(key) or "")
    for name, value in slots.items():
        template = template.replace(f"{{{name}}}", value)
    return template.strip()


def _slot(item: dict, name: str) -> str:
    for raw in item.get("slots") or []:
        if isinstance(raw, dict) and raw.get("name") == name:
            return str(raw.get("value") or "")
    return ""


def _tz(hass: HomeAssistant) -> ZoneInfo:
    name = str(getattr(getattr(hass, "config", None), "time_zone", None) or "UTC")
    try:
        return ZoneInfo(name)
    except Exception:  # noqa: BLE001 — HA timezone is a system boundary
        return ZoneInfo("UTC")


def _clock(when: datetime, pack: str, hass: HomeAssistant) -> str:
    lang = str(getattr(getattr(hass, "config", None), "language", None) or pack).lower()
    if lang.startswith("en"):
        hour = when.strftime("%I").lstrip("0") or "12"
        return f"{hour}:{when.strftime('%M %p')}"
    return when.strftime("%H:%M")


def _window(hass: HomeAssistant, hours: int = 36) -> tuple[str, str]:
    start = datetime.now(_tz(hass)).replace(second=0, microsecond=0)
    end = start + timedelta(hours=hours)
    return start.isoformat(), end.isoformat()


def _when_bounds(item: dict, hass: HomeAssistant) -> tuple[datetime, datetime, bool]:
    now = datetime.now(_tz(hass)).replace(second=0, microsecond=0)
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
        ids = [
            state.entity_id
            for state in hass.states.async_all("calendar")
            if exposed(state.entity_id)
        ]
    return [entity_id for entity_id in ids if exposed(entity_id) and hass.states.get(entity_id) is not None]


def _event_line(event: dict, pack: str, hass: HomeAssistant) -> str:
    summary = str(event.get("summary") or event.get("title") or "").strip()
    start = event.get("start") or event.get("start_date_time") or event.get("start_date") or ""
    if isinstance(start, dict):
        start = start.get("dateTime") or start.get("date") or ""
    stamp = str(start)
    try:
        when = datetime.fromisoformat(stamp.replace("Z", "+00:00"))
        label = _clock(when.astimezone(_tz(hass)), pack, hass)
    except ValueError:
        label = stamp[:10]
    if summary and label:
        return f"{label} {summary}"
    return summary or label


def _list_speech(events: list[dict], pack: str, hass: HomeAssistant) -> str:
    if not events:
        return _fill(pack, "calendar_empty")
    lines = [_event_line(event, pack, hass) for event in events[:8]]
    items = ". ".join(line for line in lines if line)
    return _fill(pack, "calendar_list", items=items, count=str(len(events)))


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
        return await _create(hass, item, pack, exposed)
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
    events, records = await _collect(hass, targets)
    calendar_session.remember(conversation_id, records)
    return True, _list_speech(events, pack, hass), None


async def _collect(hass: HomeAssistant, targets: list[str]) -> tuple[list[dict], list[dict[str, str]]]:
    start, end = _window(hass)
    events: list[dict] = []
    records: list[dict[str, str]] = []
    for entity_id in targets:
        try:
            payload = await hass.services.async_call(
                "calendar",
                "get_events",
                {"start_date_time": start, "end_date_time": end},
                blocking=True,
                return_response=True,
                target={"entity_id": entity_id},
            )
        except Exception:  # noqa: BLE001 — HA calendar is a system boundary
            continue
        block = payload.get(entity_id) if isinstance(payload, dict) else None
        found = block.get("events") if isinstance(block, dict) else None
        if not isinstance(found, list):
            continue
        for event in found:
            if not isinstance(event, dict):
                continue
            events.append(event)
            records.append(calendar_session.as_record(entity_id, event))
    return events, records


def _pick(
    conversation_id: str | None,
    summary: str,
    records: list[dict[str, str]],
) -> list[dict[str, str]]:
    pooled = calendar_session.last_events(conversation_id) + records
    seen: set[tuple[str, str]] = set()
    unique: list[dict[str, str]] = []
    for item in pooled:
        key = (item.get("entity_id", ""), item.get("uid", "") or item.get("summary", ""))
        if key in seen:
            continue
        seen.add(key)
        unique.append(item)
    return calendar_session.match_events(unique, summary)


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
    _, records = await _collect(hass, targets)
    hits = _pick(conversation_id, _slot(item, "summary"), records)
    if len(hits) != 1:
        return True, _fill(pack, "calendar_which"), None
    hit = hits[0]
    if not hit.get("uid"):
        return True, _fill(pack, "calendar_no_uid"), None
    data = {"uid": hit["uid"]}
    if hit.get("recurrence_id"):
        data["recurrence_id"] = hit["recurrence_id"]
    try:
        await hass.services.async_call("calendar", "delete_event", data, blocking=True, target={"entity_id": hit["entity_id"]})
    except Exception:  # noqa: BLE001 — read-only calendars fail here
        return True, _fill(pack, "calendar_readonly"), None
    leftover = [row for row in calendar_session.last_events(conversation_id) if row != hit]
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
    _, records = await _collect(hass, targets)
    hits = _pick(conversation_id, _slot(item, "summary"), records)
    if len(hits) != 1:
        return True, _fill(pack, "calendar_which"), None
    hit = hits[0]
    start, end, all_day = _when_bounds(item, hass)
    data: dict[str, str] = {"summary": hit.get("summary") or _slot(item, "summary")}
    if all_day:
        data["start_date"] = start.date().isoformat()
        data["end_date"] = end.date().isoformat()
    else:
        data["start_date_time"] = start.isoformat()
        data["end_date_time"] = end.isoformat()
    moved = await _update_or_recreate(hass, hit, data)
    if not moved:
        return True, _fill(pack, "calendar_readonly"), None
    when = _clock(start, pack, hass) if not all_day else start.date().isoformat()
    return True, _fill(pack, "calendar_moved", summary=data["summary"], when=when), None


async def _update_or_recreate(hass: HomeAssistant, hit: dict[str, str], data: dict[str, str]) -> bool:
    entity_id = hit["entity_id"]
    if hit.get("uid"):
        try:
            await hass.services.async_call(
                "calendar",
                "update_event",
                {**data, "uid": hit["uid"], **({"recurrence_id": hit["recurrence_id"]} if hit.get("recurrence_id") else {})},
                blocking=True,
                target={"entity_id": entity_id},
            )
            return True
        except Exception:  # noqa: BLE001 — not every calendar implements update
            pass
        try:
            delete = {"uid": hit["uid"]}
            if hit.get("recurrence_id"):
                delete["recurrence_id"] = hit["recurrence_id"]
            await hass.services.async_call("calendar", "delete_event", delete, blocking=True, target={"entity_id": entity_id})
        except Exception:  # noqa: BLE001
            return False
    try:
        await hass.services.async_call("calendar", "create_event", data, blocking=True, target={"entity_id": entity_id})
    except Exception:  # noqa: BLE001
        return False
    return True


async def _create(
    hass: HomeAssistant,
    item: dict,
    pack: str,
    exposed: Callable[[str], bool],
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
    data = {"summary": summary}
    if all_day:
        data["start_date"] = start.date().isoformat()
        data["end_date"] = end.date().isoformat()
    else:
        data["start_date_time"] = start.isoformat()
        data["end_date_time"] = end.isoformat()
    try:
        await hass.services.async_call(
            "calendar",
            "create_event",
            data,
            blocking=True,
            target={"entity_id": targets[0]},
        )
    except Exception:  # noqa: BLE001 — read-only calendars fail here
        return True, _fill(pack, "calendar_readonly"), None
    when = _clock(start, pack, hass) if not all_day else start.date().isoformat()
    return True, _fill(pack, "calendar_created", summary=summary, when=when), None
