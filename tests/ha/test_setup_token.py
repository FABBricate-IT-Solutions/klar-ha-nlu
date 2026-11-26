#!/usr/bin/env python3
"""Package 6.6: Supervisor claim of the auto-minted write token."""

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
PACKAGE = "klar_setup_token_test"


def _ensure_stubs() -> None:
    if "aiohttp" not in sys.modules:
        aiohttp = types.ModuleType("aiohttp")
        aiohttp.ClientError = type("ClientError", (Exception,), {})

        class _ClientTimeout:
            def __init__(self, total=None):
                self.total = total

        aiohttp.ClientTimeout = _ClientTimeout
        sys.modules["aiohttp"] = aiohttp
    if "homeassistant.helpers.aiohttp_client" in sys.modules:
        return
    ha = types.ModuleType("homeassistant")
    ha.core = types.ModuleType("homeassistant.core")
    ha.core.HomeAssistant = object
    ha.helpers = types.ModuleType("homeassistant.helpers")
    ha.helpers.aiohttp_client = types.ModuleType("homeassistant.helpers.aiohttp_client")
    ha.helpers.aiohttp_client.async_get_clientsession = MagicMock()
    for name, mod in (
        ("homeassistant", ha),
        ("homeassistant.core", ha.core),
        ("homeassistant.helpers", ha.helpers),
        ("homeassistant.helpers.aiohttp_client", ha.helpers.aiohttp_client),
    ):
        sys.modules[name] = mod


def _load_setup_token():
    _ensure_stubs()
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
    return load("setup_token", "setup_token.py")


class _RespCtx:
    def __init__(self, resp: object) -> None:
        self._resp = resp

    async def __aenter__(self):
        return self._resp

    async def __aexit__(self, *_exc):
        return None


class SetupTokenTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.mod = _load_setup_token()

    def test_claim_returns_token_from_engine(self) -> None:
        session = MagicMock()
        resp = SimpleNamespace(
            status=200,
            json=AsyncMock(return_value={"token": "auto-secret", "version": "2026.11.3", "sha": "abc"}),
        )
        session.get = MagicMock(return_value=_RespCtx(resp))
        with patch.object(self.mod, "async_get_clientsession", return_value=session):
            token = asyncio.run(self.mod.async_claim_setup_token(SimpleNamespace(), "http://klar-nlu:10520"))
        self.assertEqual(token, "auto-secret")

    def test_claim_skips_errors_and_empty(self) -> None:
        session = MagicMock()
        bad = SimpleNamespace(status=404, json=AsyncMock(return_value={}))
        empty = SimpleNamespace(status=200, json=AsyncMock(return_value={"token": "  "}))
        session.get = MagicMock(side_effect=[_RespCtx(bad), _RespCtx(empty)])
        with patch.object(self.mod, "async_get_clientsession", return_value=session):
            token = asyncio.run(self.mod.async_claim_setup_token(SimpleNamespace(), "http://klar-nlu:10520"))
        self.assertIsNone(token)

    def test_config_flow_claims_before_probe(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "config_flow.py").read_text(encoding="utf-8")
        self.assertIn("async_claim_setup_token", src)
        self.assertIn("mode == MODE_REMOTE and not token", src)
        init = (ROOT / "custom_components" / "klar_nlu" / "__init__.py").read_text(encoding="utf-8")
        self.assertIn("async_claim_setup_token", init)


if __name__ == "__main__":
    unittest.main()
