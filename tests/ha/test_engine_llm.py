#!/usr/bin/env python3
"""Klar engine LLM SSE glue."""

from __future__ import annotations

import asyncio
import importlib.util
import json
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
_load("klar_refine", "refine.py")
_load("klar_stream", "stream.py")
engine_llm = _load("klar_nlu.engine_llm", "engine_llm.py")


class _Content:
    def __init__(self, raw: bytes) -> None:
        self.raw = raw

    async def iter_any(self):
        yield self.raw


class EngineLlmTests(unittest.TestCase):
    def test_sse_json_reads_klar_deltas(self) -> None:
        payload = (
            b'data: {"type":"delta","text":"Hel"}\n\n'
            b'data: {"type":"delta","text":"lo"}\n\n'
            b'data: {"type":"done","text":"Hello"}\n\n'
        )

        async def collect() -> list[dict]:
            return [row async for row in engine_llm._iter_sse_json(_Content(payload))]

        rows = asyncio.run(collect())
        self.assertEqual([row.get("text") for row in rows], ["Hel", "lo", "Hello"])
        self.assertEqual(engine_llm._event_text({"type": "done", "text": "Hello"}), "Hello")
        self.assertEqual(engine_llm._event_text("nope"), "")

    def test_engine_target_prefers_explicit_url(self) -> None:
        hass = types.SimpleNamespace(data={})
        self.assertEqual(engine_llm.engine_target(hass, "http://127.0.0.1:10520/", "tok"), ("http://127.0.0.1:10520", "tok"))
        hass.data = {"klar_nlu": {"e1": {"url": "http://klar-nlu:10520", "token": "secret"}}}
        self.assertEqual(engine_llm.engine_target(hass), ("http://klar-nlu:10520", "secret"))


if __name__ == "__main__":
    unittest.main()
