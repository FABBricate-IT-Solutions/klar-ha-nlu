#!/usr/bin/env python3
"""Registered HA intent pass-through stays closed for unknown names."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]


def _module(name: str) -> types.ModuleType:
    module = types.ModuleType(name)
    module.__path__ = []
    return module


def _load() -> types.ModuleType:
    homeassistant = _module("homeassistant")
    core = types.ModuleType("homeassistant.core")
    core.HomeAssistant = object
    helpers = _module("homeassistant.helpers")
    area_registry = types.ModuleType("homeassistant.helpers.area_registry")
    area_registry.async_get = lambda _hass: None
    helpers.area_registry = area_registry
    modules = {
        "homeassistant": homeassistant,
        "homeassistant.core": core,
        "homeassistant.helpers": helpers,
        "homeassistant.helpers.area_registry": area_registry,
    }
    path = ROOT / "custom_components" / "klar_nlu" / "intents.py"
    spec = importlib.util.spec_from_file_location("klar_intents_test", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    with patch.dict(sys.modules, modules):
        spec.loader.exec_module(module)
    return module


intents = _load()


class IntentPassThroughTests(unittest.TestCase):
    def test_builtin_stays_allowed(self) -> None:
        out = intents.home_intents([{"name": "HassTurnOn", "slots": []}], set())
        self.assertEqual(out[0]["name"], "HassTurnOn")

    def test_registered_custom_passes(self) -> None:
        out = intents.home_intents([{"name": "GuestMode", "slots": []}], {"GuestMode"})
        self.assertEqual(out[0]["name"], "GuestMode")

    def test_unknown_without_handler_is_dropped(self) -> None:
        out = intents.home_intents([{"name": "NotRegistered", "slots": []}], set())
        self.assertEqual(out, [])

    def test_lab_execute_keeps_unfiltered_plan(self) -> None:
        raw = [{"name": "KlarGetCalendarEvents", "slots": [{"name": "day", "value": "tomorrow"}]}]
        self.assertEqual(intents.keep_lab_plan(raw, set())[0]["name"], "KlarGetCalendarEvents")
        unknown = [{"name": "NotRegistered", "slots": []}]
        self.assertEqual(intents.keep_lab_plan(unknown, set()), unknown)
        self.assertEqual(intents.home_intents(unknown, set()), [])

    def test_get_state_without_target_is_dropped(self) -> None:
        out = intents.home_intents([{"name": "HassGetState", "slots": []}], set())
        self.assertEqual(out, [])

    def test_umlaut_area_aliases_match_ha_slug(self) -> None:
        self.assertTrue(intents._umlaut_eq("kueche", "kuche"))
        self.assertTrue(intents._area_hit("Küche", "kueche"))
        self.assertTrue(intents._area_hit("kuche", "Küche"))
        self.assertTrue(intents._area_hit("Büro", "buro"))
        self.assertFalse(intents._area_hit("Wohnzimmer", "kuche"))


if __name__ == "__main__":
    unittest.main()
