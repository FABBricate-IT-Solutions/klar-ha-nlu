#!/usr/bin/env python3
"""Unit tests for capturing and posting undo priors (package 6.5)."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock, patch

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = "klar_undo_priors_test"


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


def _load_undo() -> types.ModuleType:
    homeassistant = _module("homeassistant")
    core = types.ModuleType("homeassistant.core")
    core.HomeAssistant = object
    helpers = _module("homeassistant.helpers")
    aiohttp_client = types.ModuleType("homeassistant.helpers.aiohttp_client")
    aiohttp_client.async_get_clientsession = MagicMock()
    helpers.aiohttp_client = aiohttp_client
    package = _module(PACKAGE)
    aiohttp = types.ModuleType("aiohttp")
    aiohttp.ClientTimeout = lambda **_kwargs: object()
    aiohttp.ClientError = type("ClientError", (Exception,), {})
    modules = {
        "homeassistant": homeassistant,
        "homeassistant.core": core,
        "homeassistant.helpers": helpers,
        "homeassistant.helpers.aiohttp_client": aiohttp_client,
        "aiohttp": aiohttp,
        PACKAGE: package,
    }
    with patch.dict(sys.modules, modules):
        languages = _load(f"{PACKAGE}.languages", "languages.py")
        sys.modules[f"{PACKAGE}.languages"] = languages
        const = _load(f"{PACKAGE}.const", "const.py")
        sys.modules[f"{PACKAGE}.const"] = const
        package.const = const
        undo = _load(f"{PACKAGE}.undo_priors", "undo_priors.py")
    return undo


undo_priors = _load_undo()


def _state(entity_id: str, **attrs):
    return SimpleNamespace(entity_id=entity_id, state="on", attributes=attrs)


class CapturePriorsTests(unittest.TestCase):
    def test_captures_brightness_position_temperature_volume_fan(self):
        hass = MagicMock()
        hass.states.get.side_effect = lambda eid: {
            "light.desk": _state("light.desk", brightness=204),  # ~80%
            "cover.blind": _state("cover.blind", current_position=70),
            "climate.heat": _state("climate.heat", temperature=19.0),
            "media_player.box": _state("media_player.box", volume_level=0.55),
            "fan.vent": _state("fan.vent", percentage=40),
        }.get(eid)

        intents = [
            {"name": "HassLightSet", "slots": [{"name": "entity_id", "value": "light.desk"}]},
            {"name": "HassSetPosition", "slots": [{"name": "entity_id", "value": "cover.blind"}]},
            {
                "name": "HassClimateSetTemperature",
                "slots": [{"name": "entity_id", "value": "climate.heat"}],
            },
            {"name": "HassSetVolume", "slots": [{"name": "entity_id", "value": "media_player.box"}]},
            {"name": "HassFanSetSpeed", "slots": [{"name": "entity_id", "value": "fan.vent"}]},
            {"name": "HassTurnOn", "slots": [{"name": "entity_id", "value": "light.desk"}]},
        ]
        priors = undo_priors.capture_priors(hass, intents)
        by_id = {row["entity_id"]: row for row in priors}
        self.assertEqual(by_id["light.desk"]["brightness"], 80)
        self.assertEqual(by_id["cover.blind"]["position"], 70)
        self.assertEqual(by_id["climate.heat"]["temperature"], "19.0")
        self.assertEqual(by_id["media_player.box"]["volume_level"], 55)
        self.assertEqual(by_id["fan.vent"]["percentage"], 40)
        self.assertEqual(len(priors), 5)

    def test_skips_unknown_entities(self):
        hass = MagicMock()
        hass.states.get.return_value = None
        priors = undo_priors.capture_priors(
            hass,
            [{"name": "HassLightSet", "slots": [{"name": "entity_id", "value": "light.missing"}]}],
        )
        self.assertEqual(priors, [])


class StampThenExecuteTests(unittest.IsolatedAsyncioTestCase):
    async def test_posts_priors_before_execute(self):
        hass = MagicMock()
        hass.states.get.return_value = _state("light.desk", brightness=128)
        execute = AsyncMock(return_value={"outcome": "success", "speech": "ok", "steps": []})
        with patch.object(undo_priors, "post_priors", new=AsyncMock()) as post:
            result = await undo_priors.stamp_then_execute(
                hass,
                MagicMock(),
                [{"name": "HassLightSet", "slots": [{"name": "entity_id", "value": "light.desk"}]}],
                "en",
                None,
                lambda _eid: True,
                url="http://127.0.0.1:10520",
                token=None,
                conversation_id="c1",
                execute_plan=execute,
            )
        self.assertEqual(result["outcome"], "success")
        post.assert_awaited_once()
        kwargs = post.await_args.kwargs
        self.assertEqual(kwargs["conversation_id"], "c1")
        self.assertEqual(kwargs["priors"][0]["brightness"], 50)
        execute.assert_awaited_once()


if __name__ == "__main__":
    unittest.main()
