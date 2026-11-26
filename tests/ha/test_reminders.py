#!/usr/bin/env python3
"""Reminder delegation stays off device commands and off Klar's own reminder intents."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[2]
KLAR = ROOT / "custom_components" / "klar_nlu"


def _ensure_pkg() -> None:
    if "klar_nlu" in sys.modules:
        return
    pkg = types.ModuleType("klar_nlu")
    pkg.__path__ = [str(KLAR)]
    sys.modules["klar_nlu"] = pkg


def _stub_homeassistant() -> None:
    """passthrough imports HA at module top; stdlib tests have no install."""
    if "homeassistant.components.conversation" in sys.modules:
        return
    homeassistant = types.ModuleType("homeassistant")
    components = types.ModuleType("homeassistant.components")
    conversation = types.ModuleType("homeassistant.components.conversation")
    conversation.async_get_agent = lambda *_a, **_k: None  # type: ignore[attr-defined]
    homeassistant.components = components  # type: ignore[attr-defined]
    components.conversation = conversation  # type: ignore[attr-defined]
    sys.modules["homeassistant"] = homeassistant
    sys.modules["homeassistant.components"] = components
    sys.modules["homeassistant.components.conversation"] = conversation


def _load(name: str):
    _ensure_pkg()
    _stub_homeassistant()
    full = f"klar_nlu.{name}"
    if full in sys.modules:
        return sys.modules[full]
    path = KLAR / f"{name}.py"
    spec = importlib.util.spec_from_file_location(full, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[full] = mod
    spec.loader.exec_module(mod)
    return mod


_load("passthrough")
reminders = _load("reminders")
zones = _load("time_zone")


class ReminderDelegateTests(unittest.TestCase):
    def test_device_plan_for_a_reminder_sentence_is_delegated(self) -> None:
        intents = [{"name": "HassLightSet", "slots": []}]
        self.assertTrue(reminders.should_delegate("remind me in 20 minutes to feed the cats", intents))

    def test_parsed_reminder_intent_is_delegated(self) -> None:
        # H-12: SetReminder* is not a Home Assistant intent; Klar must hand the turn over.
        intents = [{"name": "SetReminderRelative", "slots": []}]
        self.assertTrue(reminders.should_delegate("remind me in 20 minutes to feed the cats", intents))

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

    def test_reminder_delegate_uses_shared_default_agent(self) -> None:
        src = (KLAR / "reminders.py").read_text(encoding="utf-8")
        self.assertIn("async_delegate_default_agent", src)


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
