"""Every Assist locale has a Home Assistant UI translation with the same keys."""

from __future__ import annotations

import ast
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HA = ROOT / "custom_components" / "klar_nlu"
STRINGS = HA / "strings.json"
TRANSLATIONS = HA / "translations"


def _flatten(obj: object, prefix: str = "") -> set[str]:
    if isinstance(obj, dict):
        out: set[str] = set()
        for key, value in obj.items():
            path = f"{prefix}.{key}" if prefix else str(key)
            out.update(_flatten(value, path))
        return out
    return {prefix}


def _ha_file(code: str) -> str | None:
    if code == "en":
        return None
    if code == "zh-CN":
        return "zh-Hans"
    if code == "zh-TW":
        return "zh-Hant"
    return code


def _supported() -> tuple[str, ...]:
    tree = ast.parse((HA / "languages.py").read_text(encoding="utf-8"))
    for node in tree.body:
        if isinstance(node, ast.Assign):
            names = [target.id for target in node.targets if isinstance(target, ast.Name)]
            if "SUPPORTED_LANGUAGES" in names:
                return ast.literal_eval(node.value)
    raise AssertionError("SUPPORTED_LANGUAGES missing")


class TranslationParity(unittest.TestCase):
    def test_every_assist_locale_has_matching_ui_keys(self) -> None:
        english = _flatten(json.loads(STRINGS.read_text(encoding="utf-8")))
        expected = {_ha_file(code) for code in _supported()} - {None}
        on_disk = {path.stem for path in TRANSLATIONS.glob("*.json")}
        self.assertEqual(expected, on_disk)
        for name in sorted(expected):
            keys = _flatten(json.loads((TRANSLATIONS / f"{name}.json").read_text(encoding="utf-8")))
            self.assertEqual(english, keys, name)


if __name__ == "__main__":
    unittest.main()
