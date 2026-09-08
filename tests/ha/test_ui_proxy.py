#!/usr/bin/env python3
"""Klar operator UI proxy: HA auth, session cookie, configured engine only."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = "klar_ui_proxy_test"


def _module(name: str) -> types.ModuleType:
    module = types.ModuleType(name)
    module.__path__ = []
    return module


class _HomeAssistantView:
    requires_auth = True
    url = None
    extra_urls: list[str] = []
    name = None


def _load_ui_proxy() -> types.ModuleType:
    package = _module(PACKAGE)
    homeassistant = _module("homeassistant")
    components = _module("homeassistant.components")
    http = types.ModuleType("homeassistant.components.http")
    http.HomeAssistantView = _HomeAssistantView
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
    components.http = http
    helpers.aiohttp_client = aiohttp_client
    with patch.dict(
        sys.modules,
        {
            PACKAGE: package,
            "homeassistant": homeassistant,
            "homeassistant.components": components,
            "homeassistant.components.http": http,
            "homeassistant.core": core,
            "homeassistant.helpers": helpers,
            "homeassistant.helpers.aiohttp_client": aiohttp_client,
            "aiohttp": aiohttp,
            "aiohttp.web": aiohttp.web,
        },
    ):
        lang_path = ROOT / "custom_components" / "klar_nlu" / "languages.py"
        spec = importlib.util.spec_from_file_location(f"{PACKAGE}.languages", lang_path)
        if spec is None or spec.loader is None:
            raise RuntimeError("cannot load languages")
        languages = importlib.util.module_from_spec(spec)
        sys.modules[f"{PACKAGE}.languages"] = languages
        spec.loader.exec_module(languages)
        const_path = ROOT / "custom_components" / "klar_nlu" / "const.py"
        spec = importlib.util.spec_from_file_location(f"{PACKAGE}.const", const_path)
        if spec is None or spec.loader is None:
            raise RuntimeError("cannot load const")
        const = importlib.util.module_from_spec(spec)
        sys.modules[f"{PACKAGE}.const"] = const
        spec.loader.exec_module(const)
        path = ROOT / "custom_components" / "klar_nlu" / "ui_proxy.py"
        spec = importlib.util.spec_from_file_location(f"{PACKAGE}.ui_proxy", path)
        if spec is None or spec.loader is None:
            raise RuntimeError(f"cannot load {path}")
        module = importlib.util.module_from_spec(spec)
        sys.modules[f"{PACKAGE}.ui_proxy"] = module
        spec.loader.exec_module(module)
        return module


proxy = _load_ui_proxy()


class _FakeRequest:
    def __init__(
        self,
        *,
        authenticated: bool = False,
        user: object | None = None,
        cookie: str | None = None,
    ) -> None:
        self._data: dict[str, object] = {}
        if authenticated:
            self._data["ha_authenticated"] = True
        if user is not None:
            self._data["hass_user"] = user
        self.cookies = {proxy.COOKIE_NAME: cookie} if cookie else {}

    def get(self, key: str, default: object = None) -> object:
        return self._data.get(key, default)


class UiProxyTests(unittest.TestCase):
    def test_session_view_requires_auth(self) -> None:
        self.assertTrue(proxy.KlarUiSessionView.requires_auth)
        self.assertEqual(proxy.KlarUiSessionView.url, "/api/klar_nlu/session")

    def test_proxy_view_requires_ha_auth_check(self) -> None:
        self.assertTrue(proxy.KlarUiProxyView.requires_ha_auth)
        self.assertEqual(proxy.KlarUiProxyView.url, "/api/klar_nlu/ui")

    def test_request_allowed_rejects_anonymous(self) -> None:
        secret = b"s" * 32
        self.assertFalse(proxy.request_allowed(_FakeRequest(), secret, now=1000))

    def test_request_allowed_accepts_ha_user(self) -> None:
        secret = b"s" * 32
        req = _FakeRequest(authenticated=True, user=SimpleNamespace(id="u1"))
        self.assertTrue(proxy.request_allowed(req, secret, now=1000))

    def test_request_allowed_accepts_valid_cookie(self) -> None:
        secret = b"s" * 32
        cookie = proxy.mint_ui_cookie(secret, "u1", now=1000, ttl=60)
        req = _FakeRequest(cookie=cookie)
        self.assertTrue(proxy.request_allowed(req, secret, now=1010))
        self.assertFalse(proxy.request_allowed(req, secret, now=2000))
        self.assertFalse(proxy.request_allowed(_FakeRequest(cookie="nope"), secret, now=1010))

    def test_join_stays_on_configured_engine(self) -> None:
        base = "http://127.0.0.1:10520"
        self.assertEqual(
            proxy.join_engine_url(base, "api/v2/parse"),
            "http://127.0.0.1:10520/api/v2/parse",
        )
        self.assertEqual(
            proxy.join_engine_url(base, "api/v2/parse", "url=http://evil.test"),
            "http://127.0.0.1:10520/api/v2/parse?url=http://evil.test",
        )
        parsed = urlparse(
            proxy.join_engine_url(base, "", "host=evil.test&url=http://evil.test")
        )
        self.assertEqual(parsed.hostname, "127.0.0.1")
        self.assertEqual(parsed.port, 10520)

    def test_join_rejects_absolute_and_protocol_relative(self) -> None:
        base = "http://127.0.0.1:10520"
        with self.assertRaises(ValueError):
            proxy.join_engine_url(base, "http://evil.test/x")
        with self.assertRaises(ValueError):
            proxy.join_engine_url(base, "//evil.test/x")
        with self.assertRaises(ValueError):
            proxy.join_engine_url(base, "https://evil.test")

    def test_join_rejects_empty_base(self) -> None:
        with self.assertRaises(ValueError):
            proxy.join_engine_url("", "api/v2")

    def test_configured_engine_uses_stored_url_only(self) -> None:
        hass = SimpleNamespace(
            data={
                "klar_nlu": {
                    "panel": True,
                    "ui_proxy": True,
                    "entry": {
                        "url": "http://127.0.0.1:10520",
                        "token": "secret-token",
                    },
                }
            }
        )
        url, token = proxy.configured_engine(hass)
        self.assertEqual(url, "http://127.0.0.1:10520")
        self.assertEqual(token, "secret-token")
        hass.data["klar_nlu"]["entry"]["url"] = "http://192.168.1.40:10520"
        url, token = proxy.configured_engine(hass)
        self.assertEqual(url, "http://192.168.1.40:10520")
        self.assertNotIn("evil", url)

    def test_addon_ingress_detects_hassio_panel(self) -> None:
        hass = SimpleNamespace(
            data={"frontend_panels": {"klar_nlu": SimpleNamespace(component_name="hassio")}},
            config=SimpleNamespace(components=set()),
        )
        self.assertTrue(proxy.addon_ingress_active(hass))
        hass.data["frontend_panels"]["klar_nlu"] = SimpleNamespace(component_name="custom")
        self.assertFalse(proxy.addon_ingress_active(hass))

    def test_addon_ingress_uses_addon_url_only_on_supervisor(self) -> None:
        hass = SimpleNamespace(
            data={"klar_nlu": {"entry": {"url": "http://klar-nlu:10520"}}},
            config=SimpleNamespace(components={"hassio"}),
        )
        self.assertTrue(proxy.addon_ingress_active(hass))
        hass.config.components = set()
        self.assertFalse(proxy.addon_ingress_active(hass))
        hass.config.components = {"hassio"}
        hass.data["klar_nlu"]["entry"]["url"] = "http://127.0.0.1:10520"
        self.assertFalse(proxy.addon_ingress_active(hass))


if __name__ == "__main__":
    unittest.main()
