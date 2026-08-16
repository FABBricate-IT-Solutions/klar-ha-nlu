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
    helpers.intent = intent
    helpers.area_registry = area_registry
    package = _module(PACKAGE)
    modules = {
        "homeassistant": homeassistant,
        "homeassistant.components": components,
        "homeassistant.components.conversation": conversation,
        "homeassistant.core": core,
        "homeassistant.helpers": helpers,
        "homeassistant.helpers.intent": intent,
        "homeassistant.helpers.area_registry": area_registry,
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
        self.assertEqual(spoken, "Gerade läuft One by U2.")
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
        self.assertIsNone(spoken)
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
        self.assertIn("Warteschlange", spoken or "")
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
        self.assertIsNone(spoken)
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
        self.assertIsNone(spoken)
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
                self.assertIsNone(spoken)
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
        self.assertIsNone(spoken)
        hass.services.async_call.assert_not_awaited()

    async def test_volume_calls_media_player_service_and_speaks_result(self) -> None:
        player = _State("media_player.wohnzimmer", friendly_name="Wohnzimmer")
        hass = _hass(player)
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
        hass.services.async_call.assert_awaited_once_with(
            "media_player",
            "volume_set",
            {"entity_id": player.entity_id, "volume_level": 0.35},
            blocking=True,
        )
        self.assertIn("35 Prozent", spoken or "")
        self.assertNotIn("HassSetVolume", spoken or "")

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
                self.assertIsNone(spoken)
                hass.services.async_call.assert_not_awaited()


if __name__ == "__main__":
    unittest.main()
