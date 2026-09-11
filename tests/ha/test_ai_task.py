#!/usr/bin/env python3
"""AI Task message build and JSON extract without Home Assistant."""

from __future__ import annotations

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
_load("klar_refine", "refine.py")
_load("klar_stream", "stream.py")
_load("klar_nlu.engine_llm", "engine_llm.py")
ai_task = _load("klar_nlu.ai_task", "ai_task.py")


class AITaskHelpers(unittest.TestCase):
    def test_json_object_reads_fenced_and_prose(self) -> None:
        self.assertEqual(ai_task.json_object('```json\n{"count":2}\n```'), {"count": 2})
        self.assertEqual(ai_task.json_object('sure {"ok":true} thanks'), {"ok": True})
        self.assertIsNone(ai_task.json_object("no object here"))
        self.assertIsNone(ai_task.json_object("[1,2]"))

    def test_structure_system_asks_for_json(self) -> None:
        line = ai_task.structure_system({"title": {"selector": {"text": None}}})
        self.assertIn("JSON object", line)
        self.assertIn("title", line)
        self.assertNotIn("response_format", line)

    def test_messages_from_log_prepends_schema_and_keeps_roles(self) -> None:
        content = [
            {"role": "system", "content": "You are helpful."},
            {"role": "user", "content": "Write a title"},
        ]
        messages = ai_task.messages_from_log(content, "Write a title", {"title": "text"})
        self.assertEqual(messages[0]["role"], "system")
        self.assertIn("JSON object", messages[0]["content"])
        self.assertEqual(messages[1]["role"], "system")
        self.assertEqual(messages[2], {"role": "user", "content": "Write a title"})

    def test_messages_from_log_uses_instructions_when_empty(self) -> None:
        messages = ai_task.messages_from_log(None, "Count the lamps")
        self.assertEqual(messages, [{"role": "user", "content": "Count the lamps"}])

    def test_platform_is_registered_without_ai_task_manifest_dep(self) -> None:
        init = (PKG / "__init__.py").read_text(encoding="utf-8")
        manifest = (PKG / "manifest.json").read_text(encoding="utf-8")
        src = (PKG / "ai_task.py").read_text(encoding="utf-8")
        self.assertIn("Platform.AI_TASK", init)
        self.assertIn("GENERATE_DATA", src)
        self.assertIn("complete_engine_chat", src)
        self.assertNotIn("GENERATE_IMAGE", src)
        self.assertNotIn("SUPPORT_ATTACHMENTS", src)
        self.assertIn('"conversation"', manifest)
        self.assertNotIn('"ai_task"', manifest)


if __name__ == "__main__":
    unittest.main()
