#!/usr/bin/env python3
"""Stdlib tests for spoken device names after an intent."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def _load(name: str, rel: str):
    path = ROOT / "custom_components" / "klar_nlu" / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


speech = _load("klar_speech", "speech.py")


class SpeechTests(unittest.TestCase):
    def test_pretty_where_uses_friendly_name(self) -> None:
        item = {
            "name": "HassTurnOn",
            "slots": [
                {"name": "entity_id", "value": "light.schlafzimmer"},
                {"name": "name", "value": "Kugel"},
            ],
        }
        where = speech._pretty_where(None, item, "de")
        self.assertIn("Kugel", where)
        self.assertNotIn("light.", where)
        self.assertNotIn("schlafzimmer", where.lower())

    def test_pretty_where_never_speaks_entity_id(self) -> None:
        item = {
            "name": "HassTurnOn",
            "slots": [{"name": "entity_id", "value": "light.schlafzimmer"}],
        }
        where = speech._pretty_where(None, item, "de")
        self.assertNotIn("light.", where)

    def test_from_handled_with_name_slot(self) -> None:
        item = {
            "name": "HassTurnOn",
            "slots": [
                {"name": "entity_id", "value": "light.schlafzimmer"},
                {"name": "name", "value": "Kugel"},
            ],
        }
        spoken = speech.from_handled(None, "de", item)
        self.assertIsNotNone(spoken)
        self.assertIn("Kugel", spoken)
        self.assertNotIn("light.", spoken)


if __name__ == "__main__":
    unittest.main()
