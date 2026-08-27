#!/usr/bin/env python3
"""Stdlib tests for chat-only fallback gating."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _load():
    path = ROOT / "custom_components" / "klar_nlu" / "fallback.py"
    spec = importlib.util.spec_from_file_location("klar_fallback", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


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
        prompt = fallback.chat_only_prompt("ja", None)
        self.assertNotIn("Steuere keine Geräte", prompt)
        self.assertIn("Assist pack code: ja", prompt)
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


if __name__ == "__main__":
    unittest.main()
