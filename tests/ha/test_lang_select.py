#!/usr/bin/env python3
"""Language choice: system default, all locales, or one pack."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = "klar_lang_select_test"


def _load(name: str, rel: str) -> types.ModuleType:
    path = ROOT / "custom_components" / "klar_nlu" / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def _load_lang_select() -> types.ModuleType:
    package = types.ModuleType(PACKAGE)
    package.__path__ = []
    with patch.dict(sys.modules, {PACKAGE: package}):
        _load(f"{PACKAGE}.languages", "languages.py")
        _load(f"{PACKAGE}.const", "const.py")
        return _load(f"{PACKAGE}.lang_select", "lang_select.py")


lang = _load_lang_select()


class LangSelectTests(unittest.TestCase):
    def test_missing_keeps_all_compiled_packs(self) -> None:
        packs = lang.enabled_packs(None, "de")
        self.assertIn("de", packs)
        self.assertIn("fr", packs)
        self.assertGreater(len(packs), 2)

    def test_system_follows_home_assistant_language(self) -> None:
        self.assertEqual(lang.enabled_packs("system", "de"), ["de"])
        self.assertEqual(lang.enabled_packs("system", "en-GB"), ["en-GB"])
        self.assertEqual(lang.enabled_packs(["system"], "fr"), ["fr"])

    def test_one_pack_and_legacy_lists(self) -> None:
        self.assertEqual(lang.enabled_packs("nl"), ["nl"])
        self.assertEqual(lang.enabled_packs(["nl"]), ["nl"])
        self.assertEqual(lang.normalize_language_choice(["de", "en"]), lang.LANGUAGE_ALL)
        self.assertEqual(lang.enabled_packs(["de", "en"])[0], "de")
        self.assertGreater(len(lang.enabled_packs(["de", "en"])), 2)

    def test_empty_list_is_all(self) -> None:
        self.assertEqual(lang.normalize_language_choice([]), lang.LANGUAGE_ALL)

    def test_selected_klar_pack_wins_over_assist_language(self) -> None:
        self.assertEqual(lang.resolve_pack("de", ["en"]), "en")
        self.assertEqual(lang.resolve_pack("de-DE", ["en"]), "en")
        self.assertEqual(lang.resolve_pack("de", ["de", "en"]), "de")
        self.assertEqual(lang.resolve_pack("en-GB", ["en-GB"]), "en-GB")
        self.assertEqual(lang.speak_tag("en"), "en")
        self.assertEqual(lang.speak_tag("zh-CN"), "zh-CN")
        lock = lang.language_lock("en")
        self.assertIn("English", lock)
        self.assertIn("German", lock)
        self.assertIn("Deutsch", lang.language_lock("de"))

    def test_engine_follows_integration_language(self) -> None:
        self.assertEqual(lang.engine_language_state("fr", "de"), (["fr"], "de"))
        self.assertEqual(lang.engine_language_state("en", "de"), (["en"], "de"))
        self.assertEqual(lang.engine_language_state("de", "en"), (["de"], "en"))
        self.assertEqual(lang.engine_language_state("system", "en-GB"), (["en-GB"], "en-GB"))
        pinned, chrome = lang.engine_language_state("all", "nl")
        self.assertEqual(pinned, [])
        self.assertEqual(chrome, "nl")
        pinned, chrome = lang.engine_language_state("all", "zh-CN")
        self.assertEqual(pinned, [])
        self.assertEqual(chrome, "zh-CN")
        self.assertEqual(lang.chrome_locale("de"), "de")
        self.assertEqual(lang.chrome_locale("en-GB"), "en-GB")


if __name__ == "__main__":
    unittest.main()
