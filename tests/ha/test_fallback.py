#!/usr/bin/env python3
"""Stdlib tests for chat-only fallback gating."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _load_module(name: str, rel: str):
    path = ROOT / "custom_components" / "klar_nlu" / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


def _load():
    _load_module("calendar_say", "calendar_say.py")
    return _load_module("klar_fallback", "fallback.py")


fallback = _load()


class FallbackTests(unittest.TestCase):
    def test_control_flag_blocks(self) -> None:
        self.assertTrue(fallback.agent_has_home_control(1))
        self.assertTrue(fallback.agent_has_home_control(3))

    def test_chat_only_agent_allowed(self) -> None:
        self.assertFalse(fallback.agent_has_home_control(0))

    def test_unknown_features_fail_closed(self) -> None:
        self.assertTrue(fallback.agent_has_home_control("nope"))

    def test_prompt_appends_chat_only(self) -> None:
        prompt = fallback.chat_only_prompt("de", "Sei kurz.")
        self.assertIn("Sei kurz.", prompt)
        self.assertIn("keine Home-Assistant-Werkzeuge", prompt)

    def test_unknown_pack_does_not_default_to_german(self) -> None:
        prompt = fallback.chat_only_prompt("sw", None)
        self.assertNotIn("Steuere keine Geräte", prompt)
        self.assertIn("Assist pack code: sw", prompt)
        self.assertIn("Do not translate into German", prompt)
        ja = fallback.chat_only_prompt("ja", None)
        self.assertIn("会話", ja)
        self.assertNotIn("Steuere keine Geräte", ja)
        news = fallback.news_prompt("ja", ["One"], None)
        self.assertNotIn("Schlagzeilen", news)

    def test_personality_leads_open_chat_system_prompt(self) -> None:
        prompt = fallback.with_personality(
            "Du antwortest nur im Gespräch.",
            "Stimme: Jarvis.\nBeispiele:\nX → Y",
        )
        self.assertTrue(prompt.startswith("Stimme: Jarvis."))
        self.assertIn("nur im Gespräch", prompt)
        self.assertEqual(fallback.with_personality("", "Stimme: Butler."), "Stimme: Butler.")
        self.assertEqual(fallback.with_personality("Nur reden.", None), "Nur reden.")

    def test_control_agent_never_used_as_fallback(self) -> None:
        self.assertTrue(fallback.can_use_fallback_agent(False, False))
        self.assertFalse(fallback.can_use_fallback_agent(True, False))
        self.assertFalse(fallback.can_use_fallback_agent(True, True))

    def test_calendar_query_only_is_list_intent(self) -> None:
        self.assertTrue(
            fallback.calendar_query_only([{"name": "KlarGetCalendarEvents"}])
        )
        self.assertFalse(
            fallback.calendar_query_only([{"name": "KlarCreateCalendarEvent"}])
        )
        self.assertFalse(fallback.calendar_query_only([]))
        self.assertFalse(fallback.calendar_query_only(None))

    def test_calendar_prompt_keeps_facts_and_language(self) -> None:
        prompt = fallback.calendar_prompt("en", "dentist tomorrow at 3", None)
        self.assertIn("dentist tomorrow at 3", prompt)
        self.assertIn("Do not invent events", prompt)
        self.assertIn("Do not translate into German", prompt)
        ja = fallback.calendar_prompt("ja", "meeting", None)
        self.assertNotIn("Termine", ja)
        self.assertNotIn("Assist pack code:", ja)
        self.assertIn("予定", ja)

    def test_every_pack_has_native_calendar_prompt(self) -> None:
        languages = _load_module("klar_languages", "languages.py")
        say = _load_module("calendar_say", "calendar_say.py")
        self.assertEqual(set(say.LLM), set(languages.SUPPORTED_LANGUAGES))
        self.assertEqual(set(say.SAY), set(languages.SUPPORTED_LANGUAGES))
        for pack in languages.SUPPORTED_LANGUAGES:
            ask, label = say.llm_copy(pack)
            prompt = fallback.calendar_prompt(pack, "probe-event", None)
            self.assertTrue(ask.strip(), pack)
            self.assertTrue(label.strip(), pack)
            self.assertIn("probe-event", prompt)
            self.assertIn(label, prompt)
            self.assertNotIn("Assist pack code:", prompt)
            if pack not in {"de", "de-AT", "de-CH", "lb"}:
                self.assertNotIn("Termine", prompt)
            if pack not in {"en", "en-GB"}:
                self.assertNotEqual(ask, say.LLM["en"][0], pack)


if __name__ == "__main__":
    unittest.main()
