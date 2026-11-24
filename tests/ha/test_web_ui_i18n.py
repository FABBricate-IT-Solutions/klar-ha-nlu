"""Every Assist locale has operator UI chrome with the same keys as English."""

from __future__ import annotations

import ast
import json
import re
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
if str(ROOT / "scripts") not in sys.path:
    sys.path.insert(0, str(ROOT / "scripts"))
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
        english_hint = "Voice, languages, and the LLM live here. Home Assistant only connects the engine."
        for code in sorted(expected):
            payload = json.loads((MESSAGES / f"{code}.json").read_text(encoding="utf-8"))
            self.assertEqual(set(english), set(payload), code)
            self.assertIn("{room}", payload["tryOn"], code)
            self.assertIn("{{ text }}", payload["payloadTemplate"], code)
            self.assertIn("{count}", payload["applyDone"], code)
            self.assertNotIn("Home Assistant → Klar NLU", payload["personalityHa"], code)
            self.assertNotIn("Mode binds devices or rooms only", payload["engineHint"], code)
            self.assertNotEqual(payload["processPath"], "conversation.process", code)
            self.assertIn("{count}", payload["lexiconOverlayPlus"], code)
            if code != "en-GB":
                self.assertNotEqual(payload["engineHint"], english_hint, code)
                self.assertNotEqual(payload["laneTabs"], "Lanes", code)
                self.assertNotEqual(
                    payload["governEmpty"],
                    "Safety seeds ship with every pack. Off writes a house override; the compiled floor stays on.",
                    code,
                )

    def test_wizard_chrome_is_translated(self) -> None:
        wizard = ROOT / "web" / "src" / "i18n" / "wizard"
        expected = set(_supported()) - {"de", "en"}
        on_disk = {path.stem for path in wizard.glob("*.json")}
        self.assertEqual(expected, on_disk)
        english_console = (
            "Lovelace “Klar” is the last Assist turn. This surface (Klar NLU) is the operator console: Settings, House, Lab, and Rules."
        )
        english_llm = (
            "Assist chat, refine, and the trainer live in Settings, not in a Home Assistant conversation integration."
        )
        for code in sorted(expected):
            payload = json.loads((wizard / f"{code}.json").read_text(encoding="utf-8"))
            self.assertIn("whatConsole", payload, code)
            self.assertIn("missLlmBody", payload, code)
            self.assertIn("{count}", payload["phrasesMapping"], code)
            if code != "en-GB":
                self.assertNotEqual(payload["whatConsole"], english_console, code)
                self.assertNotEqual(payload["missLlmBody"], english_llm, code)

    def test_engine_serves_what_the_operator_ui_relies_on(self) -> None:
        # The UI's own behaviour is tested in web/tests (Vitest) and web/tests/e2e (Playwright).
        # What is left here are the engine and integration sources the UI depends on.
        engine_llm_rs = (ROOT / "src" / "io" / "llm.rs").read_text(encoding="utf-8")
        self.assertIn("/api/v2/llm/models", engine_llm_rs)
        self.assertIn("/api/v2/llm/voice", engine_llm_rs)
        overlay = (ROOT / "src" / "home" / "overlay.rs").read_text(encoding="utf-8")
        dashboard = (ROOT / "src" / "io" / "dashboard.rs").read_text(encoding="utf-8")
        self.assertIn("locale_set", overlay)
        self.assertIn("KLAR_UI_LOCALE", dashboard)
        self.assertNotIn("locale_from_accept_language", dashboard)
        self.assertNotIn("accept-language", dashboard)
        conversation = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(
            encoding="utf-8"
        )
        self.assertIn("advertised_languages()", conversation)

    def test_llm_chrome_is_translated(self) -> None:
        from lang_packs.web_ui_keys import FALLBACKS
        from lang_packs.web_ui_llm import CHECK_KEYS, KEYS

        expected = set(_supported()) - {"de", "en"}
        for code in sorted(expected):
            payload = json.loads((MESSAGES / f"{code}.json").read_text(encoding="utf-8"))
            for key in KEYS:
                value = payload[key]
                self.assertNotIn("\ufffd", value, f"{code}.{key}")
                self.assertTrue(value.strip(), f"{code}.{key}")
            if code == "en-GB":
                continue
            for key in CHECK_KEYS:
                self.assertNotEqual(payload[key], FALLBACKS[key], f"{code}.{key}")

    def test_settings_hints_are_translated(self) -> None:
        from lang_packs.settings_hints import KEYS
        from lang_packs.web_ui_keys import FALLBACKS

        english = {key: FALLBACKS[key] for key in KEYS}
        loanwords = {"refineBandStatus": {"Status"}}
        expected = set(_supported()) - {"de", "en"}
        for code in sorted(expected):
            payload = json.loads((MESSAGES / f"{code}.json").read_text(encoding="utf-8"))
            for key in KEYS:
                value = payload[key]
                self.assertNotIn("\ufffd", value, f"{code}.{key}")
                self.assertTrue(value.strip(), f"{code}.{key}")
                if code == "en-GB":
                    continue
                if value in loanwords.get(key, ()):
                    continue
                self.assertNotEqual(value, english[key], f"{code}.{key}")
