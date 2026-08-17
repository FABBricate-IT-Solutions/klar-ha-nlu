#!/usr/bin/env python3
"""Stdlib tests for opt-in NLU-RAG tools. No Assist/HA control path."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _load():
    path = ROOT / "custom_components" / "klar_nlu" / "rag_tools.py"
    spec = importlib.util.spec_from_file_location("klar_rag_tools", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


rag = _load()


class RagToolsTests(unittest.TestCase):
    def test_prompt_forbids_ha_tools(self) -> None:
        prompt = rag.rag_prompt("de", {"entities": [{"name": "Kugel"}]}, None)
        self.assertIn("klar.parse", prompt)
        self.assertIn("klar.act", prompt)
        self.assertIn("keine Home-Assistant-Werkzeuge", prompt)
        self.assertIn("Kugel", prompt)

    def test_retrieval_caps_at_eight(self) -> None:
        entities = [{"name": f"n{i}"} for i in range(12)]
        lines = rag.retrieval_lines({"entities": entities}, "en")
        self.assertIn("n0", lines)
        self.assertNotIn("n8", lines)

    def test_parse_tool_reply(self) -> None:
        self.assertEqual(
            rag.parse_tool_reply("KLAR_PARSE: mach die kugel an"),
            {"tool": "klar.parse", "text": "mach die kugel an"},
        )
        act = rag.parse_tool_reply("KLAR_ACT: HassTurnOn entity_id=light.kugel")
        self.assertEqual(act["tool"], "klar.act")
        self.assertEqual(act["intent"], "HassTurnOn")
        self.assertEqual(act["slots"], {"entity_id": "light.kugel"})
        self.assertIsNone(rag.parse_tool_reply("Just chatting."))

    def test_act_payload_has_no_plan(self) -> None:
        item = rag.act_payload("HassTurnOn", {"entity_id": "light.kugel"})
        self.assertEqual(item["name"], "HassTurnOn")
        self.assertNotIn("plan", item)
        self.assertNotIn("candidates", item)


if __name__ == "__main__":
    unittest.main()
