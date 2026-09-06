#!/usr/bin/env python3
"""Stdlib tests for LLM reply refinement helpers."""

from __future__ import annotations

import asyncio
import importlib.util
import sys
import types
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PKG = ROOT / "custom_components" / "klar_nlu"
if str(PKG) not in sys.path:
    sys.path.insert(0, str(PKG))

_pkg = types.ModuleType("klar_nlu")
_pkg.__path__ = [str(PKG)]
sys.modules.setdefault("klar_nlu", _pkg)


def _load(name: str, rel: str):
    path = PKG / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


_load("klar_nlu.languages", "languages.py")
const = _load("klar_nlu.const", "const.py")
refine = _load("klar_refine", "refine.py")
speech = _load("klar_speech", "speech.py")


class RefineTests(unittest.TestCase):
    def test_nlu_home_turn_removed_because_every_reply_refines(self) -> None:
        self.assertFalse(hasattr(refine, "nlu_home_turn"))
        self.assertFalse(hasattr(refine, "accept_refined"))
        self.assertFalse(hasattr(refine, "refine_prompt"))
        self.assertFalse(hasattr(refine, "_async_refine_raw"))

    def test_should_refine_any_spoken_reply(self) -> None:
        self.assertTrue(
            refine.should_refine(True, "conversation.llm", "Licht ist an.")
        )
        self.assertTrue(
            refine.should_refine(True, "conversation.llm", "Im Wohnzimmer sind es 21,5 °C.")
        )
        self.assertTrue(refine.should_refine(True, "conversation.llm", "Hallo"))
        self.assertTrue(refine.should_refine(True, "conversation.llm", "Die Nachrichten."))
        self.assertFalse(
            refine.should_refine(False, "conversation.llm", "Licht ist an.")
        )
        self.assertTrue(refine.should_refine(True, None, "Licht ist an."))
        self.assertFalse(refine.should_refine(True, "conversation.llm", ""))
        self.assertFalse(hasattr(refine, "_TIMEOUT"))
        self.assertFalse(hasattr(refine, "nlu_home_turn"))

    def test_options_personality_switches_style_wrap(self) -> None:
        self.assertEqual(const.resolve_personality("grantig"), "grantig")
        self.assertEqual(const.resolve_personality("nope"), "default")
        for pack in ("de", "en"):
            for name in const.PERSONALITIES:
                spoken = speech.style("Licht ist an.", name, pack)
                variants = list((speech._locale(pack).get("personality") or {}).get(name) or [""])
                expected = {f"{prefix}Licht ist an." for prefix in variants}
                expected.add("Licht ist an.")
                if name == "default":
                    self.assertEqual(spoken, "Licht ist an.")
                    continue
                self.assertTrue(variants, name)
                self.assertIn(spoken, expected, name)
        spoken = speech.style("Licht ist an.", "butler", "de")
        variants = set((speech._locale("de").get("personality") or {}).get("butler") or [])
        self.assertTrue(variants, "butler variants")
        self.assertIn(spoken, {f"{prefix}Licht ist an." for prefix in variants})

    def test_successful_refine_keeps_natural_line_without_restamping_cue(self) -> None:
        source = "Wohnzimmer Licht ist an."
        spoken = speech.style(source, "butler", "de")
        variants = set((speech._locale("de").get("personality") or {}).get("butler") or [])
        self.assertIn(spoken, {f"{prefix}{source}" for prefix in variants})

    def test_refine_calls_engine_and_fails_closed(self) -> None:
        src = (PKG / "refine.py").read_text(encoding="utf-8")
        self.assertIn("complete_engine_refine", src)
        self.assertIn("conversation_id=conversation_id", src)
        self.assertNotIn("complete_engine_chat", src)
        self.assertNotIn("accept_refined", src)
        self.assertNotIn("async_converse", src)
        self.assertNotIn("chat.completions.create", src)
        self.assertIn("Cycle: engine_llm → stream → refine", src)

    def test_no_homeassistant_runtime_falls_back_to_none(self) -> None:
        out = asyncio.run(
            refine.async_refine_speech(
                None,
                "conversation.llm",
                True,
                "Wohnzimmer Licht ist an.",
                None,
                "de",
                "de",
                "butler",
                None,
            )
        )
        self.assertIsNone(out)

    def test_skip_rewrite_for_llm_replies_only(self) -> None:
        self.assertTrue(refine.skip_rewrite("chat"))
        self.assertTrue(refine.skip_rewrite("llm"))
        self.assertTrue(refine.skip_rewrite("chime"))
        self.assertTrue(refine.skip_rewrite("error"))
        self.assertFalse(refine.skip_rewrite("execute"))
        self.assertFalse(refine.skip_rewrite("clarify"))
        self.assertFalse(refine.skip_rewrite("trigger"))
        self.assertFalse(refine.skip_rewrite(""))

    def test_isolated_ids_are_unique_and_prefixed(self) -> None:
        first = refine.isolated_conversation_id()
        second = refine.isolated_conversation_id()
        self.assertTrue(first.startswith("klar-nested-"))
        self.assertTrue(second.startswith("klar-nested-"))
        self.assertNotEqual(first, second)

    def test_nested_llm_session_never_targets_satellite(self) -> None:
        session = refine.nested_llm_session("conversation.llm", "de", "Stimme: Jarvis.")
        self.assertIsNone(session["device_id"])
        self.assertIsNone(session["satellite_id"])
        self.assertEqual(session["agent_id"], "conversation.llm")
        self.assertEqual(session["language"], "de")
        self.assertEqual(session["extra_system_prompt"], "Stimme: Jarvis.")

    def test_drop_same_turn_assistant_only_after_user(self) -> None:
        previous = types.SimpleNamespace(role="assistant", content="old")
        user = types.SimpleNamespace(role="user", content="hi")
        leaked = types.SimpleNamespace(role="assistant", content="unrefined")
        content = [previous, user, leaked]
        refine.drop_same_turn_assistant(content)
        self.assertEqual(content, [previous, user])
        refine.drop_same_turn_assistant(content)
        self.assertEqual(content, [previous, user])

    def test_speech_chunks_split_on_punctuation(self) -> None:
        chunks = refine.speech_chunks(
            "Natürlich, Sir. Im Wohnzimmer: Heizung ist 24,89. R2D2 ist pausiert."
        )
        self.assertEqual(
            chunks,
            [
                "Natürlich, Sir.",
                " Im Wohnzimmer: Heizung ist 24,89.",
                " R2D2 ist pausiert.",
            ],
        )
        self.assertEqual("".join(chunks), "Natürlich, Sir. Im Wohnzimmer: Heizung ist 24,89. R2D2 ist pausiert.")
        self.assertEqual(refine.speech_chunks("Licht ist an."), ["Licht ist an."])
        self.assertEqual(refine.speech_chunks(""), [])
        self.assertEqual(refine.speech_chunks("OK"), ["OK"])
        self.assertEqual(refine.speech_chunks("z.B. Licht ist an."), ["z.B. Licht ist an."])
        self.assertEqual(
            refine.speech_chunks("Set to 21.5 degrees. The light is on."),
            ["Set to 21.5 degrees.", " The light is on."],
        )
        done, rest = refine.pop_complete_sentences("Set to 21.5 degrees. The light")
        self.assertEqual(done, ["Set to 21.5 degrees."])
        self.assertEqual(rest, " The light")

    def test_emit_streams_sentences_as_deltas(self) -> None:
        class Log:
            def __init__(self) -> None:
                self.content = [
                    types.SimpleNamespace(role="user", content="hi"),
                    types.SimpleNamespace(role="assistant", content="unrefined"),
                ]
                self.deltas: list[dict[str, str]] = []
                self.without: list[str] = []

            def async_add_delta_content_stream(self, agent_id: str | None, stream):
                del agent_id

                async def gen():
                    parts: list[str] = []
                    async for delta in stream:
                        self.deltas.append(delta)
                        parts.append(delta.get("content") or "")
                    self.content.append("".join(parts))
                    yield None

                return gen()

            def async_add_assistant_content_without_tools(self, body: str) -> None:
                self.without.append(body)

        log = Log()
        spoken = "Natürlich, Sir. Das Licht im Wohnzimmer ist an."
        asyncio.run(refine.emit_assistant_speech(log, "conversation.klar_nlu", spoken))
        self.assertEqual(log.content[-1], spoken)
        self.assertEqual(log.without, [])
        self.assertEqual(log.deltas[0], {"role": "assistant"})
        self.assertEqual(log.deltas[1], {"content": "Natürlich, Sir."})
        self.assertEqual(log.deltas[2], {"content": " Das Licht im Wohnzimmer ist an."})

    def test_emit_falls_back_without_delta_stream(self) -> None:
        class Log:
            def __init__(self) -> None:
                self.content = [types.SimpleNamespace(role="user", content="hi")]
                self.without: list[str] = []

            def async_add_assistant_content_without_tools(self, body: str) -> None:
                self.without.append(body)

        log = Log()
        asyncio.run(refine.emit_assistant_speech(log, "conversation.klar_nlu", "Licht ist an."))
        self.assertEqual(log.without, ["Licht ist an."])

    def test_tts_hears_first_published_line_not_later_rewrite(self) -> None:
        nlu = "Wohnzimmer Licht ist an."
        refined = "Natürlich, Sir. Das Licht im Wohnzimmer ist an."
        published = [nlu]
        self.assertNotEqual(published[-1], refined)
        published = []
        spoken = nlu
        if not refine.skip_rewrite("execute"):
            spoken = refined
        published.append(spoken)
        self.assertEqual(published[-1], refined)
        llm = "Bereits im Jarvis-Ton."
        published = []
        spoken = llm
        if not refine.skip_rewrite("chat"):
            spoken = refined
        published.append(spoken)
        self.assertEqual(published[-1], llm)
        published = [llm]
        if not refine.skip_rewrite("llm"):
            published.append(refined)
        self.assertEqual(published, [llm])

    def test_fallback_converse_must_not_reuse_voice_session(self) -> None:
        src = (PKG / "conversation.py").read_text()
        start = src.index("async def _fallback")
        end = src.index("def _preferred_area")
        body = src[start:end]
        self.assertNotIn("async_converse", body)
        self.assertNotIn("nested_llm_session", body)
        self.assertIn("stream_engine_assist", body)
        self.assertNotIn("user_input.conversation_id", body)
        self.assertNotIn("user_input.device_id", body)
        self.assertNotIn("record", body)
        self.assertIn("isolated_conversation_id", src)
        spoken = src[src.index("async def _spoken") : src.index("async def _briefing")]
        self.assertIn("skip_rewrite", spoken)
        self.assertIn("emit_assistant_speech", spoken)
        calendar = src[src.index('kind="calendar"') : src.index("if self._quiet_ack()")]
        self.assertIn("_was_published(fallback)", calendar)
        self.assertIn('"llm"', calendar)
        self.assertNotIn("async_add_assistant_content_without_tools", spoken)


if __name__ == "__main__":
    unittest.main()
