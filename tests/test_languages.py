"""Generated Assist language list."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def _load_languages():
    path = ROOT / "custom_components" / "klar_nlu" / "languages.py"
    spec = importlib.util.spec_from_file_location("klar_languages", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


class LanguageListTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.langs = _load_languages()

    def test_russian_is_not_advertised(self) -> None:
        self.assertNotIn("ru", self.langs.SUPPORTED_LANGUAGES)
        self.assertNotIn("ru-RU", self.langs.SUPPORTED_LANGUAGES)

    def test_default_and_dialects_stay_distinct(self) -> None:
        self.assertIn("de", self.langs.SUPPORTED_LANGUAGES)
        self.assertIn("en", self.langs.SUPPORTED_LANGUAGES)
        self.assertIn("pt-BR", self.langs.SUPPORTED_LANGUAGES)
        self.assertIn("de-CH", self.langs.SUPPORTED_LANGUAGES)
        self.assertIn("fr", self.langs.SUPPORTED_LANGUAGES)

    def test_pt_br_is_not_a_pt_variant_only(self) -> None:
        self.assertIn("pt-BR", self.langs.LANGUAGE_VARIANTS)
        self.assertNotIn("pt-BR", self.langs.LANGUAGE_VARIANTS["pt"])


if __name__ == "__main__":
    unittest.main()
