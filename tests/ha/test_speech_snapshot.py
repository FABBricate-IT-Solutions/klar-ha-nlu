#!/usr/bin/env python3
"""Stdlib tests for the post-execute speech snapshot builder."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PKG = ROOT / "custom_components" / "klar_nlu"
if str(PKG) not in sys.path:
    sys.path.insert(0, str(PKG))


def _load(name: str, rel: str):
    path = PKG / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


snapshot = _load("speech_snapshot", "speech_snapshot.py")


class SpeechSnapshotTests(unittest.TestCase):
    def test_allow_listed_attributes_keep_unknown_dropped(self) -> None:
        body = snapshot.build_snapshot(
            language="de",
            personality="butler",
            now="2026-09-05T19:22:00+02:00",
            intent={
                "name": "HassTurnOn",
                "slots": [{"name": "area", "value": "wohnzimmer"}, {"name": "", "value": "skip"}],
            },
            outcome="success",
            entities=[
                {
                    "entity_id": "climate.wohnzimmer",
                    "name": "Wohnzimmer",
                    "domain": "climate",
                    "state": "heat",
                    "area": "wohnzimmer",
                    "area_name": "Wohnzimmer",
                    "attributes": {
                        "current_temperature": 21.5,
                        "temperature": 18.0,
                        "temperature_unit": "°C",
                        "hvac_mode": "heat",
                        "secret": "drop-me",
                        "friendly_name": "nope",
                    },
                }
            ],
            calendar_events=[{"summary": "Zahnarzt", "start": "2026-09-06T09:00:00+02:00"}],
            media_queue=[{"title": "Song"}],
        )
        self.assertEqual(body["schema_version"], "1")
        self.assertEqual(body["language"], "de")
        self.assertEqual(body["now"], "2026-09-05T19:22:00+02:00")
        self.assertEqual(body["intent"]["name"], "HassTurnOn")
        self.assertEqual(body["intent"]["slots"], [{"name": "area", "value": "wohnzimmer"}])
        attrs = body["entities"][0]["attributes"]
        self.assertEqual(attrs["current_temperature"], 21.5)
        self.assertEqual(attrs["temperature"], 18.0)
        self.assertEqual(attrs["temperature_unit"], "°C")
        self.assertEqual(body["unit_system"], "metric")
        self.assertEqual(attrs["hvac_mode"], "heat")
        self.assertNotIn("secret", attrs)
        self.assertNotIn("friendly_name", attrs)
        self.assertEqual(body["calendar_events"][0]["summary"], "Zahnarzt")
        self.assertEqual(body["media_queue"][0]["title"], "Song")

    def test_caps_and_attr_length(self) -> None:
        entities = [{"entity_id": f"light.n{i}", "name": "n", "domain": "light", "state": "on"} for i in range(40)]
        body = snapshot.build_snapshot(
            language="en",
            personality="",
            now="2026-09-05T19:22:00Z",
            intent={"name": "HassTurnOff", "slots": []},
            outcome="partial",
            entities=entities,
            calendar_events=[{"summary": "x"} for _ in range(20)],
            media_queue=[{"title": "t"} for _ in range(12)],
        )
        self.assertEqual(body["personality"], "default")
        self.assertEqual(len(body["entities"]), 32)
        self.assertEqual(len(body["calendar_events"]), 16)
        self.assertEqual(len(body["media_queue"]), 8)
        long_title = snapshot.build_snapshot(
            language="de",
            personality="default",
            now="now",
            intent={"name": "HassMediaSearch"},
            outcome="error",
            entities=[
                {
                    "entity_id": "media_player.mass",
                    "name": "MASS",
                    "domain": "media_player",
                    "state": "playing",
                    "attributes": {"media_title": "x" * 300, "volume_level": 0.4},
                }
            ],
        )
        self.assertEqual(len(long_title["entities"][0]["attributes"]["media_title"]), 256)
        self.assertEqual(long_title["entities"][0]["attributes"]["volume_level"], 0.4)

    def test_hydrate_fills_empty_slot_stub(self) -> None:
        class States:
            def get(self, entity_id: str):
                if entity_id != "weather.openweathermap":
                    return None
                return types.SimpleNamespace(
                    entity_id=entity_id,
                    state="cloudy",
                    name="OpenWeatherMap",
                    attributes={"friendly_name": "OpenWeatherMap", "temperature": 27.1},
                )

        handled = types.SimpleNamespace(matched_states=[], unmatched_states=[])
        item = {"slots": [{"name": "entity_id", "value": "weather.openweathermap"}, {"name": "domain", "value": "weather"}]}
        dry = snapshot.entities_from_handled(handled, item)
        self.assertEqual(dry[0]["state"], "")
        live = snapshot.entities_from_handled(handled, item, types.SimpleNamespace(states=States()))
        self.assertEqual(live[0]["name"], "OpenWeatherMap")
        self.assertEqual(live[0]["state"], "cloudy")
        self.assertEqual(live[0]["attributes"]["temperature"], 27.1)


if __name__ == "__main__":
    unittest.main()
