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
    helpers.intent = intent
    helpers.area_registry = area_registry
    helpers.entity_registry = entity_registry
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
        PACKAGE: package,
    }
    with patch.dict(sys.modules, modules):
        _load(f"{PACKAGE}.speech", "speech.py")
        _load(f"{PACKAGE}.intents", "intents.py")
        return _load(f"{PACKAGE}.dispatch", "dispatch.py")


dispatch = _load_dispatch()


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


def _input() -> SimpleNamespace:
    return SimpleNamespace(text="test", context=object(), language="de")


def _item(name: str, **slots: object) -> dict[str, object]:
    return {
        "name": name,
        "slots": [{"name": key, "value": value} for key, value in slots.items()],
    }


class DispatchTests(unittest.IsolatedAsyncioTestCase):
    def setUp(self) -> None:
        dispatch.intent.async_handle = AsyncMock()

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
        self.assertEqual(spoken.speech, "Gerade läuft One by U2.")
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
        self.assertIn("Warteschlange", spoken.speech or "")
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

    async def test_volume_uses_native_intent(self) -> None:
        player = _State("media_player.wohnzimmer", friendly_name="Wohnzimmer")
        hass = _hass(player)
        dispatch.intent.async_handle.return_value = object()
        with patch.object(dispatch, "from_handled", return_value="Wohnzimmer auf 35 Prozent."):
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
        self.assertIn("26", spoken.speech or "")

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
        self.assertIn("26", spoken.speech or "")
        self.assertIn("Wohnzimmer", spoken.speech or "")

    async def test_area_turn_on_uses_ha_area_name(self) -> None:
        hass = _hass()
        dispatch.intent.async_handle.return_value = object()
        with (
            patch.object(dispatch, "area_label", return_value="Wohnzimmer"),
            patch.object(dispatch, "from_handled", return_value="Licht im Wohnzimmer an."),
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
        self.assertIn("21", spoken.speech or "")
        self.assertIn("Schlafzimmer", spoken.speech or "")


if __name__ == "__main__":
    unittest.main()
