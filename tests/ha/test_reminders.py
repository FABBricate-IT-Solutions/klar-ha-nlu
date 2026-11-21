#!/usr/bin/env python3
"""Reminder delegation stays off device commands and off Klar's own reminder intents."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[2]


def _load(name: str):
    path = ROOT / "custom_components" / "klar_nlu" / name
    spec = importlib.util.spec_from_file_location(f"klar_{name}", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


reminders = _load("reminders.py")
zones = _load("time_zone.py")


class ReminderDelegateTests(unittest.TestCase):
    def test_device_plan_for_a_reminder_sentence_is_delegated(self) -> None:
        intents = [{"name": "HassLightSet", "slots": []}]
        self.assertTrue(reminders.should_delegate("remind me in 20 minutes to feed the cats", intents))

    def test_parsed_reminder_intent_stays_with_klar(self) -> None:
        intents = [{"name": "SetReminderRelative", "slots": []}]
        self.assertFalse(reminders.should_delegate("remind me in 20 minutes to feed the cats", intents))

    def test_ordinary_play_is_not_a_reminder(self) -> None:
        self.assertFalse(reminders.should_delegate("play katelyn tarver in the bedroom", []))

    def test_error_from_the_default_agent_is_not_accepted(self) -> None:
        result = SimpleNamespace(response=SimpleNamespace(response_type="error", speech={"plain": {"speech": "Sorry"}}))
        self.assertFalse(reminders.delegation_accepted(result))

    def test_action_done_with_speech_is_accepted(self) -> None:
        result = SimpleNamespace(
            response=SimpleNamespace(response_type="action_done", speech={"plain": {"speech": "Reminder set for 9:16 PM: feed the cats."}})
        )
        self.assertTrue(reminders.delegation_accepted(result))


class TimeZoneTests(unittest.TestCase):
    def test_home_assistant_zone_is_kept(self) -> None:
        self.assertEqual(zones.time_zone_name("America/New_York"), "America/New_York")

    def test_junk_zone_is_dropped(self) -> None:
        self.assertEqual(zones.time_zone_name("America/New York"), "")
        self.assertEqual(zones.time_zone_name("../etc/passwd"), "")

    def test_sync_only_when_the_engine_zone_differs(self) -> None:
        self.assertFalse(zones.time_zone_needs_sync({"time_zone": "America/New_York"}, "America/New_York"))
        self.assertTrue(zones.time_zone_needs_sync({"time_zone": "UTC"}, "America/New_York"))
        self.assertTrue(zones.time_zone_needs_sync({}, "Europe/Berlin"))


if __name__ == "__main__":
    unittest.main()
