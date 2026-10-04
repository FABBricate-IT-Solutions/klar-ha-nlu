#!/usr/bin/env python3
"""ASR boost export glue for wyoming-faster-whisper."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
PKG = ROOT / "custom_components" / "klar_nlu"
PACKAGE = "klar_asr_boost_test"


def _module(name: str) -> types.ModuleType:
    module = types.ModuleType(name)
    module.__path__ = []
    return module


def _load_asr_boost() -> types.ModuleType:
    homeassistant = _module("homeassistant")
    config_entries = types.ModuleType("homeassistant.config_entries")
    config_entries.ConfigEntry = object
    core = types.ModuleType("homeassistant.core")
    core.HomeAssistant = object
    helpers = _module("homeassistant.helpers")
    aiohttp_client = types.ModuleType("homeassistant.helpers.aiohttp_client")
    aiohttp_client.async_get_clientsession = lambda _hass: None
    aiohttp = types.ModuleType("aiohttp")
    aiohttp.ClientError = Exception
    aiohttp.ClientTimeout = lambda **_kwargs: None
    modules = {
        "aiohttp": aiohttp,
        "homeassistant": homeassistant,
        "homeassistant.config_entries": config_entries,
        "homeassistant.core": core,
        "homeassistant.helpers": helpers,
        "homeassistant.helpers.aiohttp_client": aiohttp_client,
    }
    with patch.dict(sys.modules, modules):
        languages = types.ModuleType(f"{PACKAGE}.languages")
        languages.LANGUAGE_VARIANTS = {}
        languages.SUPPORTED_LANGUAGES = ["de", "en"]
        sys.modules[f"{PACKAGE}.languages"] = languages
        const_path = PKG / "const.py"
        spec = importlib.util.spec_from_file_location(f"{PACKAGE}.const", const_path)
        assert spec and spec.loader
        const = importlib.util.module_from_spec(spec)
        sys.modules[f"{PACKAGE}.const"] = const
        spec.loader.exec_module(const)
        path = PKG / "asr_boost.py"
        spec = importlib.util.spec_from_file_location(f"{PACKAGE}.asr_boost", path)
        assert spec and spec.loader
        mod = importlib.util.module_from_spec(spec)
        sys.modules[f"{PACKAGE}.asr_boost"] = mod
        # Rewrite relative import target
        mod.__package__ = PACKAGE
        spec.loader.exec_module(mod)
        return mod


asr_boost = _load_asr_boost()


class AsrBoostExportTests(unittest.TestCase):
    def test_boost_path_under_config_dir(self) -> None:
        hass = types.SimpleNamespace(config=types.SimpleNamespace(config_dir="/tmp/ha-config"))
        self.assertEqual(asr_boost.boost_path(hass, None), Path("/tmp/ha-config/klar_nlu_asr_boost.txt"))
        self.assertEqual(asr_boost.boost_path(hass, "custom.txt"), Path("/tmp/ha-config/custom.txt"))
        self.assertEqual(asr_boost.boost_path(hass, "/abs/prompt.txt"), Path("/abs/prompt.txt"))

    def test_write_prompt_creates_file(self) -> None:
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "klar_nlu_asr_boost.txt"
            asr_boost._write_prompt(path, "Wohnzimmer Vorhang Rollo")
            self.assertEqual(path.read_text(encoding="utf-8").strip(), "Wohnzimmer Vorhang Rollo")

    def test_services_and_sync_wire_export(self) -> None:
        src = (PKG / "services.py").read_text(encoding="utf-8")
        self.assertIn("export_asr_boost", src)
        self.assertIn("SERVICE_EXPORT_ASR_BOOST", src)
        yaml = (PKG / "services.yaml").read_text(encoding="utf-8")
        self.assertIn("export_asr_boost:", yaml)
        sync = (PKG / "sync.py").read_text(encoding="utf-8")
        self.assertIn("_refresh_asr_boost", sync)
        engine = (ROOT / "src" / "io" / "speech.rs").read_text(encoding="utf-8")
        self.assertIn("/api/v2/speech/asr_boost", engine)


if __name__ == "__main__":
    unittest.main()
