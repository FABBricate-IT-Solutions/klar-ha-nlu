"""Calendars for the real-Home-Assistant tests.

``FakeCalendar`` is a real ``CalendarEntity`` kept in memory, with the capabilities a
test asks for. ``family_calendar`` sets up Home Assistant's own ``local_calendar``.
"""

from __future__ import annotations

from datetime import date, datetime, timedelta
from pathlib import Path
from typing import Any

from engine_fake import FakeEngine
from ha_entities import add_entities
from homeassistant.components.calendar import CalendarEntity, CalendarEntityFeature, CalendarEvent
from homeassistant.components.homeassistant.exposed_entities import async_expose_entity
from homeassistant.components.local_calendar.store import LocalCalendarStore
from homeassistant.core import HomeAssistant
from homeassistant.exceptions import HomeAssistantError
from homeassistant.util import dt as dt_util
from pytest_homeassistant_custom_component.common import MockConfigEntry

WRITE = CalendarEntityFeature.CREATE_EVENT | CalendarEntityFeature.DELETE_EVENT
READ_ONLY = CalendarEntityFeature(0)


class FakeCalendar(CalendarEntity):
    """An in-memory calendar that records every write Klar asked for."""

    _attr_should_poll = False

    def __init__(self, name: str, uid: str, features: CalendarEntityFeature = WRITE) -> None:
        self._attr_name = name
        self._attr_unique_id = uid
        self._attr_supported_features = features
        self.events: list[CalendarEvent] = []
        self.calls: list[str] = []
        self.fail_create = False
        self.keeps: set[str] = set()
        self._count = 0

    @property
    def event(self) -> CalendarEvent | None:
        return self.events[0] if self.events else None

    def put(self, summary: str, start: datetime | date, end: datetime | date, **fields: Any) -> CalendarEvent:
        """Store an event without going through Klar."""
        self._count += 1
        stored = CalendarEvent(start=start, end=end, summary=summary, uid=f"event-{self._count}", **fields)
        self.events.append(stored)
        return stored

    def titles(self) -> list[str]:
        return [stored.summary for stored in self.events]

    async def async_get_events(self, hass: HomeAssistant, start_date: datetime, end_date: datetime) -> list[CalendarEvent]:
        return [
            stored
            for stored in self.events
            if stored.start_datetime_local < end_date and stored.end_datetime_local > start_date
        ]

    async def async_create_event(self, **kwargs: Any) -> None:
        self.calls.append("create")
        if self.fail_create:
            raise HomeAssistantError("the calendar refused the new event")
        self.put(
            kwargs["summary"],
            kwargs["dtstart"],
            kwargs["dtend"],
            description=kwargs.get("description"),
            location=kwargs.get("location"),
        )

    async def async_delete_event(
        self, uid: str, recurrence_id: str | None = None, recurrence_range: str | None = None
    ) -> None:
        self.calls.append("delete")
        if uid in self.keeps:
            raise HomeAssistantError("the calendar refused to delete")
        self.events = [stored for stored in self.events if stored.uid != uid]


class MemoryStore(LocalCalendarStore):
    """Keeps the local calendar's file in memory."""

    def __init__(self, hass: HomeAssistant, path: Path) -> None:
        super().__init__(hass, path)
        self.text = ""

    def _load(self) -> str:
        return self.text

    def _store(self, ics_content: str) -> None:
        self.text = ics_content


async def fake_calendar(
    hass: HomeAssistant, name: str, features: CalendarEntityFeature = WRITE
) -> FakeCalendar:
    calendar = FakeCalendar(name, f"calendar-{name.casefold().replace(' ', '-')}", features)
    await add_entities(hass, "calendar", calendar)
    async_expose_entity(hass, "conversation", calendar.entity_id, True)
    return calendar


async def family_calendar(hass: HomeAssistant) -> CalendarEntity:
    """Home Assistant's local calendar, exposed to Klar. Needs ``memory_store``."""
    entry = MockConfigEntry(
        domain="local_calendar",
        title="Family",
        data={"calendar_name": "Family", "storage_key": "family"},
        unique_id="family",
    )
    entry.add_to_hass(hass)
    assert await hass.config_entries.async_setup(entry.entry_id)
    await hass.async_block_till_done()
    async_expose_entity(hass, "conversation", "calendar.family", True)
    entity = hass.data["calendar"].get_entity("calendar.family")
    assert isinstance(entity, CalendarEntity)
    return entity


async def stored(hass: HomeAssistant, calendar: CalendarEntity, days: int = 30) -> list[CalendarEvent]:
    """Every event the calendar holds in the coming ``days``."""
    start = dt_util.now() - timedelta(days=1)
    return await calendar.async_get_events(hass, start, start + timedelta(days=days))


def tomorrow_at(hour: int) -> datetime:
    start = dt_util.now().replace(hour=hour, minute=0, second=0, microsecond=0)
    return start + timedelta(days=1)


def asked(engine: FakeEngine) -> list[dict[str, str]]:
    """The slots of every sentence Klar asked the engine to speak, in order."""
    return [
        {str(slot["name"]): str(slot["value"]) for slot in snapshot["intent"]["slots"]}
        for snapshot in engine.speech_snapshots
    ]


def named(engine: FakeEngine) -> list[str]:
    """The event titles Klar handed to the engine to be spoken."""
    return [str(row["summary"]) for snapshot in engine.speech_snapshots for row in snapshot["calendar_events"]]
