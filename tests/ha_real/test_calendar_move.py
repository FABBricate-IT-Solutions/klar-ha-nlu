"""H-03: moving an event must not lose it.

Before package 1.11 a move that could not update deleted the old event first and
created the new one afterwards, with the title and the times only. When creating
failed the event was gone. Recurring events went through the same path.
"""

from __future__ import annotations

from datetime import timedelta

from calendars import READ_ONLY, WRITE, asked, fake_calendar, family_calendar, stored, tomorrow_at
from engine_fake import FakeEngine, intent
from homeassistant.core import HomeAssistant
from pytest_homeassistant_custom_component.common import MockConfigEntry
from support import spoken_input

from custom_components.klar_nlu import dispatch


async def _run(hass: HomeAssistant, item: dict, conversation_id: str = "calendar-test") -> dispatch.IntentStepResult:
    turn = spoken_input("intent", conversation_id=conversation_id)
    result = await dispatch.handle_intent(hass, turn, item, "en", "conversation", lambda _entity_id: True)
    await hass.async_block_till_done()
    return result


def _move(summary: str, **when: object) -> dict:
    return intent("KlarMoveCalendarEvent", domain="calendar", summary=summary, **when)


async def test_old_event_survives_when_the_new_one_cannot_be_created(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    calendar = await fake_calendar(hass, "Family", WRITE)
    calendar.put("Dentist", tomorrow_at(10), tomorrow_at(11))
    calendar.fail_create = True

    result = await _run(hass, _move("dentist", day="tomorrow", hour=15))

    assert result.ok, result.error
    assert calendar.titles() == ["Dentist"], "the event was deleted although the new one was never created"
    assert calendar.events[0].start == tomorrow_at(10)
    assert "delete" not in calendar.calls
    assert asked(engine)[-1].get("cue") == "readonly"


async def test_fallback_creates_first_and_carries_every_field(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    calendar = await fake_calendar(hass, "Family", WRITE)
    calendar.put(
        "Dentist", tomorrow_at(10), tomorrow_at(12), description="Bring the X-rays", location="Main St 5"
    )

    result = await _run(hass, _move("dentist", day="tomorrow", hour=15))

    assert result.ok, result.error
    assert calendar.calls == ["create", "delete"]
    assert len(calendar.events) == 1
    moved = calendar.events[0]
    assert moved.summary == "Dentist"
    assert moved.start == tomorrow_at(15)
    assert moved.end == tomorrow_at(17), "the event lost its length"
    assert moved.description == "Bring the X-rays"
    assert moved.location == "Main St 5"


async def test_copy_is_taken_back_when_the_old_event_cannot_be_deleted(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    calendar = await fake_calendar(hass, "Family", WRITE)
    old = calendar.put("Dentist", tomorrow_at(10), tomorrow_at(11))
    calendar.keeps.add(str(old.uid))

    result = await _run(hass, _move("dentist", day="tomorrow", hour=15))

    assert result.ok, result.error
    assert calendar.calls == ["create", "delete", "delete"]
    assert [event.start for event in calendar.events] == [tomorrow_at(10)], "the event is in the calendar twice"
    assert asked(engine)[-1].get("cue") == "readonly"


async def test_recurring_event_is_not_moved_through_the_fallback(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    calendar = await fake_calendar(hass, "Family", WRITE)
    calendar.put(
        "Dentist", tomorrow_at(10), tomorrow_at(11), rrule="FREQ=WEEKLY;COUNT=3", recurrence_id="20261125T100000"
    )

    result = await _run(hass, _move("dentist", day="tomorrow", hour=15))

    assert result.ok, result.error
    assert calendar.calls == [], "a series was deleted and replaced by one single event"
    assert calendar.events[0].start == tomorrow_at(10)
    assert calendar.events[0].rrule == "FREQ=WEEKLY;COUNT=3"
    assert asked(engine)[-1].get("cue") == "readonly"


async def test_read_only_calendar_is_not_touched(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    calendar = await fake_calendar(hass, "Holidays", READ_ONLY)
    calendar.put("Dentist", tomorrow_at(10), tomorrow_at(11))

    result = await _run(hass, _move("dentist", day="tomorrow", hour=15))

    assert result.ok, result.error
    assert calendar.calls == []
    assert asked(engine)[-1].get("cue") == "readonly"


async def test_local_calendar_move_keeps_description_location_and_length(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, memory_store: None
) -> None:
    calendar = await family_calendar(hass)
    await calendar.async_create_event(
        summary="Dentist",
        description="Bring the X-rays",
        location="Main St 5",
        dtstart=tomorrow_at(10),
        dtend=tomorrow_at(12),
    )

    result = await _run(hass, _move("dentist", day="tomorrow", hour=15))

    assert result.ok, result.error
    events = await stored(hass, calendar)
    assert [event.summary for event in events] == ["Dentist"]
    assert events[0].start == tomorrow_at(15)
    assert events[0].end == tomorrow_at(17)
    assert events[0].description == "Bring the X-rays"
    assert events[0].location == "Main St 5"
    assert asked(engine)[-1].get("summary") == "Dentist"


async def test_day_only_move_keeps_the_time_of_day(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, memory_store: None
) -> None:
    calendar = await family_calendar(hass)
    await calendar.async_create_event(summary="Dentist", dtstart=tomorrow_at(10), dtend=tomorrow_at(11))

    result = await _run(hass, _move("dentist", in_days=3))

    assert result.ok, result.error
    events = await stored(hass, calendar)
    assert len(events) == 1
    assert events[0].start == tomorrow_at(10) + timedelta(days=2), "a timed event became an all-day event"
    assert events[0].end == tomorrow_at(11) + timedelta(days=2)


async def test_local_calendar_moves_one_occurrence_and_keeps_the_series(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, memory_store: None
) -> None:
    calendar = await family_calendar(hass)
    await calendar.async_create_event(
        summary="Dentist", dtstart=tomorrow_at(10), dtend=tomorrow_at(11), rrule="FREQ=WEEKLY;COUNT=3"
    )

    result = await _run(hass, _move("dentist", day="tomorrow", hour=15))

    assert result.ok, result.error
    starts = sorted(event.start for event in await stored(hass, calendar))
    assert starts == [tomorrow_at(15), tomorrow_at(10) + timedelta(days=7), tomorrow_at(10) + timedelta(days=14)]
