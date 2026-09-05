#!/usr/bin/env python3
"""Engine speech render glue fails closed when the route is missing."""

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
_load("klar_refine", "refine.py")
_load("klar_stream", "stream.py")
_load("klar_nlu.engine_llm", "engine_llm.py")
_load("klar_nlu.speech_locale", "speech_locale.py")
speech = _load("klar_nlu.speech", "speech.py")
sys.modules["klar_nlu.speech"] = speech
_load("klar_nlu.speech_snapshot", "speech_snapshot.py")
render = _load("klar_nlu.speech_render", "speech_render.py")


class SpeechRenderGlueTests(unittest.TestCase):
    def test_missing_engine_returns_none(self) -> None:
        hass = types.SimpleNamespace(data={})
        item = {
            "name": "HassTurnOn",
            "slots": [
                {"name": "entity_id", "value": "light.schlafzimmer"},
                {"name": "name", "value": "Kugel"},
            ],
        }

        async def run() -> str | None:
            return await render.spoken_after_execute(hass, "de", "default", item)

        spoken = asyncio.run(run())
        self.assertIsNone(spoken)


if __name__ == "__main__":
    unittest.main()
