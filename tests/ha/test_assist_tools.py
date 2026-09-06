#!/usr/bin/env python3
"""HA 2026.9 Assist tools on Klar's conversation entity."""

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
assist_tools = _load("klar_nlu.assist_tools", "assist_tools.py")


class _Tool:
    def __init__(self, name: str) -> None:
        self.name = name
        self.description = "test"
        self.parameters = {"type": "object", "properties": {}}


class _Api:
    def __init__(self, tools: list[_Tool]) -> None:
        self.tools = tools


class _ChatLog:
    def __init__(self, tools: list[_Tool] | None = None) -> None:
        self.llm_api = _Api(tools or [])
        self.provided = False
        self.content = []
        self.unresponded_tool_results = False

    async def async_provide_llm_data(self, context, apis, user, extra):
        del context, user, extra
        self.provided = True
        self.apis = apis


class AssistToolsTests(unittest.TestCase):
    def test_openai_tools_keep_prefixed_names(self) -> None:
        log = _ChatLog([_Tool("intent__HassTurnOn"), _Tool("homeassistant__GetLiveContext")])
        tools = assist_tools.openai_tools_from_chat_log(log)
        names = [row["function"]["name"] for row in tools]
        self.assertEqual(names, ["intent__HassTurnOn", "homeassistant__GetLiveContext"])
        src = (PKG / "assist_tools.py").read_text(encoding="utf-8")
        self.assertNotIn("HassTurnOn", src.replace("intent__HassTurnOn", ""))
        self.assertNotIn("GetLiveContext", src.replace("homeassistant__GetLiveContext", ""))
        self.assertIn("async_provide_llm_data", src)
        self.assertIn('["assist"]', src)

    def test_provide_llm_data_only_when_api_exists(self) -> None:
        log = _ChatLog([_Tool("intent__HassTurnOn")])

        class _Input:
            def as_llm_context(self, domain: str) -> str:
                return f"ctx:{domain}"

        ok = asyncio.run(assist_tools.provide_llm_data(log, _Input(), None))
        self.assertTrue(ok)
        self.assertTrue(log.provided)
        self.assertEqual(log.apis, ["assist"])

        class _Bare:
            pass

        bare = types.SimpleNamespace()
        self.assertFalse(asyncio.run(assist_tools.provide_llm_data(bare, _Bare(), None)))

    def test_fallback_source_has_no_async_converse(self) -> None:
        src = (PKG / "conversation.py").read_text(encoding="utf-8")
        start = src.index("async def _fallback")
        end = src.index("def _preferred_area")
        body = src[start:end]
        self.assertIn("stream_assist_with_ha_tools", body)
        self.assertNotIn("async_converse", body)
        self.assertNotIn("can_use_fallback_agent", src)
        self.assertNotIn("agent_has_home_control", src)

    def test_copy_drops_agent_gate(self) -> None:
        strings = (PKG / "strings.json").read_text(encoding="utf-8")
        de = (PKG / "translations" / "de.json").read_text(encoding="utf-8")
        self.assertNotIn("chit-chat agent", strings)
        self.assertNotIn("Smalltalk-Agenten", de)
        self.assertIn("already prefixed", strings)


if __name__ == "__main__":
    unittest.main()
