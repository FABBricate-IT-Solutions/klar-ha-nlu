#!/usr/bin/env python3
"""Reject and empty intents use the fallback agent without NLU-RAG."""

from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class ConversationFallbackTests(unittest.TestCase):
    def test_reject_does_not_require_nlu_rag(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(encoding="utf-8")
        start = src.index("if (\n            not skips_llm_fallback(hit)")
        end = src.index("fallback = await self._fallback", start)
        block = src[start:end]
        self.assertIn("self._fallback_agent_id()", block)
        self.assertNotIn("_nlu_rag()", block)
        self.assertNotIn("decision_type != \"reject\"", block)

    def test_fallback_prompt_is_chat_only_without_rag(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(encoding="utf-8")
        start = src.index("async def _fallback")
        end = src.index("def _preferred_area")
        body = src[start:end]
        self.assertIn("chat_only_prompt", body)
        self.assertIn("refine_prompt", body)
        self.assertIn("speak_tag(pack)", body)
        self.assertIn("history_prompt", body)
        self.assertIn("_llm_session_id", body)
        self.assertNotIn("user_input.language", body)

    def test_calendar_queries_can_go_to_llm(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(
            encoding="utf-8"
        )
        start = src.index("executed = await execute_plan")
        end = src.index("if self._quiet_ack()", start)
        block = src[start:end]
        self.assertIn("calendar_query_only(plan)", block)
        self.assertIn("calendar_prompt(pack, speech", block)
        self.assertIn("self._calendar_llm()", block)

    def test_execute_keeps_conversation_and_stable_session(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(
            encoding="utf-8"
        )
        self.assertIn("keeps_conversation(decision_type)", src)
        self.assertIn("parse_session_id(", src)
        self.assertIn("engine_session_id", src)


if __name__ == "__main__":
    unittest.main()
