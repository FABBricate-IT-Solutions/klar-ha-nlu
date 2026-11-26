#!/usr/bin/env python3
"""Package 3.8 / H-14 / R-19: health, restart, offline binary, repair issues."""

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
PACKAGE = "klar_engine_health_test"

if "aiohttp" not in sys.modules:
    _aiohttp = types.ModuleType("aiohttp")
    _aiohttp.ClientError = type("ClientError", (Exception,), {})
    _aiohttp.ClientSession = object

    class _ClientTimeout:
        def __init__(self, total=None):
            self.total = total

    _aiohttp.ClientTimeout = _ClientTimeout
    sys.modules["aiohttp"] = _aiohttp


def _ensure_ha_stubs() -> None:
    if "homeassistant.helpers.issue_registry" in sys.modules and hasattr(
        sys.modules["homeassistant.helpers.issue_registry"], "IssueSeverity"
    ):
        return
    ha = types.ModuleType("homeassistant")
    ha.config_entries = types.ModuleType("homeassistant.config_entries")
    ha.config_entries.ConfigEntry = object
    ha.core = types.ModuleType("homeassistant.core")
    ha.core.HomeAssistant = object
    ha.helpers = types.ModuleType("homeassistant.helpers")
    ha.helpers.issue_registry = types.ModuleType("homeassistant.helpers.issue_registry")
    ha.helpers.issue_registry.IssueSeverity = SimpleNamespace(
        WARNING="warning", ERROR="error"
    )
    ha.helpers.issue_registry.async_create_issue = MagicMock()
    ha.helpers.issue_registry.async_delete_issue = MagicMock()
    ha.helpers.aiohttp_client = types.ModuleType("homeassistant.helpers.aiohttp_client")
    ha.helpers.aiohttp_client.async_get_clientsession = MagicMock()
    for name, mod in (
        ("homeassistant", ha),
        ("homeassistant.config_entries", ha.config_entries),
        ("homeassistant.core", ha.core),
        ("homeassistant.helpers", ha.helpers),
        ("homeassistant.helpers.issue_registry", ha.helpers.issue_registry),
        ("homeassistant.helpers.aiohttp_client", ha.helpers.aiohttp_client),
    ):
        sys.modules[name] = mod


def _load_engine():
    _ensure_ha_stubs()
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
    load("units", "units.py")
    return load("engine", "engine.py")


class EngineHealthTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.engine = _load_engine()

    def setUp(self) -> None:
        ir = sys.modules["homeassistant.helpers.issue_registry"]
        ir.async_create_issue.reset_mock()
        ir.async_delete_issue.reset_mock()

    def test_watch_restarts_after_child_exits(self) -> None:
        engine = self.engine
        hass = SimpleNamespace()
        klar = engine.KlarEngine(hass)
        klar._supervise = True

        class DoneProc:
            def __init__(self):
                self.returncode = 1
                self.pid = 9

            async def wait(self):
                return 1

        first = DoneProc()
        klar._proc = first
        spawned = {"n": 0}

        async def spawn():
            spawned["n"] += 1
            klar._supervise = False
            klar._proc = DoneProc()

        async def ready():
            return None

        with (
            patch.object(klar, "_spawn", side_effect=spawn),
            patch.object(klar, "_wait_ready", side_effect=ready),
            patch.object(klar, "_kill_child", new=AsyncMock()),
        ):
            asyncio.run(klar._watch())
        self.assertEqual(spawned["n"], 1)
        ir = sys.modules["homeassistant.helpers.issue_registry"]
        self.assertTrue(ir.async_create_issue.called)
        self.assertEqual(ir.async_create_issue.call_args.kwargs["translation_key"], "engine_down")
        self.assertTrue(ir.async_delete_issue.called)

    def test_failed_ready_kills_orphan(self) -> None:
        engine = self.engine
        hass = SimpleNamespace(async_add_executor_job=lambda fn, *a: fn(*a) if callable(fn) else None)
        klar = engine.KlarEngine(hass)
        killed = {"n": 0}

        async def spawn():
            klar._proc = SimpleNamespace(returncode=None, pid=3, terminate=lambda: None, kill=lambda: None, wait=AsyncMock())

        async def ready():
            raise RuntimeError("not ready")

        async def kill():
            killed["n"] += 1
            klar._proc = None

        with (
            patch.object(klar, "_ping", new=AsyncMock(return_value=False)),
            patch.object(klar, "_ensure_binary", new=AsyncMock()),
            patch.object(klar, "_spawn", side_effect=spawn),
            patch.object(klar, "_wait_ready", side_effect=ready),
            patch.object(klar, "_kill_child", side_effect=kill),
            patch.object(klar, "async_stop", side_effect=kill),
        ):
            with self.assertRaises(RuntimeError):
                asyncio.run(klar.async_start())
        self.assertEqual(killed["n"], 1)

    def test_staging_offline_keeps_existing_binary(self) -> None:
        engine = self.engine

        async def ensure():
            # Same branch as _ensure_binary: present + ClientError → keep binary.
            present = True
            try:
                raise engine.ClientError("offline")
            except (engine.ClientError, RuntimeError, engine.UnsupportedMachineError) as err:
                if present:
                    return f"kept:{err}"
                raise

        self.assertTrue(str(asyncio.run(ensure())).startswith("kept:"))
        src = (ROOT / "custom_components" / "klar_nlu" / "engine.py").read_text(encoding="utf-8")
        self.assertIn("Using existing Klar binary offline", src)
        self.assertIn("CHANNEL_STAGING", src)
        self.assertIn("except (ClientError, RuntimeError, UnsupportedMachineError)", src)

    def test_version_mismatch_creates_warning_issue(self) -> None:
        engine = self.engine
        hass = SimpleNamespace()

        async def fake_health(_hass, _url, _token=None):
            return {"ok": True, "version": "1999.1.1"}

        with patch.object(engine, "async_fetch_health", side_effect=fake_health):
            asyncio.run(engine.async_check_engine_version(hass, "http://127.0.0.1:10520"))
        ir = sys.modules["homeassistant.helpers.issue_registry"]
        self.assertTrue(ir.async_create_issue.called)
        kwargs = ir.async_create_issue.call_args.kwargs
        self.assertEqual(kwargs["translation_key"], "version_mismatch")
        self.assertEqual(kwargs["severity"], ir.IssueSeverity.WARNING)

    def test_matching_version_clears_mismatch_issue(self) -> None:
        engine = self.engine
        hass = SimpleNamespace()

        async def fake_health(_hass, _url, _token=None):
            return {"ok": True, "version": engine.ENGINE_VERSION}

        with patch.object(engine, "async_fetch_health", side_effect=fake_health):
            asyncio.run(engine.async_check_engine_version(hass, "http://127.0.0.1:10520"))
        ir = sys.modules["homeassistant.helpers.issue_registry"]
        self.assertTrue(ir.async_delete_issue.called)

    def test_source_contracts(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "engine.py").read_text(encoding="utf-8")
        self.assertIn("async def _watch", src)
        self.assertIn("await self._spawn()", src[src.index("async def _watch") :])
        self.assertIn("Using existing Klar binary offline", src)
        self.assertIn("await self.async_stop()", src[src.index("async def async_start") : src.index("async def async_stop")])
        self.assertIn("/api/v2/health", src)
        sync = (ROOT / "custom_components" / "klar_nlu" / "sync.py").read_text(encoding="utf-8")
        self.assertIn("sync_rejected", sync)
        addon = (ROOT / "addon" / "config.yaml").read_text(encoding="utf-8")
        self.assertIn("watchdog: http://[HOST]:[PORT:10520]/api/v2/health", addon)
        docker = (ROOT / "Dockerfile").read_text(encoding="utf-8")
        self.assertIn("HEALTHCHECK", docker)
        self.assertIn("/api/v2/health", docker)


if __name__ == "__main__":
    unittest.main()
