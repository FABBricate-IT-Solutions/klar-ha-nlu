#!/usr/bin/env python3
"""Assist conversation_id is isolated; klar-followup is only the fallback."""

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
    def test_assist_id_wins_over_device_and_followup(self) -> None:
        self.assertEqual(const.parse_session_id("assist-a", None, None), "assist-a")
        self.assertEqual(const.parse_session_id("  assist-b  ", "dev-1", None), "assist-b")
        self.assertEqual(const.parse_session_id("chat-1", None, "sat-1"), "chat-1")
        self.assertEqual(const.parse_session_id(None, None, None), const.FOLLOWUP_SESSION)
        self.assertEqual(const.parse_session_id("", None, None), const.FOLLOWUP_SESSION)
        self.assertEqual(const.parse_session_id("   ", "dev-1", None), "dev:dev-1")
        self.assertEqual(const.parse_session_id(None, None, "sat-1"), "dev:sat-1")
        self.assertNotEqual(const.parse_session_id("chat-a", None, None), const.parse_session_id("chat-b", None, None))

    def test_parse_forwards_assist_conversation_id(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(encoding="utf-8")
        start = src.index("async def _parse")
        body = src[start : src.index("def _llm_session_id")]
        self.assertIn("parse_session_id(conversation_id, device_id, satellite_id)", body)
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
