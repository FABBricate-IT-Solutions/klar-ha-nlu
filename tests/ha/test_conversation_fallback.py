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


if __name__ == "__main__":
    unittest.main()
