#!/usr/bin/env python3
"""Assist conversation_id is isolated; a turn without an id never lands in a shared session."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]


def _load_const():
    languages = types.ModuleType("klar_session_test.languages")
    languages.LANGUAGE_VARIANTS = {}
    languages.SUPPORTED_LANGUAGES = ("de", "en")
    package = types.ModuleType("klar_session_test")
    package.__path__ = []
    with patch.dict(
        sys.modules,
        {
            "klar_session_test": package,
            "klar_session_test.languages": languages,
        },
    ):
        path = ROOT / "custom_components" / "klar_nlu" / "const.py"
        spec = importlib.util.spec_from_file_location("klar_session_test.const", path)
        if spec is None or spec.loader is None:
            raise RuntimeError(f"cannot load {path}")
        module = importlib.util.module_from_spec(spec)
        sys.modules["klar_session_test.const"] = module
        spec.loader.exec_module(module)
        return module


const = _load_const()


class SessionIsolationTests(unittest.TestCase):
    def test_assist_id_wins_over_device(self) -> None:
        self.assertEqual(const.parse_session_id("assist-a", None, None), "assist-a")
        self.assertEqual(const.parse_session_id("  assist-b  ", "dev-1", None), "assist-b")
        self.assertEqual(const.parse_session_id("chat-1", None, "sat-1"), "chat-1")
        self.assertEqual(const.parse_session_id("   ", "dev-1", None), "dev:dev-1")
        self.assertEqual(const.parse_session_id(None, None, "sat-1"), "dev:sat-1")
        self.assertNotEqual(const.parse_session_id("chat-a", None, None), const.parse_session_id("chat-b", None, None))

    def test_no_id_and_no_device_is_never_a_shared_session(self) -> None:
        # H-01: these used to be the constant "klar-followup" for every caller.
        self.assertFalse(hasattr(const, "FOLLOWUP_SESSION"))
        for assist_id in (None, "", "   "):
            first = const.parse_session_id(assist_id, None, None)
            second = const.parse_session_id(assist_id, None, None)
            self.assertTrue(first)
            self.assertNotEqual(first, second)
        self.assertIsNone(const.engine_session_id(None, None))
        self.assertIsNone(const.engine_session_id("  ", ""))

    def test_chat_session_decides_when_home_assistant_opened_one(self) -> None:
        # No id from the caller: the voice device keeps its session, everyone else
        # gets the chat session Home Assistant just created.
        self.assertEqual(const.parse_session_id(None, None, None, "chat-new"), "chat-new")
        self.assertEqual(const.parse_session_id(None, None, "sat-1", "chat-new"), "dev:sat-1")
        self.assertEqual(const.parse_session_id(None, "dev-1", None, "chat-new"), "dev:dev-1")
        # An id from the caller: Home Assistant may have replaced an idle one.
        self.assertEqual(const.parse_session_id("chat-old", None, "sat-1", "chat-new"), "chat-new")
        self.assertEqual(const.parse_session_id("chat-old", None, None, None), "chat-old")
        self.assertEqual(len(const.parse_session_id("x" * 400, None, None)), 128)
        self.assertEqual(const.parse_session_id(7, None, None, "chat-new"), "chat-new")

    def test_turn_is_bound_to_the_chat_log_before_parsing(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(encoding="utf-8")
        start = src.index("async def _async_handle_message")
        head = src[start : src.index("await self._parse(", start)]
        self.assertIn('getattr(chat_log, "conversation_id", None)', head)

    def test_parse_forwards_assist_conversation_id(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(encoding="utf-8")
        start = src.index("async def _parse")
        body = src[start : src.index("def _llm_session_id")]
        self.assertIn("parse_session_id(conversation_id, device_id, satellite_id)", body)
        self.assertIn("if not self._engine_settings():", body)
        self.assertIn('body["personality"] = self._personality()', body)
        self.assertNotIn('"personality": self._personality()', body)
        self.assertIn("post_parse(", src)
        self.assertIn("finish_clock_speech", src)
        self.assertIn("keeps_engine_chat", src)
        http = (ROOT / "custom_components" / "klar_nlu" / "engine_http.py").read_text(encoding="utf-8")
        self.assertIn("engine_url_candidates(url)", http)
        self.assertIn("ClientTimeout(total=30)", http)
        self.assertNotIn("ClientTimeout(total=5)", http)
        self.assertNotIn('"conversation_id": engine_session_id(device_id, satellite_id)', body)

    def test_empty_plan_does_not_say_done(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(encoding="utf-8")
        self.assertIn("def _ack_speech(", src)
        start = src.index('engine_speech = str(payload.get("speech") or "")')
        block = src[start : src.index("clarify = decision_type")]
        self.assertIn("_ack_speech(", block)
        self.assertNotIn("engine_speech or _cue(_DONE", block)


if __name__ == "__main__":
    unittest.main()
