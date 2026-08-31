#!/usr/bin/env python3
"""Stdlib tests for LLM token streaming into HA deltas."""

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
_load("klar_nlu.const", "const.py")
_load("refine_voices", "refine_voices.py")
refine = _load("klar_refine", "refine.py")
stream = _load("klar_stream", "stream.py")


class _Delta:
    def __init__(self, text: str) -> None:
        self.content = text


class _Choice:
    def __init__(self, text: str) -> None:
        self.delta = _Delta(text)


class _Chunk:
    def __init__(self, text: str) -> None:
        self.choices = [_Choice(text)]


class _Stream:
    def __init__(self, parts: list[str]) -> None:
        self._parts = parts

    def __aiter__(self):
        async def gen():
            for part in self._parts:
                yield _Chunk(part)

        return gen()


class _Client:
    def __init__(self, parts: list[str]) -> None:
        self.parts = parts
        self.kwargs: dict = {}

        class Completions:
            def __init__(self, outer: _Client) -> None:
                self._outer = outer

            async def create(self, **kwargs):
                self._outer.kwargs = kwargs
                return _Stream(self._outer.parts)

        class Chat:
            def __init__(self, outer: _Client) -> None:
                self.completions = Completions(outer)

        self.chat = Chat(self)


class _Log:
    def __init__(self) -> None:
        self.content = [types.SimpleNamespace(role="user", content="hi")]
        self.deltas: list[dict[str, str]] = []

    def async_add_delta_content_stream(self, agent_id: str | None, deltas):
        del agent_id

        async def gen():
            async for delta in deltas:
                self.deltas.append(delta)
                yield None

        return gen()


class StreamTests(unittest.TestCase):
    def test_pop_keeps_unfinished_sentence(self) -> None:
        done, rest = refine.pop_complete_sentences("Natürlich, Sir. Im Wohnzimmer")
        self.assertEqual(done, ["Natürlich, Sir."])
        self.assertEqual(rest, " Im Wohnzimmer")
        self.assertEqual(refine.pop_complete_sentences("OK")[0], [])
        self.assertEqual(refine.pop_complete_sentences("Licht ist an.")[0], ["Licht ist an."])

    def test_stream_delta_reads_openai_chunk(self) -> None:
        self.assertEqual(refine.speech_from_stream_delta(_Chunk("Licht")), "Licht")
        self.assertEqual(
            refine.speech_from_stream_delta({"choices": [{"delta": {"content": "an"}}]}),
            "an",
        )
        self.assertEqual(refine.speech_from_stream_delta({"choices": []}), "")

    def test_token_deltas_hit_chat_log_immediately(self) -> None:
        log = _Log()
        client = _Client(["Natür", "lich, Sir. ", "Licht ist an."])
        speech, published = asyncio.run(
            stream.stream_chat(client, "Gemma-4-E4B-it-GGUF", "hi", "sys", log, "conversation.klar_nlu")
        )
        self.assertTrue(published)
        self.assertEqual(speech, "Natürlich, Sir. Licht ist an.")
        self.assertEqual(log.deltas[0], {"role": "assistant"})
        self.assertEqual(
            [item.get("content") for item in log.deltas if item.get("content")],
            ["Natür", "lich, Sir. ", "Licht ist an."],
        )
        self.assertTrue(client.kwargs.get("stream"))

    def test_emit_awaits_coroutine_streamer(self) -> None:
        log = types.SimpleNamespace(content=[], deltas=[])

        async def streamer(agent_id, deltas):
            del agent_id
            async for delta in deltas:
                log.deltas.append(delta)

        log.async_add_delta_content_stream = streamer
        speech, published = asyncio.run(
            stream.stream_chat(_Client(["Hi"]), "Gemma-4-E4B-it-GGUF", "hi", "sys", log, "conversation.klar_nlu")
        )
        self.assertTrue(published)
        self.assertEqual(speech, "Hi")
        self.assertEqual(log.deltas, [{"role": "assistant"}, {"content": "Hi"}])

    def test_hold_blocks_tool_line(self) -> None:
        log = _Log()

        def hold(speech: str) -> bool | None:
            if speech.startswith("KLAR_PARSE:"):
                return None
            return True

        speech, published = asyncio.run(
            stream.stream_chat(
                _Client(["KLAR_PARSE: Licht an"]),
                "Gemma-4-E4B-it-GGUF",
                "hi",
                "sys",
                log,
                "conversation.klar_nlu",
                hold=hold,
            )
        )
        self.assertEqual(speech, "KLAR_PARSE: Licht an")
        self.assertFalse(published)
        self.assertEqual(log.deltas, [])

    def test_conversation_enables_ha_streaming(self) -> None:
        src = (PKG / "conversation.py").read_text(encoding="utf-8")
        self.assertIn("_attr_supports_streaming = True", src)
        self.assertIn("stream_chat", src)
        self.assertIn("klar_published", src)
        self.assertIn("_was_published", src)
        self.assertNotIn("result.klar_published", src)
        spoken = src[src.index("async def _spoken") : src.index("async def _briefing")]
        self.assertIn("published", spoken)


if __name__ == "__main__":
    unittest.main()
