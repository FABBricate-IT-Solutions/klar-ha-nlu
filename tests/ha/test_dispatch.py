#!/usr/bin/env python3
"""Stdlib tests for hardened Home Assistant media dispatch."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, Mock, patch

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = "klar_dispatch_test"


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


def _load_dispatch() -> types.ModuleType:
    homeassistant = _module("homeassistant")
    components = _module("homeassistant.components")
    conversation = types.ModuleType("homeassistant.components.conversation")
    conversation.ConversationInput = object
    core = types.ModuleType("homeassistant.core")
    core.HomeAssistant = object
    helpers = _module("homeassistant.helpers")
    intent = types.ModuleType("homeassistant.helpers.intent")
    intent.async_handle = AsyncMock()
    area_registry = types.ModuleType("homeassistant.helpers.area_registry")
    area_registry.async_get = Mock()
    entity_registry = types.ModuleType("homeassistant.helpers.entity_registry")
    entity_registry.async_get = Mock()
    floor_registry = types.ModuleType("homeassistant.helpers.floor_registry")
    floor_registry.async_get = Mock()
    device_registry = types.ModuleType("homeassistant.helpers.device_registry")
    device_registry.async_get = Mock()
    helpers.intent = intent
    helpers.area_registry = area_registry
    helpers.entity_registry = entity_registry
    helpers.floor_registry = floor_registry
    helpers.device_registry = device_registry
    package = _module(PACKAGE)
    modules = {
        "homeassistant": homeassistant,
        "homeassistant.components": components,
        "homeassistant.components.conversation": conversation,
        "homeassistant.core": core,
        "homeassistant.helpers": helpers,
        "homeassistant.helpers.intent": intent,
        "homeassistant.helpers.area_registry": area_registry,
        "homeassistant.helpers.entity_registry": entity_registry,
        "homeassistant.helpers.floor_registry": floor_registry,
        "homeassistant.helpers.device_registry": device_registry,
        PACKAGE: package,
    }
    with patch.dict(sys.modules, modules):
        _load(f"{PACKAGE}.clock_speech", "clock_speech.py")
        _load(f"{PACKAGE}.speech_place", "speech_place.py")
        _load("clock_speech", "clock_speech.py")
        _load("speech_place", "speech_place.py")
        _load(f"{PACKAGE}.speech", "speech.py")
        _load(f"{PACKAGE}.speech_locale", "speech_locale.py")
        _load(f"{PACKAGE}.calendar_session", "calendar_session.py")
        _load("calendar_session", "calendar_session.py")
        package.calendar_entity = _load(f"{PACKAGE}.calendar_entity", "calendar_entity.py")
        package.calendar_say = _load(f"{PACKAGE}.calendar_say", "calendar_say.py")
        _load("calendar_entity", "calendar_entity.py")
        _load("calendar_say", "calendar_say.py")
        _load(f"{PACKAGE}.calendar_ha", "calendar_ha.py")
        _load(f"{PACKAGE}.languages", "languages.py")
        _load(f"{PACKAGE}.const", "const.py")
        _load(f"{PACKAGE}.lang_select", "lang_select.py")
        _load(f"{PACKAGE}.intents", "intents.py")
        _load(f"{PACKAGE}.speech_status_device", "speech_status_device.py")
        _load(f"{PACKAGE}.speech_status", "speech_status.py")
        _load(f"{PACKAGE}.floor_query", "floor_query.py")
        _load(f"{PACKAGE}.dispatch_result", "dispatch_result.py")
        _load(f"{PACKAGE}.refine", "refine.py")
        _load(f"{PACKAGE}.stream", "stream.py")
        _load(f"{PACKAGE}.engine_llm", "engine_llm.py")
        _load(f"{PACKAGE}.speech_snapshot", "speech_snapshot.py")
        _load(f"{PACKAGE}.speech_render", "speech_render.py")
        media = _load(f"{PACKAGE}.dispatch_media", "dispatch_media.py")
        return _load(f"{PACKAGE}.dispatch", "dispatch.py"), media


dispatch, dispatch_media = _load_dispatch()


class _State:
    def __init__(self, entity_id: str, state: str = "playing", **attrs: object) -> None:
        self.entity_id = entity_id
        self.state = state
        self.name = str(attrs.get("friendly_name") or entity_id)
        self.attributes = dict(attrs)


class _States:
    def __init__(self, *states: _State) -> None:
        self._states = {state.entity_id: state for state in states}

    def get(self, entity_id: str) -> _State | None:
        return self._states.get(entity_id)

    def async_all(self, domain: str | None = None) -> list[_State]:
        if domain is None:
            return list(self._states.values())
        return [state for state in self._states.values() if state.entity_id.startswith(f"{domain}.")]


def _hass(*states: _State) -> SimpleNamespace:
    return SimpleNamespace(states=_States(*states), services=SimpleNamespace(async_call=AsyncMock()))


def _input(text: str = "test") -> SimpleNamespace:
    return SimpleNamespace(text=text, context=object(), language="de")


def _item(name: str, **slots: object) -> dict[str, object]:
    return {
        "name": name,
        "slots": [{"name": key, "value": value} for key, value in slots.items()],
    }


class DispatchTests(unittest.IsolatedAsyncioTestCase):
    def setUp(self) -> None:
        dispatch.intent.async_handle = AsyncMock()
        media = dispatch_media
        spoken = AsyncMock(return_value="ok.")
        self._speech_patches = [
            patch.object(dispatch, "spoken_after_execute", new=spoken),
            patch.object(media, "spoken_after_execute", new=spoken),
            patch.object(media, "try_engine_speech", new=spoken),
        ]
        for item in self._speech_patches:
            item.start()

    def tearDown(self) -> None:
        for item in reversed(self._speech_patches):
            item.stop()

    async def test_media_status_uses_entity_state_directly(self) -> None:
        player = _State(
            "media_player.wohnzimmer",
            media_title="One",
            media_artist="U2",
            friendly_name="Wohnzimmer",
        )
        hass = _hass(player)
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item(
                "HassGetState",
                entity_id=player.entity_id,
                media_status="now_playing",
            ),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertTrue(spoken.ok)
        dispatch.intent.async_handle.assert_not_awaited()

    async def test_area_only_media_status_never_falls_back_to_lights(self) -> None:
        hass = _hass(_State("light.wohnzimmer", "on", friendly_name="Licht"))
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("HassGetState", area="wohnzimmer", media_status="now_playing"),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertFalse(spoken.ok)
        dispatch.intent.async_handle.assert_not_awaited()

    async def test_queue_requires_media_player_entity(self) -> None:
        player = _State("media_player.wohnzimmer", friendly_name="Wohnzimmer")
        hass = _hass(player)
        hass.services.async_call.return_value = {"items": []}
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("MassGetQueue", entity_id=player.entity_id),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertTrue(spoken.ok)
        call = hass.services.async_call.await_args
        self.assertEqual(call.args[:2], ("music_assistant", "get_queue"))
        self.assertEqual(call.kwargs["target"], {"entity_id": player.entity_id})

        no_target = _hass(player)
        spoken = await dispatch.handle_intent(
            no_target,
            _input(),
            _item("MassGetQueue", area="wohnzimmer"),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertFalse(spoken.ok)
        no_target.services.async_call.assert_not_awaited()

    async def test_unexposed_media_action_does_not_fall_back(self) -> None:
        player = _State("media_player.wohnzimmer", friendly_name="Wohnzimmer")
        hass = _hass(player)
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("HassMediaPause", entity_id=player.entity_id),
            "de",
            None,
            lambda _entity_id: False,
        )
        self.assertFalse(spoken.ok)
        hass.services.async_call.assert_not_awaited()
        dispatch.intent.async_handle.assert_not_awaited()

    async def test_unexposed_media_queries_are_blocked(self) -> None:
        player = _State("media_player.wohnzimmer", friendly_name="Wohnzimmer")
        for item in (
            _item(
                "HassGetState",
                entity_id=player.entity_id,
                media_status="volume",
            ),
            _item("MassGetQueue", entity_id=player.entity_id),
        ):
            with self.subTest(name=item["name"]):
                hass = _hass(player)
                spoken = await dispatch.handle_intent(
                    hass,
                    _input(),
                    item,
                    "de",
                    None,
                    lambda _entity_id: False,
                )
                self.assertFalse(spoken.ok)
                hass.services.async_call.assert_not_awaited()

    async def test_unavailable_media_player_is_not_controlled(self) -> None:
        player = _State(
            "media_player.wohnzimmer",
            "unavailable",
            friendly_name="Wohnzimmer",
        )
        hass = _hass(player)
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("HassMediaNext", entity_id=player.entity_id),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertFalse(spoken.ok)
        hass.services.async_call.assert_not_awaited()

    async def test_search_and_play_on_mass_player_uses_music_assistant(self) -> None:
        player = _State(
            "media_player.wohnzimmer",
            mass_player_type="player",
            friendly_name="Wohnzimmer",
        )
        hass = _hass(player)
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item(
                "HassMediaSearchAndPlay",
                entity_id=player.entity_id,
                search_query="linkin park",
            ),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertTrue(spoken.ok)
        call = hass.services.async_call.await_args
        self.assertEqual(call.args[:2], ("music_assistant", "play_media"))
        self.assertEqual(call.args[2]["media_id"], "linkin park")
        dispatch.intent.async_handle.assert_not_awaited()

    async def test_unpause_uses_media_play_service(self) -> None:
        player = _State("media_player.wohnzimmer", "paused", friendly_name="Wohnzimmer")
        hass = _hass(player)
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("HassMediaUnpause", entity_id=player.entity_id),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertTrue(spoken.ok)
        call = hass.services.async_call.await_args
        self.assertEqual(call.args[:2], ("media_player", "media_play"))
        dispatch.intent.async_handle.assert_not_awaited()

    async def test_volume_uses_native_intent(self) -> None:
        player = _State("media_player.wohnzimmer", friendly_name="Wohnzimmer")
        hass = _hass(player)
        dispatch.intent.async_handle.return_value = object()
        with patch.object(dispatch, "spoken_after_execute", new=AsyncMock(return_value="Wohnzimmer auf 35 Prozent.")):
            spoken = await dispatch.handle_intent(
                hass,
                _input(),
                _item(
                    "HassSetVolume",
                    entity_id=player.entity_id,
                    volume_level=35,
                ),
                "de",
                None,
                lambda _entity_id: True,
            )
        dispatch.intent.async_handle.assert_awaited()
        self.assertEqual(dispatch.intent.async_handle.await_args.args[2], "HassSetVolume")
        hass.services.async_call.assert_not_awaited()
        self.assertTrue(spoken.ok)
        self.assertIn("35 Prozent", spoken.speech or "")

    async def test_transfer_rejects_missing_or_identical_source(self) -> None:
        target = _State(
            "media_player.wohnzimmer",
            friendly_name="Wohnzimmer",
        )
        for source in (None, "media_player.missing", target.entity_id):
            with self.subTest(source=source):
                hass = _hass(target)
                spoken = await dispatch.handle_intent(
                    hass,
                    _input(),
                    _item(
                        "MassTransferQueue",
                        entity_id=target.entity_id,
                        source_player=source,
                    ),
                    "de",
                    None,
                    lambda _entity_id: True,
                )
                self.assertFalse(spoken.ok)
                hass.services.async_call.assert_not_awaited()

    async def test_climate_get_strips_entity_id_for_ha(self) -> None:
        climate = _State(
            "climate.better_thermostat_wohnzimmer",
            "off",
            friendly_name="Heizung Wohnzimmer",
            current_temperature=26.2,
            temperature_unit="°C",
        )
        hass = _hass(climate)
        dispatch.intent.async_handle.return_value = SimpleNamespace(
            matched_states=[climate],
            unmatched_states=[],
            success_results=[],
            response_type="query_answer",
        )
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("HassClimateGetTemperature", entity_id=climate.entity_id, domain="climate"),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertTrue(spoken.ok)
        slots = dispatch.intent.async_handle.await_args.args[3]
        self.assertNotIn("entity_id", slots)
        self.assertNotIn("domain", slots)
        self.assertEqual(slots["name"]["value"], "Heizung Wohnzimmer")

    async def test_climate_get_reads_state_when_ha_intent_fails(self) -> None:
        climate = _State(
            "climate.better_thermostat_wohnzimmer",
            "off",
            friendly_name="Heizung Wohnzimmer",
            current_temperature=26.2,
            temperature_unit="°C",
        )
        hass = _hass(climate)
        dispatch.intent.async_handle.side_effect = Exception("extra keys not allowed")
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("HassClimateGetTemperature", entity_id=climate.entity_id, domain="climate"),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertTrue(spoken.ok)
        dispatch.intent.async_handle.assert_awaited()

    async def test_area_turn_on_uses_ha_area_name(self) -> None:
        hass = _hass()
        dispatch.intent.async_handle.return_value = object()
        with (
            patch.object(dispatch, "area_label", return_value="Wohnzimmer"),
            patch.object(dispatch, "spoken_after_execute", new=AsyncMock(return_value="Licht im Wohnzimmer an.")),
        ):
            spoken = await dispatch.handle_intent(
                hass,
                _input(),
                _item("HassTurnOn", area="wohnzimmer", domain="light"),
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertTrue(spoken.ok)
        slots = dispatch.intent.async_handle.await_args.args[3]
        self.assertEqual(slots["area"]["value"], "Wohnzimmer")
        self.assertEqual(slots["domain"]["value"], "light")

    async def test_generic_named_light_turns_on_by_entity_id(self) -> None:
        light = _State("light.kuche_kuche", "off", friendly_name="Licht")
        hass = _hass(light)
        with patch.object(dispatch, "spoken_after_execute", new=AsyncMock(return_value="Licht ist an.")):
            spoken = await dispatch.handle_intent(
                hass,
                _input(),
                _item("HassTurnOn", entity_id=light.entity_id, area="kuche", domain="light"),
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertTrue(spoken.ok)
        dispatch.intent.async_handle.assert_not_awaited()
        hass.services.async_call.assert_awaited()
        args = hass.services.async_call.await_args
        self.assertEqual(args.args[:2], ("light", "turn_on"))
        self.assertEqual(args.args[2]["entity_id"], "light.kuche_kuche")

    async def test_climate_set_calls_set_temperature(self) -> None:
        climate = _State(
            "climate.better_thermostat_wohnzimmer",
            "off",
            friendly_name="Heizung Wohnzimmer",
            temperature=6.0,
        )
        hass = _hass(climate)
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("HassClimateSetTemperature", entity_id=climate.entity_id, temperature=21),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertTrue(spoken.ok)
        dispatch.intent.async_handle.assert_not_awaited()
        hass.services.async_call.assert_awaited()
        args = hass.services.async_call.await_args
        self.assertEqual(args.args[:2], ("climate", "set_temperature"))
        self.assertEqual(args.args[2]["entity_id"], climate.entity_id)
        self.assertEqual(args.args[2]["temperature"], 21.0)
        self.assertEqual(args.args[2]["hvac_mode"], "heat")
        self.assertNotIn("nicht geklappt", (spoken.speech or "").lower())

    async def test_relative_dim_sends_brightness_step(self) -> None:
        light = _State("light.wohnzimmer", "on", friendly_name="Wohnzimmer Licht", brightness=80)
        hass = _hass(light)
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("HassLightSet", entity_id=light.entity_id, brightness_step="down"),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertTrue(spoken.ok)
        args = hass.services.async_call.await_args
        self.assertEqual(args.args[:2], ("light", "turn_on"))
        self.assertEqual(args.args[2]["brightness_step_pct"], -15)
        self.assertNotIn("brightness_pct", args.args[2])

    async def test_warm_white_uses_kelvin_not_color_name(self) -> None:
        light = _State("light.wohnzimmer", "on", friendly_name="Wohnzimmer Licht")
        hass = _hass(light)
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("HassLightSet", entity_id=light.entity_id, color="warmwhite"),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertTrue(spoken.ok)
        args = hass.services.async_call.await_args
        self.assertEqual(args.args[2]["color_temp_kelvin"], 2700)
        self.assertNotIn("color_name", args.args[2])
        self.assertNotIn("rgb_color", args.args[2])

    async def test_light_set_without_entity_does_not_claim_success(self) -> None:
        hass = _hass()
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("HassLightSet", brightness=100),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertFalse(spoken.ok)
        self.assertEqual(spoken.error, "missing_entity")
        hass.services.async_call.assert_not_awaited()
        dispatch.intent.async_handle.assert_not_awaited()

    async def test_idle_kitchen_music_starts_mass_playback(self) -> None:
        player = _State(
            "media_player.kuchenbereich",
            "idle",
            mass_player_type="player",
            friendly_name="Küche Player",
            volume_level=0.33,
        )
        hass = _hass(player)
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("HassMediaUnpause", entity_id=player.entity_id),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertTrue(spoken.ok)
        call = hass.services.async_call.await_args
        self.assertEqual(call.args[:2], ("music_assistant", "play_media"))
        self.assertEqual(call.args[2]["media_id"], "Musik")
        self.assertEqual(call.kwargs["target"], {"entity_id": player.entity_id})
        self.assertNotIn("volume_level", call.args[2])

    async def test_lab_turn_on_without_entity_keeps_the_plan(self) -> None:
        hass = _hass()
        dispatch.intent.async_handle.return_value = object()
        with (
            patch.object(dispatch, "area_label", return_value="Wohnzimmer"),
            patch.object(dispatch, "spoken_after_execute", new=AsyncMock(return_value="Fernseher an.")),
        ):
            spoken = await dispatch.handle_intent(
                hass,
                _input("Fernseher im Wohnzimmer"),
                _item("HassTurnOn", domain="media_player", area="wohnzimmer"),
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertTrue(spoken.ok)
        dispatch.intent.async_handle.assert_awaited()
        self.assertEqual(dispatch.intent.async_handle.await_args.args[2], "HassTurnOn")

    async def test_lab_turn_on_runs_the_bound_player(self) -> None:
        player = _State("media_player.lg_dsn9yg_8909", "idle", friendly_name="Wohnzimmer")
        hass = _hass(player)
        with patch.object(dispatch, "spoken_after_execute", new=AsyncMock(return_value="Wohnzimmer an.")):
            spoken = await dispatch.handle_intent(
                hass,
                _input("Fernseher im Wohnzimmer"),
                _item("HassTurnOn", entity_id=player.entity_id),
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertTrue(spoken.ok)
        hass.services.async_call.assert_awaited()
        args = hass.services.async_call.await_args
        self.assertEqual(args.args[:2], ("media_player", "turn_on"))
        self.assertEqual(args.args[2]["entity_id"], player.entity_id)

    async def test_idle_alexa_kitchen_starts_text_command(self) -> None:
        player = _State("media_player.kuchenbereich_2", "idle", friendly_name="Küchenbereich", volume_level=0.33)
        hass = _hass(player)
        entry = SimpleNamespace(platform="alexa_devices", device_id="kitchen-echo")
        with patch.object(dispatch.entity_registry, "async_get", return_value=SimpleNamespace(async_get=lambda _id: entry)):
            spoken = await dispatch.handle_intent(
                hass,
                _input("Musik in der Küche"),
                _item("HassMediaUnpause", entity_id=player.entity_id),
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertTrue(spoken.ok)
        call = hass.services.async_call.await_args
        self.assertEqual(call.args[:2], ("alexa_devices", "send_text_command"))
        self.assertEqual(call.args[2]["device_id"], "kitchen-echo")
        self.assertEqual(call.args[2]["text_command"], "spiel Musik")
        self.assertNotIn("volume_level", call.args[2])

    async def test_unavailable_tv_turn_on_does_not_claim_success(self) -> None:
        tv = _State("media_player.wohnzimmer_tv", "unavailable", friendly_name="Wohnzimmer TV")
        hass = _hass(tv)
        spoken = await dispatch.handle_intent(
            hass,
            _input(),
            _item("HassTurnOn", entity_id=tv.entity_id),
            "de",
            None,
            lambda _entity_id: True,
        )
        self.assertFalse(spoken.ok)
        self.assertEqual(spoken.error, "media_unavailable")
        hass.services.async_call.assert_not_awaited()
        dispatch.intent.async_handle.assert_not_awaited()

    async def test_area_climate_get_reads_room_states_when_ha_fails(self) -> None:
        climate = _State(
            "climate.better_thermostat_schlafzimmer",
            "off",
            friendly_name="Heizung Schlafzimmer",
            current_temperature=21.5,
            temperature_unit="°C",
        )
        hass = _hass(climate)
        dispatch.intent.async_handle.side_effect = Exception("no match")
        with (
            patch.object(dispatch, "area_label", return_value="Schlafzimmer"),
            patch.object(dispatch, "climate_states_in_area", return_value=[climate]),
        ):
            spoken = await dispatch.handle_intent(
                hass,
                _input(),
                _item("HassClimateGetTemperature", area="schlafzimmer", domain="climate"),
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertTrue(spoken.ok)
        dispatch.intent.async_handle.assert_awaited()

    async def test_floor_status_speaks_each_area(self) -> None:
        living = _State("light.wohnzimmer", "on", friendly_name="Wohnzimmer")
        kitchen = _State("light.kuche", "off", friendly_name="Küche")
        hass = _hass(living, kitchen)
        with patch.object(
            dispatch,
            "place_status_rooms",
            return_value=[("Wohnzimmer", [living]), ("Küche", [kitchen])],
        ):
            spoken = await dispatch.handle_intent(
                hass,
                _input(),
                _item("HassGetState", floor="wohnung"),
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertTrue(spoken.ok)
        self.assertEqual(spoken.speech, "ok.")
        dispatch.intent.async_handle.assert_not_awaited()

    async def test_area_status_uses_place_speech(self) -> None:
        living = _State("light.wohnzimmer", "on", friendly_name="Wohnzimmer")
        hass = _hass(living)
        with patch.object(
            dispatch,
            "place_status_rooms",
            return_value=[("Wohnzimmer", [living])],
        ):
            spoken = await dispatch.handle_intent(
                hass,
                _input(),
                _item("HassGetState", area="wohnzimmer"),
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertTrue(spoken.ok)
        self.assertEqual(spoken.speech, "ok.")
        dispatch.intent.async_handle.assert_not_awaited()

    async def test_empty_floor_does_not_call_ha(self) -> None:
        hass = _hass()
        with patch.object(dispatch, "place_status_rooms", return_value=[]):
            spoken = await dispatch.handle_intent(
                hass,
                _input("Wie ist der Status der Wohnung"),
                _item("HassGetState", floor="wohnung"),
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertTrue(spoken.ok)
        self.assertEqual(spoken.speech, "ok.")
        dispatch.intent.async_handle.assert_not_awaited()

    def test_non_weather_intent_does_not_forward_utterance(self) -> None:
        user = SimpleNamespace(text="What's on my calendar tomorrow?")
        self.assertEqual(dispatch.intent_query_text(user, "HassSetVolume", {}), "HassSetVolume")
        self.assertEqual(
            dispatch.intent_query_text(user, "HassGetState", {"domain": {"value": "calendar"}}),
            "HassGetState",
        )
        self.assertEqual(
            dispatch.intent_query_text(user, "HassGetState", {"entity_id": {"value": "weather.home"}}),
            user.text,
        )

    async def test_floor_status_speaks_each_area(self) -> None:
        living = _State("light.wohnzimmer", "on", friendly_name="Wohnzimmer")
        kitchen = _State("light.kuche", "off", friendly_name="Küche")
        hass = _hass(living, kitchen)
        with patch.object(
            dispatch,
            "place_get_state",
            return_value="Wohnzimmer. Licht an. Küche. Licht aus.",
        ):
            spoken = await dispatch.handle_intent(
                hass,
                _input(),
                _item("HassGetState", floor="wohnung"),
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertTrue(spoken.ok)
        self.assertIn("Wohnzimmer", spoken.speech or "")
        self.assertIn("Küche", spoken.speech or "")
        dispatch.intent.async_handle.assert_not_awaited()

    async def test_area_status_uses_place_speech(self) -> None:
        living = _State("light.wohnzimmer", "on", friendly_name="Wohnzimmer")
        hass = _hass(living)
        with patch.object(
            dispatch,
            "place_get_state",
            return_value="Wohnzimmer. Licht an.",
        ):
            spoken = await dispatch.handle_intent(
                hass,
                _input(),
                _item("HassGetState", area="wohnzimmer"),
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertTrue(spoken.ok)
        self.assertEqual(spoken.speech, "Wohnzimmer. Licht an.")
        dispatch.intent.async_handle.assert_not_awaited()

    async def test_empty_floor_does_not_call_ha(self) -> None:
        hass = _hass()
        with patch.object(dispatch, "place_get_state", return_value="Keine Geräte."):
            spoken = await dispatch.handle_intent(
                hass,
                _input("Wie ist der Status der Wohnung"),
                _item("HassGetState", floor="wohnung"),
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertTrue(spoken.ok)
        self.assertEqual(spoken.speech, "Keine Geräte.")
        dispatch.intent.async_handle.assert_not_awaited()

    def test_non_weather_intent_does_not_forward_utterance(self) -> None:
        user = SimpleNamespace(text="What's on my calendar tomorrow?")
        self.assertEqual(dispatch.intent_query_text(user, "HassSetVolume", {}), "HassSetVolume")
        self.assertEqual(
            dispatch.intent_query_text(user, "HassGetState", {"domain": {"value": "calendar"}}),
            "HassGetState",
        )
        self.assertEqual(
            dispatch.intent_query_text(user, "HassGetState", {"entity_id": {"value": "weather.home"}}),
            user.text,
        )


if __name__ == "__main__":
    unittest.main()
