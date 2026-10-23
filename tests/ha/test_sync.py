#!/usr/bin/env python3
"""Home Assistant registry snapshot push."""

from __future__ import annotations

import importlib.util
import json
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = "klar_sync_test"


def _module(name: str) -> types.ModuleType:
    module = types.ModuleType(name)
    module.__path__ = []
    return module


def _load_sync() -> types.ModuleType:
    homeassistant = _module("homeassistant")
    components = _module("homeassistant.components")
    exposed = types.ModuleType("homeassistant.components.homeassistant.exposed_entities")
    exposed.async_should_expose = lambda _hass, _assistant, entity_id: entity_id == "light.living"
    ha_home = _module("homeassistant.components.homeassistant")
    config_entries = types.ModuleType("homeassistant.config_entries")
    config_entries.ConfigEntry = object
    core = types.ModuleType("homeassistant.core")
    core.HomeAssistant = object
    core.Event = object
    core.callback = lambda fn: fn
    helpers = _module("homeassistant.helpers")
    aiohttp_client = types.ModuleType("homeassistant.helpers.aiohttp_client")
    aiohttp_client.async_get_clientsession = lambda _hass: None
    area_registry = types.ModuleType("homeassistant.helpers.area_registry")
    device_registry = types.ModuleType("homeassistant.helpers.device_registry")
    entity_registry = types.ModuleType("homeassistant.helpers.entity_registry")
    floor_registry = types.ModuleType("homeassistant.helpers.floor_registry")
    label_registry = types.ModuleType("homeassistant.helpers.label_registry")
    aiohttp = types.ModuleType("aiohttp")
    aiohttp.ClientError = Exception
    aiohttp.ClientTimeout = lambda **_kwargs: None
    modules = {
        "aiohttp": aiohttp,
        "homeassistant": homeassistant,
        "homeassistant.components": components,
        "homeassistant.components.homeassistant": ha_home,
        "homeassistant.components.homeassistant.exposed_entities": exposed,
        "homeassistant.config_entries": config_entries,
        "homeassistant.core": core,
        "homeassistant.helpers": helpers,
        "homeassistant.helpers.aiohttp_client": aiohttp_client,
        "homeassistant.helpers.area_registry": area_registry,
        "homeassistant.helpers.device_registry": device_registry,
        "homeassistant.helpers.entity_registry": entity_registry,
        "homeassistant.helpers.floor_registry": floor_registry,
        "homeassistant.helpers.label_registry": label_registry,
        PACKAGE: _module(PACKAGE),
        f"{PACKAGE}.languages": _load_languages(),
        f"{PACKAGE}.const": _load_const(),
    }
    asr_path = ROOT / "custom_components" / "klar_nlu" / "asr_boost.py"
    asr_spec = importlib.util.spec_from_file_location(f"{PACKAGE}.asr_boost", asr_path)
    if asr_spec is None or asr_spec.loader is None:
        raise RuntimeError(f"cannot load {asr_path}")
    asr_module = importlib.util.module_from_spec(asr_spec)
    path = ROOT / "custom_components" / "klar_nlu" / "sync.py"
    spec = importlib.util.spec_from_file_location(f"{PACKAGE}.sync", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    with patch.dict(sys.modules, modules):
        sys.modules[f"{PACKAGE}.asr_boost"] = asr_module
        asr_spec.loader.exec_module(asr_module)
        spec.loader.exec_module(module)
    return module


def _load_languages() -> types.ModuleType:
    path = ROOT / "custom_components" / "klar_nlu" / "languages.py"
    spec = importlib.util.spec_from_file_location(f"{PACKAGE}.languages", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[f"{PACKAGE}.languages"] = module
    spec.loader.exec_module(module)
    return module


def _load_const() -> types.ModuleType:
    if PACKAGE not in sys.modules:
        sys.modules[PACKAGE] = _module(PACKAGE)
    if f"{PACKAGE}.languages" not in sys.modules:
        _load_languages()
    path = ROOT / "custom_components" / "klar_nlu" / "const.py"
    spec = importlib.util.spec_from_file_location(f"{PACKAGE}.const", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


sync = _load_sync()


class _Map:
    def __init__(self, *items: object) -> None:
        self._items = {getattr(item, "id", getattr(item, "entity_id", getattr(item, "floor_id", ""))): item for item in items}

    def values(self) -> list[object]:
        return list(self._items.values())


class SyncTests(unittest.TestCase):
    def test_snapshot_includes_floors_aliases_and_assist(self) -> None:
        hass = SimpleNamespace()
        entry = SimpleNamespace(options={"assist_filter": True}, data={})
        entity = SimpleNamespace(
            entity_id="light.living",
            name="Living",
            original_name="Ceiling",
            has_entity_name=True,
            area_id="living",
            device_id="dev1",
            platform="hue",
            aliases=["decke"],
            labels=["Licht"],
            disabled_by=None,
        )
        device = SimpleNamespace(id="dev1", name="Hue", name_by_user=None, area_id="living")
        area = SimpleNamespace(id="living", name="Wohnzimmer", aliases=["wohnzimmer"], floor_id="upper")
        floor = SimpleNamespace(floor_id="upper", name="Upper Floor", aliases=["upstairs"], level=1)
        label = SimpleNamespace(label_id="lbl_1", name="Licht")
        sync.entity_registry.async_get = lambda _hass: SimpleNamespace(entities=_Map(entity))
        sync.device_registry.async_get = lambda _hass: SimpleNamespace(devices=_Map(device))
        sync.area_registry.async_get = lambda _hass: SimpleNamespace(areas=_Map(area))
        sync.floor_registry.async_get = lambda _hass: SimpleNamespace(floors=_Map(floor))
        sync.label_registry.async_get = lambda _hass: SimpleNamespace(labels=_Map(label))
        pusher = sync.HomeGraphSync(hass, entry, "http://127.0.0.1:10520", "token")
        snapshot = pusher.build_snapshot()
        self.assertEqual(snapshot["schema_version"], "1")
        self.assertEqual(snapshot["floors"][0]["floor_id"], "upper")
        self.assertEqual(snapshot["areas"][0]["floor_id"], "upper")
        self.assertEqual(snapshot["entities"][0]["aliases"], ["decke"])
        self.assertEqual(snapshot["labels"][0]["name"], "Licht")
        self.assertEqual(snapshot["assist"], ["light.living"])
        self.assertIn("registered_intents", snapshot)

    def test_hidden_entities_are_omitted_from_assist(self) -> None:
        hass = SimpleNamespace()
        entry = SimpleNamespace(options={"assist_filter": True}, data={})
        hidden = SimpleNamespace(
            entity_id="light.hidden",
            name="Hidden",
            original_name=None,
            has_entity_name=False,
            area_id=None,
            device_id=None,
            platform=None,
            aliases=[],
            labels=[],
            disabled_by=None,
        )
        sync.entity_registry.async_get = lambda _hass: SimpleNamespace(entities=_Map(hidden))
        sync.device_registry.async_get = lambda _hass: SimpleNamespace(devices=_Map())
        sync.area_registry.async_get = lambda _hass: SimpleNamespace(areas=_Map())
        sync.floor_registry.async_get = lambda _hass: SimpleNamespace(floors=_Map())
        sync.label_registry.async_get = lambda _hass: SimpleNamespace(labels=_Map())
        snapshot = sync.HomeGraphSync(hass, entry, "http://127.0.0.1:10520", None).build_snapshot()
        self.assertEqual(snapshot["assist"], [])

    def test_large_snapshot_keeps_every_enabled_entity_regardless_of_exposure_or_domain(self) -> None:
        count = 2216
        domains = ["light", "sensor", "automation", "button", "binary_sensor", "update", "person"]
        entities = [SimpleNamespace(
            entity_id=f"{domains[index % len(domains)]}.fixture_{index}", name=f"Fixture {index}",
            original_name="Original fixture name " * 4, has_entity_name=True, area_id=None,
            device_id=f"device_{index % 300}", platform="fixture",
            aliases=[f"fixture alias {index}", "alias " * 12], labels=["lighting"], disabled_by=None,
        ) for index in range(count)]
        devices = [SimpleNamespace(id=f"device_{index}", name=f"Device {index}", name_by_user=None,
            area_id=f"area_{index % 24}") for index in range(300)]
        areas = [SimpleNamespace(id=f"area_{index}", name=f"Area {index}", aliases=[],
            floor_id=f"floor_{index % 4}") for index in range(24)]
        floors = [SimpleNamespace(floor_id=f"floor_{index}", name=f"Floor {index}", aliases=[], level=index)
            for index in range(4)]
        label = SimpleNamespace(label_id="lighting", name="Lighting")
        sync.entity_registry.async_get = lambda _: SimpleNamespace(entities=_Map(*entities))
        sync.device_registry.async_get = lambda _: SimpleNamespace(devices=_Map(*devices))
        sync.area_registry.async_get = lambda _: SimpleNamespace(areas=_Map(*areas))
        sync.floor_registry.async_get = lambda _: SimpleNamespace(floors=_Map(*floors))
        sync.label_registry.async_get = lambda _: SimpleNamespace(labels=_Map(label))
        with patch.object(sync, "_exposed", side_effect=lambda _, entity_id: entity_id == entities[0].entity_id):
            snapshot = sync.HomeGraphSync(SimpleNamespace(), SimpleNamespace(options={"assist_filter": True}, data={}),
                "http://127.0.0.1:10520", None).build_snapshot()
        self.assertGreater(len(json.dumps(snapshot["entities"]).encode("utf-8")), 538_682)
        self.assertEqual([row["entity_id"] for row in snapshot["entities"]], [entity.entity_id for entity in entities])
        self.assertEqual(snapshot["assist"], [entities[0].entity_id])
        self.assertEqual(len(snapshot["devices"]), 300)
        self.assertEqual(len(snapshot["areas"]), 24)
        self.assertEqual(len(snapshot["floors"]), 4)
        self.assertEqual(snapshot["labels"], [{"label_id": "lighting", "name": "Lighting"}])

    def test_live_sync_retries_when_engine_is_gone(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "sync.py").read_text(encoding="utf-8")
        self.assertIn("_PUSH_EVERY_S", src)
        self.assertIn("_RETRY_S", src)
        self.assertIn("async def _loop", src)
        self.assertIn("return False", src)
        self.assertIn("async_create_background_task", src)
        self.assertIn("self._loop()", src)


if __name__ == "__main__":
    unittest.main()
