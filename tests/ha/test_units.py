#!/usr/bin/env python3
"""Home Assistant unit system mapping (H-05)."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[2]


def _load():
    path = ROOT / "custom_components" / "klar_nlu" / "units.py"
    spec = importlib.util.spec_from_file_location("klar_units_test", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


units = _load()


class UnitsTests(unittest.TestCase):
    def test_fahrenheit_home_is_imperial(self) -> None:
        hass = SimpleNamespace(config=SimpleNamespace(units=SimpleNamespace(temperature_unit="°F")))
        self.assertEqual(units.ha_unit_system(hass), "imperial")

    def test_celsius_home_is_metric(self) -> None:
        hass = SimpleNamespace(config=SimpleNamespace(units=SimpleNamespace(temperature_unit="°C")))
        self.assertEqual(units.ha_unit_system(hass), "metric")

    def test_missing_config_is_metric(self) -> None:
        self.assertEqual(units.ha_unit_system(SimpleNamespace()), "metric")


if __name__ == "__main__":
    unittest.main()
