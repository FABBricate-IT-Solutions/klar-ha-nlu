#!/usr/bin/env python3
"""Calendar dispatch uses HA services and pack-native speech."""

from __future__ import annotations

import asyncio
import importlib.util
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, Mock, patch

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
    modules = {"homeassistant": homeassistant, "homeassistant.core": core}
    with patch.dict(sys.modules, modules):
        _load("speech_locale", "speech_locale.py")
        _load("calendar_session", "calendar_session.py")
        return _load("klar_calendar_ha", "calendar_ha.py")


calendar_ha = _load_calendar()


class _Hass:
    def __init__(self) -> None:
        self.config = SimpleNamespace(language="ja", time_zone="UTC")
        self.services = SimpleNamespace(async_call=AsyncMock())
        self.states = SimpleNamespace(
            get=lambda entity_id: SimpleNamespace(entity_id=entity_id, state="on"),
            async_all=lambda domain: [SimpleNamespace(entity_id="calendar.home")] if domain == "calendar" else [],
        )


class CalendarDispatchTests(unittest.TestCase):
    def test_list_events_pack_speech_is_not_english(self) -> None:
        hass = _Hass()
        hass.services.async_call.return_value = {
            "calendar.home": {"events": [{"summary": "歯科", "start": {"dateTime": "2026-08-27T15:00:00+00:00"}}]}
        }
        ok, speech, error = asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {"name": "KlarGetCalendarEvents", "slots": []},
                "ja",
                lambda _: True,
            )
        )
        self.assertTrue(ok)
        self.assertIsNone(error)
        self.assertIsNotNone(speech)
        self.assertNotIn("upcoming", speech.lower())
        self.assertNotIn("I did not", speech)

    def test_create_calls_service(self) -> None:
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
        hass.services.async_call.assert_awaited()
        self.assertEqual(hass.services.async_call.await_args.args[1], "create_event")

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
        hass.services.async_call.assert_not_awaited()

    def test_delete_calls_service_with_uid(self) -> None:
        hass = _Hass()
        hass.services.async_call.return_value = {
            "calendar.home": {"events": [{"summary": "dentist", "uid": "uid-1", "start": {"dateTime": "2026-08-27T15:00:00+00:00"}}]}
        }
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
        self.assertIn("deleted", (speech or "").lower())
        self.assertEqual(hass.services.async_call.await_args.args[1], "delete_event")
        self.assertEqual(hass.services.async_call.await_args.args[2]["uid"], "uid-1")

    def test_delete_without_uid_skips_service(self) -> None:
        hass = _Hass()
        hass.services.async_call.return_value = {
            "calendar.home": {"events": [{"summary": "dentist", "start": {"dateTime": "2026-08-27T15:00:00+00:00"}}]}
        }
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
        self.assertNotEqual(hass.services.async_call.await_args.args[1], "delete_event")

    def test_ambiguous_delete_skips_ha_mutate(self) -> None:
        hass = _Hass()
        hass.services.async_call.return_value = {
            "calendar.home": {
                "events": [
                    {"summary": "dentist", "uid": "a", "start": {"dateTime": "2026-08-27T15:00:00+00:00"}},
                    {"summary": "dentist", "uid": "b", "start": {"dateTime": "2026-08-27T16:00:00+00:00"}},
                ]
            }
        }
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
        self.assertEqual(hass.services.async_call.await_args.args[1], "get_events")

    def test_delete_uses_session_without_summary(self) -> None:
        hass = _Hass()
        hass.services.async_call.return_value = {
            "calendar.home": {"events": [{"summary": "dentist", "uid": "uid-1", "start": {"dateTime": "2026-08-27T15:00:00+00:00"}}]}
        }
        asyncio.run(
            calendar_ha.handle_calendar_intent(
                hass,
                {"name": "KlarGetCalendarEvents", "slots": []},
                "en",
                lambda _: True,
                "conv-session",
            )
        )
        hass.services.async_call.reset_mock()
        hass.services.async_call.return_value = {
            "calendar.home": {"events": [{"summary": "dentist", "uid": "uid-1", "start": {"dateTime": "2026-08-27T15:00:00+00:00"}}]}
        }
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
        self.assertIn("deleted", (speech or "").lower())
        self.assertEqual(hass.services.async_call.await_args.args[1], "delete_event")
        self.assertEqual(hass.services.async_call.await_args.args[2]["uid"], "uid-1")

    def test_move_calls_update_event(self) -> None:
        hass = _Hass()
        hass.services.async_call.return_value = {
            "calendar.home": {"events": [{"summary": "dentist", "uid": "uid-1", "start": {"dateTime": "2026-08-27T15:00:00+00:00"}}]}
        }
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
        self.assertEqual(hass.services.async_call.await_args.args[1], "update_event")
        self.assertEqual(hass.services.async_call.await_args.args[2]["uid"], "uid-1")

    def test_delete_readonly_speaks_pack(self) -> None:
        hass = _Hass()

        async def call(_domain: str, service: str, *_args: object, **_kwargs: object):
            if service == "delete_event":
                raise RuntimeError("readonly")
            return {
                "calendar.home": {"events": [{"summary": "dentist", "uid": "uid-1", "start": {"dateTime": "2026-08-27T15:00:00+00:00"}}]}
            }

        hass.services.async_call = AsyncMock(side_effect=call)
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


if __name__ == "__main__":
    unittest.main()
