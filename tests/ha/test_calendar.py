#!/usr/bin/env python3
"""Calendar dispatch uses HA entity APIs and pack-native speech."""

from __future__ import annotations

import asyncio
import importlib.util
import sys
import types
import unittest
from datetime import date, datetime, timedelta, timezone
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, patch

ROOT = Path(__file__).resolve().parents[2]


def _module(name: str) -> types.ModuleType:
    module = types.ModuleType(name)
    module.__path__ = []
    return module


def _load(name: str, rel: str) -> types.ModuleType:
    path = ROOT / "custom_components" / "klar_nlu" / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def _load_calendar() -> types.ModuleType:
    homeassistant = _module("homeassistant")
    core = types.ModuleType("homeassistant.core")
    core.HomeAssistant = object
    helpers = _module("homeassistant.helpers")
    entity_component = types.ModuleType("homeassistant.helpers.entity_component")

    class EntityComponentKey:
        def __init__(self, domain: str) -> None:
            self.domain = domain

        def __hash__(self) -> int:
            return hash(self.domain)

        def __eq__(self, other: object) -> bool:
            return isinstance(other, EntityComponentKey) and other.domain == self.domain

    entity_component.EntityComponentKey = EntityComponentKey
    modules = {
        "homeassistant": homeassistant,
        "homeassistant.core": core,
        "homeassistant.helpers": helpers,
        "homeassistant.helpers.entity_component": entity_component,
    }
    with patch.dict(sys.modules, modules):
        _load("speech_locale", "speech_locale.py")
        _load("calendar_session", "calendar_session.py")
        _load("calendar_say", "calendar_say.py")
        _load("calendar_entity", "calendar_entity.py")
        return _load("klar_calendar_ha", "calendar_ha.py")


calendar_ha = _load_calendar()


async def _fake_engine_speech(hass, pack, personality, item, calendar_events=None, extra_entities=None):
    slots = {row.get("name"): row.get("value") for row in (item.get("slots") or []) if isinstance(row, dict)}
    need = str(slots.get("need") or "")
    cue = str(slots.get("cue") or "")
    name = str(item.get("name") or "")
    english = pack.startswith("en")
    if need == "title" or cue == "need_title":
        return "What should I call the event?" if english else "Wie soll der Termin heißen?"
    if need == "when" or cue == "need_when":
        return "When is the event?" if english else "Wann ist der Termin?"
    if need == "which" or cue == "which":
        return "Which event?" if english else "Welcher Termin?"
    if cue == "none":
        return "No calendar is available." if english else "Kein Kalender ist verfügbar."
    if cue == "readonly":
        return "That calendar cannot be changed." if english else "Dieser Kalender lässt sich nicht ändern."
    if cue == "no_uid":
        return "That event has no identifier." if english else "Dieser Termin hat keine Kennung."
    if name == "KlarGetCalendarEvents":
        if not calendar_events:
            return "No upcoming events." if english else "Keine anstehenden Termine."
        bits = []
        for event in calendar_events:
            summary = str(event.get("summary") or "")
            start = str(event.get("start") or "")
            bits.append(f"{summary} {start}".strip())
        return ". ".join(bits)
    summary = str(slots.get("summary") or "")
    when = str(slots.get("when") or "")
    if name in {"KlarCreateCalendarEvent", "KlarMoveCalendarEvent"}:
        return f"{summary} {when}.".strip()
    if name == "KlarDeleteCalendarEvent":
        return "Event deleted." if english else "Termin gelöscht."
    return summary or when or "ok."


calendar_ha.try_engine_speech = _fake_engine_speech


def _tomorrow_at(hour: int) -> datetime:
    now = datetime.now(timezone.utc).replace(second=0, microsecond=0)
    return now.replace(hour=hour, minute=0) + timedelta(days=1)


class _Event:
    def __init__(self, summary: str, uid: str, start: datetime | date, end: datetime | date | None = None, recurrence_id: str = "") -> None:
        self.summary = summary
        self.uid = uid
        self.start = start
        self.end = end
        self.recurrence_id = recurrence_id


class _Cal:
    def __init__(self, events: list[_Event] | None = None) -> None:
        self.entity_id = "calendar.home"
        self.events = list(events or [])
        self.deleted: list[str] = []
        self.created: list[dict] = []
        self.updated: list[tuple[str, dict]] = []
        self.fail_delete = False
        self.fail_update = False

    async def async_get_events(self, _hass: object, _start: object, _end: object) -> list[_Event]:
        return list(self.events)

    async def async_delete_event(self, uid: str, recurrence_id: str | None = None, recurrence_range: str | None = None) -> None:
        if self.fail_delete:
            raise RuntimeError("readonly")
        self.deleted.append(uid)
        self.events = [event for event in self.events if event.uid != uid]

    async def async_create_event(self, **kwargs: object) -> None:
        self.created.append(kwargs)
        start = kwargs.get("start_date_time") or kwargs.get("start_date")
        self.events.append(_Event(str(kwargs.get("summary") or ""), f"new-{len(self.created)}", start or _tomorrow_at(15)))

    async def async_update_event(self, uid: str, event: dict, recurrence_id: str | None = None, recurrence_range: str | None = None) -> None:
        if self.fail_update:
            raise RuntimeError("no update")
        self.updated.append((uid, event))


class _Hass:
    def __init__(self, cal: _Cal | None = None, language: str = "en") -> None:
        self.cal = cal or _Cal()
        self.config = SimpleNamespace(language=language, time_zone="UTC")
        self.services = SimpleNamespace(async_call=AsyncMock())
        self.states = SimpleNamespace(
            get=lambda entity_id: SimpleNamespace(entity_id=entity_id, state="on"),
            async_all=lambda domain: [SimpleNamespace(entity_id="calendar.home")] if domain == "calendar" else [],
        )
        self.data = {"calendar": SimpleNamespace(get_entity=lambda entity_id: self.cal if entity_id == "calendar.home" else None)}


class CalendarDispatchTests(unittest.TestCase):
    def test_list_speech_is_natural_german(self) -> None:
        start = datetime(2026, 8, 27, 0, 0, tzinfo=timezone.utc)
        hass = _Hass(_Cal([_Event("test", "uid-0", start.date())]), language="de")
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(hass, {"name": "KlarGetCalendarEvents", "slots": []}, "de", lambda _: True)
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertIsNotNone(speech)
        self.assertNotIn("00:00", speech or "")
        self.assertNotIn("upcoming", (speech or "").lower())
        self.assertIn("test", speech or "")

    def test_list_english_all_day(self) -> None:
        hass = _Hass(_Cal([_Event("dentist", "uid-1", date.today())]))
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(hass, {"name": "KlarGetCalendarEvents", "slots": []}, "en", lambda _: True)
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertIn("dentist", speech or "")
        self.assertIn("all day", (speech or "").lower())

    def test_tomorrow_empty_is_honest(self) -> None:
        hass = _Hass(_Cal([]), language="de")
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {"name": "KlarGetCalendarEvents", "slots": [{"name": "day", "value": "tomorrow"}]},
                "de",
                lambda _: True,
            )
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertEqual(speech, "Keine anstehenden Termine.")

    def test_create_calls_entity_and_speaks(self) -> None:
        hass = _Hass()
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {
                    "name": "KlarCreateCalendarEvent",
                    "slots": [
                        {"name": "summary", "value": "dentist"},
                        {"name": "day", "value": "tomorrow"},
                        {"name": "hour", "value": "15"},
                    ],
                },
                "en",
                lambda _: True,
            )
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertIn("dentist", speech or "")
        self.assertIn("tomorrow", (speech or "").lower())
        self.assertTrue(hass.cal.created)
        self.assertEqual(hass.cal.created[0]["summary"], "dentist")

    def test_need_title_skips_ha(self) -> None:
        hass = _Hass()
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {"name": "KlarCreateCalendarEvent", "slots": [{"name": "need", "value": "title"}]},
                "en",
                lambda _: True,
            )
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertTrue(speech)
        self.assertFalse(hass.cal.created)
        hass.services.async_call.assert_not_awaited()

    def test_delete_uses_entity_uid(self) -> None:
        hass = _Hass(_Cal([_Event("dentist", "uid-1", _tomorrow_at(15))]))
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {"name": "KlarDeleteCalendarEvent", "slots": [{"name": "summary", "value": "dentist"}]},
                "en",
                lambda _: True,
                "conv-1",
            )
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertTrue(speech)
        self.assertEqual(hass.cal.deleted, ["uid-1"])

    def test_delete_without_uid_skips_mutate(self) -> None:
        hass = _Hass(_Cal([_Event("dentist", "", _tomorrow_at(15))]))
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {"name": "KlarDeleteCalendarEvent", "slots": [{"name": "summary", "value": "dentist"}]},
                "en",
                lambda _: True,
            )
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertIn("identifier", (speech or "").lower())
        self.assertEqual(hass.cal.deleted, [])

    def test_ambiguous_delete_skips_ha_mutate(self) -> None:
        hass = _Hass(
            _Cal(
                [
                    _Event("dentist", "a", _tomorrow_at(15)),
                    _Event("dentist", "b", _tomorrow_at(16)),
                ]
            )
        )
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {"name": "KlarDeleteCalendarEvent", "slots": [{"name": "summary", "value": "dentist"}]},
                "en",
                lambda _: True,
            )
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertIn("which", (speech or "").lower())
        self.assertEqual(hass.cal.deleted, [])

    def test_delete_uses_session_without_summary(self) -> None:
        hass = _Hass(_Cal([_Event("dentist", "uid-1", _tomorrow_at(15))]))
        asyncio.run(
            calendar_ha.handle_calendar_intent(hass, {"name": "KlarGetCalendarEvents", "slots": []}, "en", lambda _: True, "conv-session")
        )
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {"name": "KlarDeleteCalendarEvent", "slots": []},
                "en",
                lambda _: True,
                "conv-session",
            )
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertEqual(hass.cal.deleted, ["uid-1"])

    def test_move_updates_instead_of_duplicating(self) -> None:
        hass = _Hass(_Cal([_Event("dentist", "uid-1", _tomorrow_at(15))]))
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {
                    "name": "KlarMoveCalendarEvent",
                    "slots": [
                        {"name": "summary", "value": "dentist"},
                        {"name": "day", "value": "tomorrow"},
                        {"name": "hour", "value": "16"},
                    ],
                },
                "en",
                lambda _: True,
            )
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertIn("dentist", speech or "")
        self.assertEqual(hass.cal.updated[0][0], "uid-1")
        self.assertEqual(hass.cal.deleted, [])
        self.assertEqual(hass.cal.created, [])

    def test_move_recreates_when_update_missing(self) -> None:
        cal = _Cal([_Event("dentist", "uid-1", _tomorrow_at(15))])
        cal.fail_update = True
        hass = _Hass(cal)
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {
                    "name": "KlarMoveCalendarEvent",
                    "slots": [
                        {"name": "summary", "value": "dentist"},
                        {"name": "day", "value": "tomorrow"},
                        {"name": "hour", "value": "16"},
                    ],
                },
                "en",
                lambda _: True,
            )
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertEqual(hass.cal.deleted, ["uid-1"])
        self.assertTrue(hass.cal.created)

    def test_delete_readonly_speaks_pack(self) -> None:
        cal = _Cal([_Event("dentist", "uid-1", _tomorrow_at(15))])
        cal.fail_delete = True
        hass = _Hass(cal)
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {"name": "KlarDeleteCalendarEvent", "slots": [{"name": "summary", "value": "dentist"}]},
                "en",
                lambda _: True,
            )
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertIn("cannot be changed", (speech or "").lower())

    def test_every_pack_has_spoken_calendar_lines(self) -> None:
        say = calendar_ha.calendar_say
        languages = _load("languages", "languages.py")
        self.assertEqual(set(say.SAY), set(languages.SUPPORTED_LANGUAGES))
        self.assertEqual(set(say.LLM), set(languages.SUPPORTED_LANGUAGES))
        hass = SimpleNamespace(config=SimpleNamespace(time_zone="UTC"))
        event = {"summary": "test", "start": date.today()}
        for pack in say.SAY:
            line = say.event_line(event, pack, hass)
            empty = say.fill(pack, "calendar_empty")
            self.assertTrue(line, pack)
            self.assertNotIn("00:00", line, pack)
            self.assertTrue(empty, pack)
            self.assertNotIn("I did not", empty, pack)

    def test_german_create_sounds_spoken(self) -> None:
        hass = _Hass(language="de")
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {
                    "name": "KlarCreateCalendarEvent",
                    "slots": [
                        {"name": "summary", "value": "zahnarzt"},
                        {"name": "day", "value": "tomorrow"},
                        {"name": "hour", "value": "15"},
                    ],
                },
                "de",
                lambda _: True,
            )
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertIn("zahnarzt", speech or "")
        self.assertIn("morgen", speech or "")
        self.assertIn("15 Uhr", speech or "")
        self.assertNotIn("15:00", speech or "")


if __name__ == "__main__":
    unittest.main()
