#!/usr/bin/env python3
"""Reject and empty intents use the fallback agent without NLU-RAG."""

from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class ConversationFallbackTests(unittest.TestCase):
    def test_reject_does_not_require_nlu_rag(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(encoding="utf-8")
        start = src.index("if (\n            decision_type == \"chat\"\n            and not skips_llm_fallback(hit)")
        end = src.index("fallback = await self._fallback", start)
        block = src[start:end]
        self.assertNotIn("self._fallback_agent_id()", block)
        self.assertIn("keeps_engine_chat(hit, chat, engine_speech)", block)
        self.assertIn("decision_type == \"chat\"", block)
        self.assertNotIn("_nlu_rag()", block)
        self.assertNotIn("decision_type != \"reject\"", block)

    def test_fallback_uses_engine_assist_only(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(encoding="utf-8")
        start = src.index("async def _fallback")
        end = src.index("def _preferred_area")
        body = src[start:end]
        self.assertIn("stream_engine_assist", body)
        self.assertIn("stream_assist_with_ha_tools", body)
        self.assertIn("self._allow_llm_tools()", body)
        self.assertIn("async_provide_llm_data", (ROOT / "custom_components" / "klar_nlu" / "assist_tools.py").read_text(encoding="utf-8"))
        self.assertIn("speak_tag(pack)", src)
        self.assertIn("_llm_session_id", body)
        self.assertNotIn("async_converse", body)
        self.assertNotIn("_fallback_agent_id", src)
        self.assertNotIn("chat_only_prompt", body)
        self.assertNotIn("refine_prompt(", body)
        self.assertNotIn("history_prompt", body)
        self.assertNotIn("yarn_nudge", body)
        self.assertNotIn("yarn_asks_permission", body)
        self.assertNotIn("stream_engine_chat", body)
        self.assertNotIn("stream_chat", body)
        self.assertNotIn("_stream_fallback", body)
        self.assertNotIn("yarn_canned", src)
        self.assertIn("leaks_klar_tools", src)
        self.assertIn("parse_tool_reply", src)
        self.assertIn("_attr_supports_streaming = True", src)

    def test_calendar_queries_can_go_to_llm(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(
            encoding="utf-8"
        )
        start = src.index("executed = await execute_plan")
        end = src.index("if self._quiet_ack()", start)
        block = src[start:end]
        self.assertIn("calendar_query_only(plan)", block)
        self.assertIn("self._calendar_llm()", block)
        self.assertIn('executed.get("outcome") != "error"', block)
        self.assertIn("calendar_readback(pack, speech)", block)
        self.assertIn('kind="calendar"', block)
        self.assertNotIn("calendar_prompt", block)
        self.assertNotIn("keeps_calendar_reply", block)
        self.assertIn("if llm.strip():", block)
        self.assertIn("_was_published(fallback)", block)
        self.assertIn('"llm"', block)

    def test_home_execute_skips_sentence_triggers(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(
            encoding="utf-8"
        )
        start = src.index("async def _async_handle_message")
        end = src.index("if payload.get(\"briefing\"):", start)
        block = src[start:end]
        self.assertLess(block.index("await self._parse("), block.index("_sentence_triggers"))
        self.assertIn('if payload.get("unreachable"):', block)
        self.assertNotIn('if decision_type != "execute":', block)
        self.assertIn("keep_lab_plan(", src)
        self.assertIn('decision_type != "execute"', src[src.index("if hit == \"llm\"") : src.index("if (\n            decision_type == \"chat\"")])

    def test_execute_closes_conversation_and_keeps_stable_session(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(
            encoding="utf-8"
        )
        self.assertIn("keeps_conversation(decision_type)", src)
        self.assertIn("parse_session_id(", src)
        self.assertIn("engine_session_id", src)
        spoken = src[src.index("if self._quiet_ack()") : src.index("async def _after_fallback")]
        self.assertIn(', False, "chime"', spoken)
        self.assertNotIn(', True, "execute"', src)
        self.assertNotIn("HassVacuumReturnToBase", src[src.index("if decision_type == \"execute\"") : src.index("if self._quiet_ack()")])


if __name__ == "__main__":
    unittest.main()
