#!/usr/bin/env python3
"""Klar sidebar dashboard config and operator panel registration."""

from __future__ import annotations

import asyncio
import importlib.util
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = "klar_panel_test"


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


class _HomeAssistantView:
    requires_auth = True
    url = None
    extra_urls: list[str] = []
    name = None


def _load_panel() -> types.ModuleType:
    package = _module(PACKAGE)
    homeassistant = _module("homeassistant")
    components = _module("homeassistant.components")
    frontend = types.ModuleType("homeassistant.components.frontend")
    frontend.add_extra_js_url = lambda *_args, **_kwargs: None
    frontend.async_register_built_in_panel = lambda *_args, **_kwargs: None
    frontend.async_remove_panel = lambda *_args, **_kwargs: None
    http = types.ModuleType("homeassistant.components.http")
    http.HomeAssistantView = _HomeAssistantView
    http.StaticPathConfig = None
    core = types.ModuleType("homeassistant.core")
    core.HomeAssistant = object
    helpers = _module("homeassistant.helpers")
    aiohttp_client = types.ModuleType("homeassistant.helpers.aiohttp_client")
    aiohttp_client.async_get_clientsession = lambda _hass: None
    aiohttp = types.ModuleType("aiohttp")
    aiohttp.ClientError = Exception
    aiohttp.ClientTimeout = lambda **_kwargs: None
    aiohttp.web = types.ModuleType("aiohttp.web")
    aiohttp.web.Request = object
    aiohttp.web.Response = object
    aiohttp.web.StreamResponse = object
    homeassistant.components = components
    homeassistant.core = core
    homeassistant.helpers = helpers
    components.frontend = frontend
    components.http = http
    helpers.aiohttp_client = aiohttp_client
    with patch.dict(
        sys.modules,
        {
            PACKAGE: package,
            "homeassistant": homeassistant,
            "homeassistant.components": components,
            "homeassistant.components.frontend": frontend,
            "homeassistant.components.http": http,
            "homeassistant.core": core,
            "homeassistant.helpers": helpers,
            "homeassistant.helpers.aiohttp_client": aiohttp_client,
            "aiohttp": aiohttp,
            "aiohttp.web": aiohttp.web,
        },
    ):
        _load(f"{PACKAGE}.languages", "languages.py")
        _load(f"{PACKAGE}.const", "const.py")
        _load(f"{PACKAGE}.ui_proxy", "ui_proxy.py")
        return _load(f"{PACKAGE}.panel", "panel.py")


panel = _load_panel()


class _FakeHttp:
    def register_static_path(self, *_args, **_kwargs) -> None:
        return None

    async def async_register_static_paths(self, _configs) -> None:
        return None


def _hass(*, components=(), panels=None, domain=None):
    data = {}
    if domain is not None:
        data[panel.DOMAIN] = domain
    if panels is not None:
        data["frontend_panels"] = panels
    return SimpleNamespace(
        data=data,
        config=SimpleNamespace(components=set(components)),
        http=_FakeHttp(),
    )


class PanelDashboardTests(unittest.TestCase):
    def test_dashboard_config_has_storage_id(self) -> None:
        config = panel._dashboard_config("klar-nlu")
        self.assertEqual(config["id"], "klar-nlu")
        self.assertEqual(config["url_path"], "klar-nlu")
        self.assertEqual(config["mode"], "storage")

    def test_operator_panel_uses_custom_element(self) -> None:
        config = panel.operator_panel_config()
        custom = config["_panel_custom"]
        self.assertEqual(custom["name"], "klar-nlu-panel")
        self.assertEqual(custom["js_url"], "/klar_nlu/panel.js")
        self.assertFalse(custom["embed_iframe"])

    def test_registers_sidebar_panel(self) -> None:
        hass = _hass(domain={})
        captured: list[dict] = []

        def capture(_hass, **kwargs) -> None:
            captured.append(kwargs)

        with patch.object(panel, "async_register_built_in_panel", capture):
            asyncio.run(panel.async_setup_panel(hass))
        self.assertEqual(len(captured), 1)
        self.assertEqual(captured[0]["component_name"], "custom")
        self.assertEqual(captured[0]["sidebar_title"], "Klar NLU")
        self.assertEqual(captured[0]["sidebar_icon"], "mdi:brain")
        self.assertEqual(captured[0]["frontend_url_path"], "klar-nlu")
        self.assertTrue(captured[0]["require_admin"])
        self.assertEqual(
            captured[0]["config"]["_panel_custom"]["name"], "klar-nlu-panel"
        )

    def test_skips_sidebar_when_hassio_addon_panel_exists(self) -> None:
        hass = _hass(
            domain={"entry": {"url": "http://klar-nlu:10520", "token": None}},
            panels={"klar_nlu": SimpleNamespace(component_name="hassio")},
        )
        captured: list[dict] = []
        with patch.object(
            panel, "async_register_built_in_panel", lambda *_a, **_k: captured.append({})
        ):
            asyncio.run(panel.async_setup_panel(hass))
        self.assertEqual(captured, [])

    def test_registers_hacs_when_addon_panel_missing(self) -> None:
        hass = _hass(
            components=("hassio",),
            domain={"entry": {"url": "http://klar-nlu-staging:10520", "token": None}},
            panels={"klar_nlu": SimpleNamespace(component_name="hassio")},
        )
        captured: list[dict] = []
        with patch.object(
            panel, "async_register_built_in_panel", lambda *_a, **kwargs: captured.append(kwargs)
        ):
            asyncio.run(panel.async_setup_panel(hass))
        self.assertEqual(len(captured), 1)
        self.assertEqual(captured[0]["frontend_url_path"], "klar-nlu")

    def test_registers_when_hassio_uses_bundled_loopback(self) -> None:
        hass = _hass(
            components=("hassio",),
            domain={"entry": {"url": "http://127.0.0.1:10520", "token": None}},
        )
        captured: list[dict] = []
        with patch.object(
            panel, "async_register_built_in_panel", lambda *_a, **kwargs: captured.append(kwargs)
        ):
            asyncio.run(panel.async_setup_panel(hass))
        self.assertEqual(len(captured), 1)
        self.assertEqual(captured[0]["frontend_url_path"], "klar-nlu")


if __name__ == "__main__":
    unittest.main()
