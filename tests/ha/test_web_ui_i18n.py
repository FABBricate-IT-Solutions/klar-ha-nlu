"""Every Assist locale has operator UI chrome with the same keys as English."""

from __future__ import annotations

import ast
import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HA = ROOT / "custom_components" / "klar_nlu"
EN = ROOT / "web" / "src" / "i18n" / "en.ts"
DE = ROOT / "web" / "src" / "i18n" / "de.ts"
MESSAGES = ROOT / "web" / "src" / "i18n" / "messages"


def _keys(text: str) -> list[str]:
    return re.findall(r"^\s+(\w+):", text, re.M)


def _supported() -> tuple[str, ...]:
    tree = ast.parse((HA / "languages.py").read_text(encoding="utf-8"))
    for node in tree.body:
        if isinstance(node, ast.Assign):
            names = [target.id for target in node.targets if isinstance(target, ast.Name)]
            if "SUPPORTED_LANGUAGES" in names:
                return ast.literal_eval(node.value)
    raise AssertionError("SUPPORTED_LANGUAGES missing")


class OperatorUiParity(unittest.TestCase):
    def test_every_assist_locale_has_operator_chrome(self) -> None:
        english = _keys(EN.read_text(encoding="utf-8"))
        self.assertEqual(english, _keys(DE.read_text(encoding="utf-8")))
        self.assertIn("parseSample", english)
        self.assertIn("tryOn", english)
        on_disk = {path.stem for path in MESSAGES.glob("*.json")}
        expected = set(_supported()) - {"de", "en"}
        self.assertEqual(expected, on_disk)
        for code in sorted(expected):
            payload = json.loads((MESSAGES / f"{code}.json").read_text(encoding="utf-8"))
            self.assertEqual(set(english), set(payload), code)
            self.assertIn("{room}", payload["tryOn"], code)
            self.assertIn("{{ text }}", payload["payloadTemplate"], code)
