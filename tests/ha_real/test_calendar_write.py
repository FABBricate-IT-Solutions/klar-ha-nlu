"""H-04 and P-19: deleting and creating calendar events.

Before package 1.11 "delete it" without a title removed whatever single event the
next 36 hours held, a new event went to the first calendar even when that one was
read-only, and a date Klar did not understand became "today".
"""

from __future__ import annotations

from datetime import datetime, timedelta

import pytest
from calendars import READ_ONLY, WRITE, asked, fake_calendar, named, tomorrow_at
from engine_fake import FakeEngine, intent
from freezegun.api import FrozenDateTimeFactory
from homeassistant.core import HomeAssistant
from homeassistant.util import dt as dt_util
from pytest_homeassistant_custom_component.common import MockConfigEntry
from support import spoken_input

from custom_components.klar_nlu import dispatch

# A Tuesday afternoon in the test time zone (US/Pacific).
TUESDAY_4PM = "2026-11-24 16:00:00-08:00"


async def _run(hass: HomeAssistant, item: dict, conversation_id: str = "calendar-test") -> dispatch.IntentStepResult:
    turn = spoken_input("intent", conversation_id=conversation_id)
    result = await dispatch.handle_intent(hass, turn, item, "en", "conversation", lambda _entity_id: True)
    await hass.async_block_till_done()
    return result


def _create(summary: str, **when: object) -> dict:
    return intent("KlarCreateCalendarEvent", domain="calendar", summary=summary, **when)


def _delete(**slots: object) -> dict:
    return intent("KlarDeleteCalendarEvent", domain="calendar", **slots)


async def test_delete_without_a_title_asks_and_names_the_event(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    calendar = await fake_calendar(hass, "Family")
    calendar.put("Dentist", tomorrow_at(10), tomorrow_at(11))

    result = await _run(hass, _delete())

    assert result.ok, result.error
    assert calendar.titles() == ["Dentist"], "the only event of the next 36 hours was deleted without being named"
    assert calendar.calls == []
    assert asked(engine)[-1].get("need") == "which"
    assert named(engine) == ["Dentist"]


async def test_delete_without_a_title_after_listing_still_asks(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    calendar = await fake_calendar(hass, "Family")
    calendar.put("Dentist", tomorrow_at(10), tomorrow_at(11))
    await _run(hass, intent("KlarGetCalendarEvents", domain="calendar"))

    await _run(hass, _delete())

    assert calendar.titles() == ["Dentist"]
    assert asked(engine)[-1].get("need") == "which"


async def test_delete_with_a_title_deletes_and_names_the_event(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    calendar = await fake_calendar(hass, "Family")
    calendar.put("Dentist", tomorrow_at(10), tomorrow_at(11))
    calendar.put("Football", tomorrow_at(17), tomorrow_at(18))

    result = await _run(hass, _delete(summary="dentist"))

    assert result.ok, result.error
    assert calendar.titles() == ["Football"]
    assert named(engine) == ["Dentist"]
    assert asked(engine)[-1].get("summary") == "Dentist"
    assert "need" not in asked(engine)[-1]


async def test_delete_on_a_read_only_calendar_says_so(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    calendar = await fake_calendar(hass, "Holidays", READ_ONLY)
    calendar.put("Dentist", tomorrow_at(10), tomorrow_at(11))

    await _run(hass, _delete(summary="dentist"))

    assert calendar.titles() == ["Dentist"]
    assert calendar.calls == []
    assert asked(engine)[-1].get("cue") == "readonly"


async def test_new_event_goes_to_a_calendar_that_can_be_written(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    holidays = await fake_calendar(hass, "Holidays", READ_ONLY)
    family = await fake_calendar(hass, "Family", WRITE)
    assert [state.entity_id for state in hass.states.async_all("calendar")][0] == holidays.entity_id

    result = await _run(hass, _create("dentist", day="tomorrow", hour=15))

    assert result.ok, result.error
    assert holidays.calls == []
    assert family.titles() == ["dentist"]
    assert family.events[0].start == tomorrow_at(15)
    assert asked(engine)[-1].get("cue") is None
    assert asked(engine)[-1].get("summary") == "dentist"


async def test_new_event_with_only_read_only_calendars_says_so(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    holidays = await fake_calendar(hass, "Holidays", READ_ONLY)

    await _run(hass, _create("dentist", day="tomorrow", hour=15))

    assert holidays.events == []
    assert holidays.calls == []
    assert asked(engine)[-1].get("cue") == "readonly"


async def test_a_time_that_has_passed_today_creates_nothing(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, freezer: FrozenDateTimeFactory
) -> None:
    """What the engine sent for "friday at 3": an hour and no day."""
    freezer.move_to(TUESDAY_4PM)
    calendar = await fake_calendar(hass, "Family")

    result = await _run(hass, _create("dentist", hour=3))

    assert result.ok, result.error
    assert calendar.events == [], "an event was created in the past"
    assert calendar.calls == []
    assert asked(engine)[-1].get("need") == "when"


@pytest.mark.parametrize(
    "when",
    [{"day": "friday", "hour": 15}, {"day": "next week"}, {"in_days": "a few"}, {"day": "tomorrow", "hour": "noon"}],
)
async def test_a_date_klar_does_not_understand_asks_instead_of_assuming_today(
    hass: HomeAssistant,
    klar: MockConfigEntry,
    engine: FakeEngine,
    freezer: FrozenDateTimeFactory,
    when: dict[str, object],
) -> None:
    freezer.move_to(TUESDAY_4PM)
    calendar = await fake_calendar(hass, "Family")

    await _run(hass, _create("dentist", **when))

    assert calendar.events == []
    assert asked(engine)[-1].get("need") == "when"


async def test_dates_klar_understands_are_still_created(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, freezer: FrozenDateTimeFactory
) -> None:
    freezer.move_to(TUESDAY_4PM)
    calendar = await fake_calendar(hass, "Family")
    now = dt_util.now()

    await _run(hass, _create("dinner", day="today", hour=18))
    await _run(hass, _create("dentist", day="tomorrow", hour=3))
    await _run(hass, _create("school trip", in_days=3))
    await _run(hass, _create("cleaning", day="today"))

    starts = {stored.summary: stored.start for stored in calendar.events}
    assert starts["dinner"] == now.replace(hour=18)
    assert starts["dentist"] == now.replace(hour=3) + timedelta(days=1)
    assert starts["school trip"] == now.date() + timedelta(days=3)
    assert starts["cleaning"] == now.date()
    assert isinstance(starts["dinner"], datetime)


async def test_move_to_a_time_that_has_passed_changes_nothing(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, freezer: FrozenDateTimeFactory
) -> None:
    freezer.move_to(TUESDAY_4PM)
    calendar = await fake_calendar(hass, "Family")
    calendar.put("Dentist", tomorrow_at(10), tomorrow_at(11))

    await _run(hass, intent("KlarMoveCalendarEvent", domain="calendar", summary="dentist", hour=3))

    assert calendar.calls == []
    assert calendar.events[0].start == tomorrow_at(10)
    assert asked(engine)[-1].get("need") == "when"
