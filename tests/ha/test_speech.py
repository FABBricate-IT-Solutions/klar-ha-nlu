#!/usr/bin/env python3
"""Stdlib tests for spoken device names after an intent."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PKG = ROOT / "custom_components" / "klar_nlu"
if str(PKG) not in sys.path:
    sys.path.insert(0, str(PKG))


def _load(name: str, rel: str):
    path = ROOT / "custom_components" / "klar_nlu" / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


speech = _load("speech", "speech.py")
sys.modules["klar_speech"] = speech


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

    def test_clock_speech_is_one_sentence_without_seconds(self) -> None:
        from datetime import datetime

        now = datetime(2026, 8, 29, 16, 2, 55)
        self.assertEqual(speech.finish_clock_speech("Es ist 14:44:55.", "de", now), "Es ist 16:02.")
        self.assertEqual(
            speech.finish_clock_speech("Es ist 14:44:55. Die genaue Uhrzeit.", "de", now),
            "Es ist 16:02.",
        )
        self.assertEqual(speech.finish_clock_speech("It is 14:44:55.", "en", now), "It is 16:02.")
        self.assertEqual(speech.strip_clock_seconds("Es ist 14:44:55."), "Es ist 14:44.")
        self.assertEqual(speech.finish_clock_speech("Licht ist an.", "de", now), "Licht ist an.")

    def test_kitchen_turn_on_names_the_room(self) -> None:
        item = {
            "name": "HassTurnOn",
            "slots": [{"name": "entity_id", "value": "light.kuche_kuche"}],
        }
        handled = type("Handled", (), {})()
        handled.success_results = [type("Hit", (), {"name": "Licht", "id": "light.kuche_kuche"})()]
        spoken = speech.from_handled(handled, "de", item)
        self.assertIsNotNone(spoken)
        self.assertIn("Küche", spoken)
        self.assertNotIn("kuche kuche", spoken.lower())
        english = speech.from_handled(handled, "en", item)
        self.assertIsNotNone(english)
        self.assertIn("kitchen", english.lower())
        bare = speech.from_handled(None, "de", item)
        self.assertIsNotNone(bare)
        self.assertIn("Küche", bare)

    def test_kitchen_status_names_the_room(self) -> None:
        handled = _States(
            [
                _State("light.kuche_kuche", "off", "Licht"),
            ]
        )
        item = {
            "name": "HassGetState",
            "slots": [
                {"name": "area", "value": "kuche"},
                {"name": "area_name", "value": "Küche"},
            ],
        }
        spoken = speech.from_handled(handled, "de", item)
        self.assertIsNotNone(spoken)
        self.assertIn("Küche", spoken)
        self.assertIn("Licht aus", spoken)
        self.assertNotIn("In der Küche", spoken)
        french = speech.from_handled(handled, "fr", item)
        self.assertIn("Licht éteinte", french)

    def test_room_temperature_drops_satellite(self) -> None:
        handled = _States(
            [
                _State("climate.better_thermostat_wohnzimmer", "heat", "Heizung Wohnzimmer"),
                _State("sensor.satellite1_db12c8_temperature", "40.99", "Satellite1 db12c8 Temperature"),
            ]
        )
        item = {
            "name": "HassClimateGetTemperature",
            "slots": [
                {"name": "area", "value": "wohnzimmer"},
                {"name": "area_name", "value": "Wohnzimmer"},
            ],
        }
        spoken = speech.from_handled(handled, "de", item)
        self.assertIsNotNone(spoken)
        self.assertIn("Wohnzimmer", spoken)
        self.assertNotIn("Satellite", spoken)
        self.assertNotIn("40", spoken)

    def test_climate_set_speaks_degrees(self) -> None:
        item = {
            "name": "HassClimateSetTemperature",
            "slots": [
                {"name": "entity_id", "value": "climate.better_thermostat_wohnzimmer"},
                {"name": "name", "value": "Heizung Wohnzimmer"},
                {"name": "temperature", "value": "21"},
            ],
        }
        spoken = speech.from_handled(None, "de", item)
        self.assertEqual(spoken, "Heizung Wohnzimmer auf 21 Grad.")
        self.assertNotIn("nicht geklappt", spoken)

    def test_warm_white_speaks_color_not_percent(self) -> None:
        item = {
            "name": "HassLightSet",
            "slots": [
                {"name": "entity_id", "value": "light.wohnzimmer"},
                {"name": "color", "value": "warmwhite"},
            ],
        }
        spoken = speech.from_handled(None, "de", item)
        self.assertIsNotNone(spoken)
        self.assertIn("warmweiß", spoken)
        self.assertNotIn("Prozent", spoken)
        self.assertNotIn("orange", spoken)

    def test_tv_turn_on_does_not_claim_lights(self) -> None:
        item = {
            "name": "HassTurnOn",
            "slots": [
                {"name": "entity_id", "value": "media_player.wohnzimmer_tv"},
                {"name": "name", "value": "Wohnzimmer TV"},
                {"name": "area", "value": "wohnzimmer"},
            ],
        }
        spoken = speech.from_handled(None, "de", item)
        self.assertIsNotNone(spoken)
        self.assertIn("TV", spoken)
        self.assertNotIn("Licht", spoken)

    def test_light_set_speaks_color(self) -> None:
        item = {
            "name": "HassLightSet",
            "slots": [
                {"name": "area", "value": "schlafzimmer"},
                {"name": "color", "value": "red"},
            ],
        }
        spoken = speech.from_handled(None, "de", item)
        self.assertIsNotNone(spoken)
        self.assertIn("rot", spoken)
        self.assertNotIn("Prozent", spoken)
        self.assertNotIn("Stufe", spoken)

    def test_infra_needles_are_shared(self) -> None:
        needles = speech._INFRA
        self.assertIn("satellite", needles)
        self.assertIn("led_ring", needles)
        self.assertIn("cpu_temperature", needles)

    def test_media_now_playing_speech_de(self) -> None:
        handled = _States(
            [
                _State(
                    "media_player.wohnzimmer_2",
                    "playing",
                    "Wohnzimmer Soundbar",
                    media_title="Bohemian Rhapsody",
                    media_artist="Queen",
                ),
            ]
        )
        item = {
            "name": "HassGetState",
            "slots": [
                {"name": "entity_id", "value": "media_player.wohnzimmer_2"},
                {"name": "media_status", "value": "now_playing"},
            ],
        }
        spoken = speech.from_handled(handled, "de", item)
        self.assertIsNotNone(spoken)
        self.assertIn("Bohemian Rhapsody", spoken)
        self.assertIn("Queen", spoken)

    def test_media_volume_speech_en(self) -> None:
        state = _State("media_player.wohnzimmer_2", "playing", "Living Room", volume_level=0.3, is_volume_muted=True)
        spoken = speech.media_state_speech(state, "volume", "en")
        self.assertIn("30 percent", spoken)
        self.assertIn("muted", spoken)

    def test_media_status_without_player_does_not_describe_room_light(self) -> None:
        handled = _States([_State("light.wohnzimmer", "on", "Wohnzimmer Licht")])
        item = {
            "name": "HassGetState",
            "slots": [
                {"name": "area", "value": "wohnzimmer"},
                {"name": "media_status", "value": "now_playing"},
            ],
        }
        spoken = speech.from_handled(handled, "de", item)
        self.assertIsNone(spoken)

    def test_queue_speech_empty(self) -> None:
        state = _State("media_player.wohnzimmer_2", "idle", "Living Room")
        self.assertEqual(
            speech.queue_speech({"items": []}, state, "en"),
            "The queue is empty.",
        )

    def test_queue_speech_one_item(self) -> None:
        state = _State("media_player.wohnzimmer_2", "idle", "Living Room")
        spoken = speech.queue_speech({"items": [{"name": "B"}]}, state, "en")
        self.assertEqual(spoken, "Next is B.")

    def test_queue_speech_multiple_items(self) -> None:
        state = _State("media_player.wohnzimmer_2", "playing", "Living Room", media_title="A", media_artist="Artist")
        response = {"items": [{"name": "A", "artist": "Artist"}, {"name": "B"}, {"name": "C"}]}
        spoken = speech.queue_speech(response, state, "en")
        self.assertIn("Now playing A by Artist", spoken)
        self.assertIn("Next is B", spoken)
        self.assertIn("Then C", spoken)
        self.assertEqual(spoken.count("A by Artist"), 1)

    def test_set_volume_speech_is_specific(self) -> None:
        item = {
            "name": "HassSetVolume",
            "slots": [
                {"name": "entity_id", "value": "media_player.wohnzimmer"},
                {"name": "name", "value": "Wohnzimmer"},
                {"name": "volume_level", "value": "35"},
            ],
        }
        spoken = speech.from_handled(None, "de", item)
        self.assertEqual(spoken, "Die Lautstärke von Wohnzimmer ist auf 35 Prozent.")

    def test_relative_volume_speech_names_direction(self) -> None:
        item = {
            "name": "HassSetVolumeRelative",
            "slots": [
                {"name": "entity_id", "value": "media_player.wohnzimmer"},
                {"name": "name", "value": "Wohnzimmer"},
                {"name": "volume_step", "value": "down"},
            ],
        }
        spoken = speech.from_handled(None, "de", item)
        self.assertIn("verringert", spoken or "")
        self.assertNotIn("HassSetVolumeRelative", spoken or "")

    def test_transport_actions_have_clean_speech(self) -> None:
        expected = {
            "HassMediaPause": "pausiert",
            "HassMediaNext": "nächste",
            "HassMediaPrevious": "vorherige",
        }
        for name, phrase in expected.items():
            with self.subTest(name=name):
                item = {
                    "name": name,
                    "slots": [
                        {"name": "entity_id", "value": "media_player.wohnzimmer"},
                        {"name": "name", "value": "Wohnzimmer"},
                    ],
                }
                spoken = speech.from_handled(None, "de", item)
                self.assertIn(phrase, spoken or "")
                self.assertNotIn(name, spoken or "")


class _State:
    def __init__(self, entity_id: str, state: str, name: str, **attrs: object) -> None:
        self.entity_id = entity_id
        self.state = state
        self.name = name
        self.attributes = {"friendly_name": name, **attrs}


class _States:
    def __init__(self, states: list[_State]) -> None:
        self.matched_states = states
        self.response_type = "query_answer"


if __name__ == "__main__":
    unittest.main()
