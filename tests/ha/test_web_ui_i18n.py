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

    def test_wizard_writes_engine_settings(self) -> None:
        wizard = (ROOT / "web" / "src" / "pages" / "Wizard.tsx").read_text(encoding="utf-8")
        app = (ROOT / "web" / "src" / "App.tsx").read_text(encoding="utf-8")
        self.assertIn("api.saveSettings", wizard)
        self.assertIn("api.saveLlmEndpoint", wizard)
        self.assertIn("chrome={t}", app)
        self.assertIn("onSettings={setSettings}", app)

    def test_operator_chrome_follows_saved_ui_not_nlu_pin(self) -> None:
        i18n = (ROOT / "web" / "src" / "i18n.ts").read_text(encoding="utf-8")
        app = (ROOT / "web" / "src" / "App.tsx").read_text(encoding="utf-8")
        self.assertIn("export function chromeLocale(saved?: string)", i18n)
        self.assertIn("export function assistParseLanguage(languages: string[], chrome?: string)", i18n)
        self.assertIn("return languages.length === 1 ? languages[0] : chrome", i18n)
        self.assertIn("chromeLocale(ui.locale)", app)
        self.assertIn("assistParseLanguage(settings.languages, locale)", app)
        self.assertNotIn("chromeLocale(settings.languages", app)
        self.assertNotIn('locale: "de"', app)
        self.assertIn("assistParseLanguage", app)
        self.assertIn("onLocale", app)
        i18n_src = (ROOT / "web" / "src" / "i18n.ts").read_text(encoding="utf-8")
        self.assertNotIn("navigator.language", i18n_src)
        self.assertIn("assistParseLanguage", i18n_src)
        lab = (ROOT / "web" / "src" / "pages" / "ParsePage.tsx").read_text(encoding="utf-8")
        self.assertNotIn("HA trigger", lab)
        self.assertNotIn("dispatch / intent_script", lab)
        self.assertIn("Klar parse", lab)
        self.assertIn("armedPipeline", lab)
        self.assertIn("labPath", lab)
        self.assertIn("LLM refine", lab)
        self.assertIn("calendar LLM", lab)
        self.assertIn("quiet ack", lab)
        self.assertIn("LLM tools", lab)
        self.assertIn("fallback LLM", lab)
        self.assertIn("NLU-RAG", lab)
        self.assertIn("aria-label=\"pipeline\"", lab)
        self.assertIn("policy_trace?.hit", lab)
        en = EN.read_text(encoding="utf-8")
        self.assertIn("Lab is the Assist path for the selected language", en)
        self.assertIn("Sentence triggers run only if Klar is unreachable", en)
        self.assertNotIn("trigger, then Klar, then intent_script", en)
        self.assertNotIn("when this parse is not execute", en)
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
