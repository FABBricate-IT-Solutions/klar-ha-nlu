#!/usr/bin/env python3
"""D-08 / H-15: settings PATCH and short-lived cache."""

from __future__ import annotations

import asyncio
import importlib.util
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, MagicMock, patch

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = "klar_settings_cache_test"


def _install_ha_stubs() -> None:
    if "homeassistant" in sys.modules:
        return
    ha = types.ModuleType("homeassistant")
    ha.config_entries = types.ModuleType("homeassistant.config_entries")
    ha.config_entries.ConfigEntry = object
    ha.core = types.ModuleType("homeassistant.core")
    ha.core.HomeAssistant = object
    ha.helpers = types.ModuleType("homeassistant.helpers")
    ha.helpers.issue_registry = types.ModuleType("homeassistant.helpers.issue_registry")
    ha.helpers.aiohttp_client = types.ModuleType("homeassistant.helpers.aiohttp_client")
    ha.helpers.aiohttp_client.async_get_clientsession = MagicMock()
    aiohttp = types.ModuleType("aiohttp")
    aiohttp.ClientError = Exception
    aiohttp.ClientSession = object

    class ClientTimeout:
        def __init__(self, total=None):
            self.total = total

    aiohttp.ClientTimeout = ClientTimeout
    sys.modules.update(
        {
            "homeassistant": ha,
            "homeassistant.config_entries": ha.config_entries,
            "homeassistant.core": ha.core,
            "homeassistant.helpers": ha.helpers,
            "homeassistant.helpers.issue_registry": ha.helpers.issue_registry,
            "homeassistant.helpers.aiohttp_client": ha.helpers.aiohttp_client,
            "aiohttp": aiohttp,
        }
    )


def _load_engine():
    _install_ha_stubs()
    package = types.ModuleType(PACKAGE)
    package.__path__ = [str(ROOT / "custom_components" / "klar_nlu")]
    sys.modules[PACKAGE] = package

    def load(name: str, file_name: str):
        path = ROOT / "custom_components" / "klar_nlu" / file_name
        spec = importlib.util.spec_from_file_location(f"{PACKAGE}.{name}", path)
        assert spec and spec.loader
        module = importlib.util.module_from_spec(spec)
        sys.modules[f"{PACKAGE}.{name}"] = module
        spec.loader.exec_module(module)
        setattr(package, name, module)
        return module

    load("languages", "languages.py")
    load("const", "const.py")
    load("lang_select", "lang_select.py")
    load("time_zone", "time_zone.py")
    load("archive", "archive.py")
    if (ROOT / "custom_components" / "klar_nlu" / "units.py").is_file():
        load("units", "units.py")
    return load("engine", "engine.py")


engine = _load_engine()


class SettingsCacheTests(unittest.TestCase):
    def test_refresh_respects_ttl(self) -> None:
        entry = SimpleNamespace(entry_id="e1", data={}, options={})
        hass = SimpleNamespace(data={engine.DOMAIN: {"e1": {"url": "http://engine:10520"}}})

        fetches: list[str] = []

        async def fake_fetch(hass_, url, token=None):
            fetches.append(url)
            return {"quiet_ack": False, "personality": "butler"}

        with patch.object(engine, "async_fetch_settings", side_effect=fake_fetch):
            first = asyncio.run(engine.async_refresh_engine_settings(hass, entry))
            second = asyncio.run(engine.async_refresh_engine_settings(hass, entry))
        self.assertEqual(len(fetches), 1)
        self.assertEqual(first.get("personality"), "butler")
        self.assertEqual(second.get("personality"), "butler")

    def test_patch_prefers_http_patch(self) -> None:
        entry = SimpleNamespace(entry_id="e1", data={}, options={})
        hass = SimpleNamespace(
            data={
                engine.DOMAIN: {
                    "e1": {
                        "url": "http://engine:10520",
                        "token": None,
                        "engine_settings": {"quiet_ack": False, "extra_prompt": "keep"},
                    }
                }
            }
        )

        async def fake_patch(hass_, url, patch, token=None):
            self.assertEqual(patch, {"quiet_ack": True})
            return {"quiet_ack": True, "extra_prompt": "keep", "personality": "butler"}

        with (
            patch.object(engine, "async_patch_settings", side_effect=fake_patch) as patched,
            patch.object(engine, "async_put_settings", new_callable=AsyncMock) as put,
        ):
            result = asyncio.run(engine.async_patch_engine_settings(hass, entry, {"quiet_ack": True}))
        patched.assert_awaited()
        put.assert_not_awaited()
        self.assertTrue(result["quiet_ack"])
        self.assertEqual(result["extra_prompt"], "keep")
        self.assertEqual(result["personality"], "butler")


if __name__ == "__main__":
    unittest.main()
