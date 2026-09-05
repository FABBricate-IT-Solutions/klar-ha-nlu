#!/usr/bin/env python3
"""Floor GetState expands to one spoken room status per area."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = "klar_floor_query_test"


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


def _load_floor_query() -> types.ModuleType:
    homeassistant = _module("homeassistant")
    core = types.ModuleType("homeassistant.core")
    core.HomeAssistant = object
    helpers = _module("homeassistant.helpers")
    area_registry = types.ModuleType("homeassistant.helpers.area_registry")
    area_registry.async_get = Mock()
    entity_registry = types.ModuleType("homeassistant.helpers.entity_registry")
    entity_registry.async_get = Mock()
    floor_registry = types.ModuleType("homeassistant.helpers.floor_registry")
    floor_registry.async_get = Mock()
    device_registry = types.ModuleType("homeassistant.helpers.device_registry")
    device_registry.async_get = Mock()
    helpers.area_registry = area_registry
    helpers.entity_registry = entity_registry
    helpers.floor_registry = floor_registry
    helpers.device_registry = device_registry
    package = _module(PACKAGE)
    sys.modules.update(
        {
            "homeassistant": homeassistant,
            "homeassistant.core": core,
            "homeassistant.helpers": helpers,
            "homeassistant.helpers.area_registry": area_registry,
            "homeassistant.helpers.entity_registry": entity_registry,
            "homeassistant.helpers.floor_registry": floor_registry,
            "homeassistant.helpers.device_registry": device_registry,
            PACKAGE: package,
        }
    )
    _load(f"{PACKAGE}.speech_place", "speech_place.py")
    _load(f"{PACKAGE}.clock_speech", "clock_speech.py")
    _load(f"{PACKAGE}.speech_locale", "speech_locale.py")
    _load(f"{PACKAGE}.speech", "speech.py")
    _load(f"{PACKAGE}.speech_status_device", "speech_status_device.py")
    _load(f"{PACKAGE}.speech_status", "speech_status.py")
    _load(f"{PACKAGE}.intents", "intents.py")
    return _load(f"{PACKAGE}.floor_query", "floor_query.py")


floor_query = _load_floor_query()


class _State:
    def __init__(self, entity_id: str, state: str, name: str, **attrs: object) -> None:
        self.entity_id = entity_id
        self.state = state
        self.name = name
        self.attributes = {"friendly_name": name, **attrs}


class _States:
    def __init__(self, *states: _State) -> None:
        self._states = list(states)

    def async_all(self, domain: str | None = None) -> list[_State]:
        if domain is None:
            return list(self._states)
        return [state for state in self._states if state.entity_id.startswith(f"{domain}.")]


class FloorQueryTests(unittest.TestCase):
    def test_rooms_status_names_each_area(self) -> None:
        spoken = floor_query.rooms_status_speech(
            [
                ("Wohnzimmer", [_State("light.wohnzimmer", "on", "Wohnzimmer")]),
                ("Küche", [_State("light.kuche", "off", "Küche")]),
            ],
            "de",
        )
        self.assertIn("Wohnzimmer", spoken)
        self.assertIn("Küche", spoken)
        self.assertIn("an", spoken)
        self.assertIn("aus", spoken)

    def test_rooms_status_skips_infra_only_rooms(self) -> None:
        spoken = floor_query.rooms_status_speech(
            [
                ("Flur", [_State("sensor.satellite1_temperature", "40", "Satellite1 Temperature")]),
                ("Wohnzimmer", [_State("light.wohnzimmer", "on", "Wohnzimmer")]),
            ],
            "de",
        )
        self.assertIn("Wohnzimmer", spoken)
        self.assertNotIn("Satellite", spoken)
        self.assertNotIn("Flur", spoken)

    def test_floor_status_collects_areas_on_floor(self) -> None:
        living = _State("light.wohnzimmer", "on", "Wohnzimmer")
        kitchen = _State("light.kuche", "off", "Küche")
        weather = _State("weather.openweathermap", "sunny", "OpenWeather")
        hass = SimpleNamespace(states=_States(living, kitchen, weather))
        living_area = SimpleNamespace(id="wohnzimmer", name="Wohnzimmer", floor_id="wohnung")
        kitchen_area = SimpleNamespace(id="kuche", name="Küche", floor_id="wohnung")
        technik = SimpleNamespace(id="technik", name="Technik", floor_id=None)
        floor = SimpleNamespace(floor_id="wohnung", name="Wohnung", aliases=["zuhause"])
        entries = {
            living.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
            kitchen.entity_id: SimpleNamespace(area_id="kuche", device_id=None),
            weather.entity_id: SimpleNamespace(area_id="technik", device_id=None),
        }
        with (
            patch.object(floor_query, "resolve_floor", return_value=floor),
            patch.object(floor_query, "areas_on_floor", return_value=[kitchen_area, living_area, technik]),
            patch("homeassistant.helpers.entity_registry.async_get", return_value=SimpleNamespace(async_get=entries.get)),
        ):
            rooms = floor_query.floor_status_rooms(hass, "wohnung", "", lambda _id: True)
        names = [name for name, _states in rooms]
        self.assertEqual(names, ["Küche", "Wohnzimmer"])
        spoken = floor_query.rooms_status_speech(rooms, "de")
        self.assertIn("Küche", spoken)
        self.assertIn("Wohnzimmer", spoken)
        self.assertNotIn("OpenWeather", spoken)
        self.assertNotIn("Technik", spoken)

    def test_floor_status_collects_sockets_and_presence(self) -> None:
        plug = _State("switch.wz_plug", "on", "Stecker")
        occ = _State("binary_sensor.wz_occ", "off", "Präsenz", device_class="occupancy")
        hass = SimpleNamespace(states=_States(plug, occ))
        living = SimpleNamespace(id="wohnzimmer", name="Wohnzimmer", floor_id="wohnung")
        floor = SimpleNamespace(floor_id="wohnung", name="Wohnung", aliases=[])
        entries = {
            plug.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
            occ.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
        }
        with (
            patch.object(floor_query, "resolve_floor", return_value=floor),
            patch.object(floor_query, "areas_on_floor", return_value=[living]),
            patch("homeassistant.helpers.entity_registry.async_get", return_value=SimpleNamespace(async_get=entries.get)),
        ):
            rooms = floor_query.floor_status_rooms(hass, "wohnung", "", lambda _id: True)
        spoken = floor_query.rooms_status_speech(rooms, "de")
        self.assertIn("Stecker an", spoken)
        self.assertNotIn("Steckdose", spoken)
        self.assertIn("niemand da", spoken)

    def test_area_facts_follow_fixed_order(self) -> None:
        spoken = floor_query.rooms_status_speech(
            [
                (
                    "Wohnzimmer",
                    [
                        _State("vacuum.r2d2", "paused", "R2D2"),
                        _State("sensor.wz_lux", "40", "Helligkeit", device_class="illuminance"),
                        _State("climate.wz", "heat", "Heizung", current_temperature=22.8),
                        _State("binary_sensor.wz_occ", "on", "Präsenz", device_class="occupancy"),
                        _State("switch.wz_plug", "off", "Stecker", device_class="outlet"),
                        _State("light.wohnzimmer", "on", "Wohnzimmer Licht"),
                    ],
                )
            ],
            "de",
        )
        self.assertTrue(spoken.startswith("Wohnzimmer."))
        self.assertLess(spoken.index("Wohnzimmer Licht an"), spoken.index("Stecker aus"))
        self.assertLess(spoken.index("Stecker aus"), spoken.index("jemand da"))
        self.assertLess(spoken.index("jemand da"), spoken.index("22,8 Grad"))
        self.assertLess(spoken.index("22,8 Grad"), spoken.index("40 Lux"))
        self.assertLess(spoken.index("40 Lux"), spoken.index("R2D2"))
        self.assertLess(spoken.index("R2D2"), spoken.index("Heizung heizt"))

    def test_every_speech_pack_has_status_words(self) -> None:
        locale = sys.modules[f"{PACKAGE}.speech_locale"]
        states = [_State("light.room", "on", "Room")]
        for pack in locale.SPEECH_PACKS:
            spoken = floor_query.rooms_status_speech([("Room", states)], pack)
            self.assertTrue(spoken.startswith("Room"), msg=pack)
            self.assertTrue("." in spoken or "。" in spoken, msg=pack)

    def test_status_uses_local_words(self) -> None:
        states = [_State("light.salon", "on", "Salon"), _State("switch.prise", "off", "Prise")]
        french = floor_query.rooms_status_speech([("Salon", states)], "fr")
        english = floor_query.rooms_status_speech([("Living room", states)], "en")
        self.assertIn("Salon allumée", french)
        self.assertIn("Prise éteinte", french)
        self.assertNotIn("light on", french)
        self.assertIn("Salon on", english)
        self.assertIn("Prise off", english)

    def test_kannada_and_malayalam_use_native_script(self) -> None:
        states = [_State("light.room", "on", "Room")]
        kannada = floor_query.rooms_status_speech([("Room", states)], "kn")
        malayalam = floor_query.rooms_status_speech([("Room", states)], "ml")
        self.assertIn("ಆನ್", kannada)
        self.assertNotIn("belaku", kannada)
        self.assertIn("ഓൺ", malayalam)
        self.assertNotIn("vilakku", malayalam)

    def test_device_states_follow_speech_pack(self) -> None:
        rooms = [("Wohnzimmer", [_State("vacuum.r2d2", "paused", "R2D2")])]
        self.assertIn("pausiert", floor_query.rooms_status_speech(rooms, "de"))
        self.assertIn("paused", floor_query.rooms_status_speech(rooms, "en"))
        french = floor_query.rooms_status_speech(rooms, "fr")
        self.assertIn("en pause", french)
        self.assertNotIn("paused", french)

    def test_area_status_matches_floor_clause(self) -> None:
        living = _State("light.wohnzimmer", "on", "Wohnzimmer")
        plug = _State("switch.wz_plug", "off", "Stecker")
        hass = SimpleNamespace(states=_States(living, plug))
        area = SimpleNamespace(id="wohnzimmer", name="Wohnzimmer", floor_id="wohnung")
        entries = {
            living.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
            plug.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
        }
        with (
            patch.object(floor_query, "resolve_area", return_value=area),
            patch("homeassistant.helpers.entity_registry.async_get", return_value=SimpleNamespace(async_get=entries.get)),
        ):
            rooms = floor_query.area_status_rooms(hass, "wohnzimmer", "", lambda _id: True)
        spoken = floor_query.rooms_status_speech(rooms, "de")
        self.assertEqual(spoken, "Wohnzimmer. Wohnzimmer an. Stecker aus.")
        french = floor_query.rooms_status_speech(rooms, "fr")
        self.assertIn("Wohnzimmer allumée", french)
        self.assertIn("Stecker éteinte", french)

    def test_place_get_state_uses_area_not_ha_fallback(self) -> None:
        living = _State("light.wohnzimmer", "on", "Wohnzimmer")
        hass = SimpleNamespace(states=_States(living))
        area = SimpleNamespace(id="wohnzimmer", name="Wohnzimmer", floor_id="wohnung")
        entries = {living.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None)}
        with (
            patch.object(floor_query, "resolve_area", return_value=area),
            patch("homeassistant.helpers.entity_registry.async_get", return_value=SimpleNamespace(async_get=entries.get)),
        ):
            spoken = floor_query.place_get_state(
                hass,
                {"area": {"value": "wohnzimmer"}},
                "de",
                lambda _id: True,
            )
        self.assertEqual(spoken, "Wohnzimmer. Wohnzimmer an.")
        skipped = floor_query.place_get_state(
            hass,
            {"area": {"value": "wohnzimmer"}, "device_class": {"value": "temperature"}},
            "de",
            lambda _id: True,
        )
        self.assertEqual(skipped, "")

    def test_empty_floor_speaks_no_devices(self) -> None:
        hass = SimpleNamespace(states=_States())
        floor = SimpleNamespace(floor_id="wohnung", name="Wohnung", aliases=[])
        with (
            patch.object(floor_query, "resolve_floor", return_value=floor),
            patch.object(floor_query, "areas_on_floor", return_value=[]),
        ):
            rooms = floor_query.place_status_rooms(
                hass, {"floor": {"value": "wohnung"}}, lambda _id: True
            )
        self.assertEqual(rooms, [])

    def test_domain_filter_keeps_only_lights(self) -> None:
        living = _State("light.wohnzimmer", "on", "Wohnzimmer")
        plug = _State("switch.wz_plug", "off", "Stecker")
        cover = _State("cover.wz_rollo", "opening", "Rollo")
        hass = SimpleNamespace(states=_States(living, plug, cover))
        area = SimpleNamespace(id="wohnzimmer", name="Wohnzimmer", floor_id="wohnung")
        floor = SimpleNamespace(floor_id="wohnung", name="Wohnung", aliases=[])
        entries = {
            living.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
            plug.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
            cover.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
        }
        with (
            patch.object(floor_query, "resolve_floor", return_value=floor),
            patch.object(floor_query, "areas_on_floor", return_value=[area]),
            patch("homeassistant.helpers.entity_registry.async_get", return_value=SimpleNamespace(async_get=entries.get)),
        ):
            spoken = floor_query.place_get_state(
                hass,
                {"floor": {"value": "wohnung"}, "domain": {"value": "light"}},
                "de",
                lambda _id: True,
            )
        self.assertEqual(spoken, "Wohnzimmer. Wohnzimmer an.")
        self.assertNotIn("Stecker", spoken)
        self.assertNotIn("Rollo", spoken)

    def test_wohnungsstatus_names_each_room(self) -> None:
        living = _State("light.wohnzimmer", "on", "Wohnzimmer")
        kitchen = _State("light.kuche", "off", "Küche")
        vacuum = _State("vacuum.r2d2", "error", "R2D2")
        cover = _State("cover.wz_rollo", "opening", "Rollo")
        extra = _State("fan.wz", "fan_only", "Lüfter")
        climate = _State("climate.wz", "auto", "Heizung")
        hass = SimpleNamespace(states=_States(living, kitchen, vacuum, cover, extra, climate))
        living_area = SimpleNamespace(id="wohnzimmer", name="Wohnzimmer", floor_id="wohnung")
        kitchen_area = SimpleNamespace(id="kuche", name="Küche", floor_id="wohnung")
        floor = SimpleNamespace(floor_id="wohnung", name="Wohnung", aliases=["zuhause"])
        entries = {
            living.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
            kitchen.entity_id: SimpleNamespace(area_id="kuche", device_id=None),
            vacuum.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
            cover.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
            extra.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
            climate.entity_id: SimpleNamespace(area_id="wohnzimmer", device_id=None),
        }
        with (
            patch.object(floor_query, "resolve_floor", return_value=floor),
            patch.object(floor_query, "areas_on_floor", return_value=[kitchen_area, living_area]),
            patch("homeassistant.helpers.entity_registry.async_get", return_value=SimpleNamespace(async_get=entries.get)),
        ):
            spoken = floor_query.place_get_state(
                hass, {"floor": {"value": "Wohnung"}}, "de", lambda _id: True
            )
        self.assertTrue(spoken.startswith("Küche."))
        self.assertIn("Küche. Küche aus.", spoken)
        self.assertIn("Wohnzimmer. Wohnzimmer an.", spoken)
        self.assertIn("R2D2 Fehler", spoken)
        self.assertIn("Rollo öffnet", spoken)
        self.assertIn("Lüfter nur Lüfter", spoken)
        self.assertIn("Heizung automatisch", spoken)

    def test_named_devices_are_not_swallowed(self) -> None:
        spoken = floor_query.rooms_status_speech(
            [
                (
                    "Schlafzimmer",
                    [
                        _State("light.kugel", "on", "Kugel"),
                        _State("light.decke", "on", "Schlafzimmer Licht"),
                        _State("switch.tv", "on", "Schlafzimmer TV"),
                        _State("climate.ac", "off", "Schlafzimmer Klima", current_temperature=25.5),
                        _State("sensor.temp", "23.6", "Heizung Temperatur", device_class="temperature"),
                    ],
                )
            ],
            "de",
        )
        self.assertIn("Kugel an", spoken)
        self.assertIn("Schlafzimmer Licht an", spoken)
        self.assertIn("Schlafzimmer TV an", spoken)
        self.assertIn("Schlafzimmer Klima aus", spoken)
        self.assertIn("23,6 Grad", spoken)
        self.assertNotIn("2 Lichter", spoken)
        self.assertNotIn("Steckdose", spoken)
        self.assertNotIn("Heizung Temperatur", spoken)

    def test_all_other_devices_are_spoken(self) -> None:
        states = [
            _State(f"vacuum.bot{index}", "docked", f"Bot{index}")
            for index in range(6)
        ]
        spoken = floor_query.rooms_status_speech([("Wohnzimmer", states)], "de")
        for index in range(6):
            self.assertIn(f"Bot{index} an der Station", spoken)

    def test_floor_alias_resolves(self) -> None:
        floor = SimpleNamespace(floor_id="wohnung", name="Wohnung", aliases=["zuhause", "home"])
        registry = SimpleNamespace(
            async_get_floor=lambda _key: None,
            floors={"wohnung": floor},
        )
        with patch("homeassistant.helpers.floor_registry.async_get", return_value=registry):
            self.assertIs(floor_query.resolve_floor(object(), "zuhause"), floor)
            self.assertIs(floor_query.resolve_floor(object(), "Wohnung"), floor)


if __name__ == "__main__":
    unittest.main()
