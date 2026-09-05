#!/usr/bin/env python3
"""OpenAI-compatible endpoint extraction from a HA conversation agent."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace

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
endpoint = _load("klar_nlu.llm_endpoint", "llm_endpoint.py")


class LlmEndpointTests(unittest.TestCase):
    def test_setup_does_not_copy_ha_agent_onto_klar(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "__init__.py").read_text(encoding="utf-8")
        self.assertNotIn("async_push_llm_endpoint", src)

    def test_reads_openai_client_and_entry_data(self) -> None:
        client = SimpleNamespace(
            chat=object(),
            base_url="https://api.openai.com/v1/",
            api_key="sk-secret",
        )
        agent = SimpleNamespace(
            model="gpt-4o-mini",
            entry=SimpleNamespace(runtime_data=client, options={}, data={}),
            subentry=None,
            options=None,
        )
        fake = types.ModuleType("conversation")
        fake.async_get_agent = lambda _hass, _agent_id: agent
        original = endpoint.ha_conversation
        endpoint.ha_conversation = fake
        try:
            out = endpoint.openai_compatible_endpoint(object(), "conversation.llm")
        finally:
            endpoint.ha_conversation = original
        self.assertEqual(out["model"], "gpt-4o-mini")
        self.assertEqual(out["base_url"], "https://api.openai.com/v1")
        self.assertEqual(out["api_key"], "sk-secret")

    def test_ollama_base_and_empty_key(self) -> None:
        agent = SimpleNamespace(
            model="llama3",
            entry=SimpleNamespace(
                runtime_data=None,
                options={},
                data={"base_url": "http://192.168.1.8:11434", "api_key": ""},
            ),
            subentry=None,
            options=None,
            client=None,
            _client=None,
            openai=None,
            coordinator=None,
        )
        fake = types.ModuleType("conversation")
        fake.async_get_agent = lambda _hass, _agent_id: agent
        original = endpoint.ha_conversation
        endpoint.ha_conversation = fake
        try:
            out = endpoint.openai_compatible_endpoint(object(), "conversation.ollama")
        finally:
            endpoint.ha_conversation = original
        self.assertEqual(out["base_url"], "http://192.168.1.8:11434/v1")
        self.assertEqual(out["model"], "llama3")
        self.assertEqual(out["api_key"], "")

    def test_rejects_userinfo_urls(self) -> None:
        self.assertEqual(endpoint._normalize_base("https://u:p@host/v1"), "")
        self.assertEqual(endpoint._normalize_base("file:///tmp"), "")


if __name__ == "__main__":
    unittest.main()
