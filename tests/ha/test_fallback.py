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


fallback = _load_module("klar_fallback", "fallback.py")


class FallbackTests(unittest.TestCase):
    def test_control_flag_blocks(self) -> None:
        self.assertTrue(fallback.agent_has_home_control(1))
        self.assertTrue(fallback.agent_has_home_control(3))

    def test_chat_only_agent_allowed(self) -> None:
        self.assertFalse(fallback.agent_has_home_control(0))

    def test_unknown_features_fail_closed(self) -> None:
        self.assertTrue(fallback.agent_has_home_control("nope"))

    def test_control_agent_never_used_as_fallback(self) -> None:
        self.assertTrue(fallback.can_use_fallback_agent(False, False))
        self.assertFalse(fallback.can_use_fallback_agent(True, False))
        self.assertFalse(fallback.can_use_fallback_agent(True, True))
        self.assertTrue(fallback.can_use_fallback_agent(True, False, True))
        self.assertTrue(fallback.can_use_fallback_agent(True, True, True))

    def test_calendar_query_only_is_list_intent(self) -> None:
        self.assertTrue(
            fallback.calendar_query_only([{"name": "KlarGetCalendarEvents"}])
        )
        self.assertFalse(
            fallback.calendar_query_only([{"name": "KlarCreateCalendarEvent"}])
        )
        self.assertFalse(fallback.calendar_query_only([]))
        self.assertFalse(fallback.calendar_query_only(None))

    def test_calendar_readback_does_not_repeat_the_question(self) -> None:
        asked = fallback.calendar_readback("en", "dentist is tomorrow at 3.")
        self.assertIn("dentist is tomorrow at 3.", asked)
        self.assertNotIn("What's on my calendar", asked)
        self.assertTrue(asked.startswith("Read back only these calendar events"))
        de = fallback.calendar_readback("de", "")
        self.assertIn("Keine Termine", de)

    def test_llm_session_keeps_recent_turns(self) -> None:
        self.assertEqual(fallback.llm_conversation_id("chat-7"), "klar-llm-chat-7")
        # H-01: no key used to mean the history every caller without an id shared.
        self.assertNotEqual(fallback.llm_conversation_id(""), fallback.llm_conversation_id(""))
        self.assertNotEqual(fallback.llm_conversation_id("  "), "klar-llm-klar-followup")
        turns = fallback.append_llm_turn(
            None, "erzähl eine Geschichte", "Kurz oder lang?"
        )
        turns = fallback.append_llm_turn(turns, "science fiction", "Raumschiff oder KI?")
        self.assertEqual(len(turns), 2)
        self.assertEqual(turns[0][0], "erzähl eine Geschichte")
        trimmed = fallback.append_llm_turn(turns, "a", "b", keep=2)
        self.assertEqual(len(trimmed), 2)

    def test_llm_history_caps_article_length(self) -> None:
        # A-15: a 12k article must not dominate later LLM turns.
        article = "x" * 12000
        turns = fallback.append_llm_turn(None, "read it", article)
        self.assertEqual(len(turns), 1)
        self.assertLessEqual(len(turns[0][1]), fallback._MAX_TURN_CHARS)
        bloated = [(f"u{i}", "y" * fallback._MAX_TURN_CHARS) for i in range(8)]
        capped = fallback.append_llm_turn(bloated, "more", "ok")
        self.assertLessEqual(sum(len(u) + len(a) for u, a in capped), fallback._MAX_HISTORY_CHARS)

    def test_llm_history_forgets_the_oldest_conversations(self) -> None:
        store = {f"klar-llm-{index}": [("asked", "answered")] for index in range(70)}
        fallback.forget_old_llm_sessions(store)
        self.assertEqual(len(store), 64)
        self.assertNotIn("klar-llm-0", store)
        self.assertIn("klar-llm-69", store)
        fallback.forget_old_llm_sessions(store, keep=1)
        self.assertEqual(list(store), ["klar-llm-69"])


if __name__ == "__main__":
    unittest.main()
