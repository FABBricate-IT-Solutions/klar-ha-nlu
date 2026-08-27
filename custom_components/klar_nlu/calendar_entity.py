"""Read and mutate HA calendars through the entity API.

``calendar.get_events`` omits ``uid``. Delete and update are not REST services.
"""

from __future__ import annotations

from datetime import date, datetime, timedelta
from typing import Any
from zoneinfo import ZoneInfo

from homeassistant.core import HomeAssistant

try:
    from . import calendar_session
except ImportError:
    import calendar_session  # type: ignore[no-redef]


def timezone(hass: HomeAssistant) -> ZoneInfo:
    name = str(getattr(getattr(hass, "config", None), "time_zone", None) or "UTC")
    try:
        return ZoneInfo(name)
    except Exception:  # noqa: BLE001 — HA timezone is a system boundary
        return ZoneInfo("UTC")


def window(hass: HomeAssistant, hours: int = 36) -> tuple[datetime, datetime]:
    start = datetime.now(timezone(hass)).replace(second=0, microsecond=0)
    return start, start + timedelta(hours=hours)


def calendar_entity(hass: HomeAssistant, entity_id: str) -> Any | None:
    try:
        from homeassistant.helpers.entity_component import EntityComponentKey

        found = hass.data.get(EntityComponentKey("calendar"))
        entity = found.get_entity(entity_id) if found is not None else None
        if entity is not None:
            return entity
    except Exception:  # noqa: BLE001 — HA storage key moved across versions
        pass
    for value in list(hass.data.values()):
        getter = getattr(value, "get_entity", None)
        if not callable(getter):
            continue
        try:
            entity = getter(entity_id)
        except Exception:  # noqa: BLE001
            continue
        if entity is not None and getattr(entity, "entity_id", None) == entity_id:
            return entity
    return None


def as_event(entity_id: str, raw: Any) -> dict[str, Any]:
    if isinstance(raw, dict):
        record = calendar_session.as_record(entity_id, raw)
        return {**raw, **record, "entity_id": entity_id}
    start = getattr(raw, "start", None)
    end = getattr(raw, "end", None)
    return {
        "summary": str(getattr(raw, "summary", None) or getattr(raw, "title", None) or "").strip(),
        "uid": str(getattr(raw, "uid", None) or getattr(raw, "id", None) or ""),
        "recurrence_id": str(getattr(raw, "recurrence_id", None) or ""),
        "start": start,
        "end": end,
        "entity_id": entity_id,
    }


async def collect(hass: HomeAssistant, targets: list[str]) -> tuple[list[dict[str, Any]], list[dict[str, str]]]:
    start, end = window(hass)
    events: list[dict[str, Any]] = []
    records: list[dict[str, str]] = []
    for entity_id in targets:
        found = await _events_for(hass, entity_id, start, end)
        for event in found:
            events.append(event)
            records.append(calendar_session.as_record(entity_id, event))
    return events, records


async def _events_for(hass: HomeAssistant, entity_id: str, start: datetime, end: datetime) -> list[dict[str, Any]]:
    entity = calendar_entity(hass, entity_id)
    if entity is not None and hasattr(entity, "async_get_events"):
        try:
            raw = await entity.async_get_events(hass, start, end)
        except Exception:  # noqa: BLE001 — HA calendar is a system boundary
            raw = None
        if isinstance(raw, list):
            return [as_event(entity_id, item) for item in raw]
    try:
        payload = await hass.services.async_call(
            "calendar",
            "get_events",
            {"start_date_time": start.isoformat(), "end_date_time": end.isoformat()},
            blocking=True,
            return_response=True,
            target={"entity_id": entity_id},
        )
    except Exception:  # noqa: BLE001
        return []
    block = payload.get(entity_id) if isinstance(payload, dict) else None
    found = block.get("events") if isinstance(block, dict) else None
    if not isinstance(found, list):
        return []
    return [as_event(entity_id, item) for item in found if isinstance(item, dict)]


async def create_event(hass: HomeAssistant, entity_id: str, data: dict[str, str]) -> bool:
    entity = calendar_entity(hass, entity_id)
    if entity is not None and hasattr(entity, "async_create_event"):
        try:
            await entity.async_create_event(**_create_kwargs(data))
            return True
        except Exception:  # noqa: BLE001
            pass
    try:
        await hass.services.async_call("calendar", "create_event", data, blocking=True, target={"entity_id": entity_id})
    except Exception:  # noqa: BLE001
        return False
    return True


async def delete_event(hass: HomeAssistant, hit: dict[str, str]) -> bool:
    entity_id = hit["entity_id"]
    uid = hit.get("uid") or ""
    if not uid:
        return False
    rid = hit.get("recurrence_id") or None
    entity = calendar_entity(hass, entity_id)
    if entity is not None and hasattr(entity, "async_delete_event"):
        try:
            await entity.async_delete_event(uid, recurrence_id=rid, recurrence_range=None)
            return True
        except TypeError:
            try:
                await entity.async_delete_event(uid)
                return True
            except Exception:  # noqa: BLE001 — entity rejected the delete
                return False
        except Exception:  # noqa: BLE001 — read-only calendars fail here
            return False
    try:
        data = {"uid": uid}
        if rid:
            data["recurrence_id"] = rid
        await hass.services.async_call("calendar", "delete_event", data, blocking=True, target={"entity_id": entity_id})
    except Exception:  # noqa: BLE001
        return False
    return True


async def move_event(hass: HomeAssistant, hit: dict[str, str], data: dict[str, str], start: datetime, end: datetime, all_day: bool) -> bool:
    entity = calendar_entity(hass, hit["entity_id"])
    uid = hit.get("uid") or ""
    if entity is not None and uid and hasattr(entity, "async_update_event"):
        payload = _update_payload(data, start, end, all_day)
        try:
            extra = {"recurrence_id": hit["recurrence_id"]} if hit.get("recurrence_id") else {}
            await entity.async_update_event(uid, payload, **extra)
            return True
        except Exception:  # noqa: BLE001 — not every calendar implements update
            pass
    if uid:
        deleted = await delete_event(hass, hit)
        if not deleted:
            return False
    return await create_event(hass, hit["entity_id"], data)


def _create_kwargs(data: dict[str, str]) -> dict[str, Any]:
    out: dict[str, Any] = {"summary": data["summary"]}
    if "start_date" in data:
        out["start_date"] = _as_date(data["start_date"])
        out["end_date"] = _as_date(data["end_date"])
        return out
    out["start_date_time"] = _as_datetime(data["start_date_time"])
    out["end_date_time"] = _as_datetime(data["end_date_time"])
    return out


def _update_payload(data: dict[str, str], start: datetime, end: datetime, all_day: bool) -> dict[str, Any]:
    if all_day:
        first, last = start.date(), end.date()
    else:
        first, last = start, end
    return {"summary": data.get("summary", ""), "dtstart": first, "dtend": last, "start": first, "end": last}


def _as_date(value: str) -> date:
    return date.fromisoformat(value[:10])


def _as_datetime(value: str) -> datetime:
    return datetime.fromisoformat(value)
