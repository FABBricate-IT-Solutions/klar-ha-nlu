#!/usr/bin/env python3
"""Stdlib tests for LLM reply refinement helpers."""

from __future__ import annotations

import asyncio
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def _load():
    path = ROOT / "custom_components" / "klar_nlu" / "refine.py"
    spec = importlib.util.spec_from_file_location("klar_refine", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


refine = _load()


class RefineTests(unittest.TestCase):
    def test_prompt_keeps_hard_safety_rules(self) -> None:
        prompt = refine.refine_prompt("de", "butler", "Maximal ein Satz.")
        self.assertIn("keine Home-Assistant-Werkzeuge", prompt)
        self.assertIn("Ändere keine Geräte", prompt)
        self.assertIn("keine neuen Fakten", prompt)
        self.assertIn("butlerhaft", prompt)
        self.assertIn("Maximal ein Satz.", prompt)

    def test_english_prompt_uses_personality(self) -> None:
        prompt = refine.refine_prompt("en", "locker", None)
        self.assertIn("Do not call Home Assistant tools", prompt)
        self.assertIn("casual", prompt)
        self.assertNotIn("Additional style instruction", prompt)

    def test_should_refine_only_for_non_chat_replies_with_agent(self) -> None:
        self.assertTrue(
            refine.should_refine(True, "conversation.llm", "Licht ist an.", False, False)
        )
        self.assertFalse(
            refine.should_refine(False, "conversation.llm", "Licht ist an.", False, False)
        )
        self.assertFalse(refine.should_refine(True, None, "Licht ist an.", False, False))
        self.assertFalse(refine.should_refine(True, "conversation.llm", "", False, False))
        self.assertFalse(refine.should_refine(True, "conversation.llm", "Hallo", True, False))
        self.assertFalse(refine.should_refine(True, "conversation.llm", "News", False, True))

    def test_empty_result_speech_is_ignored(self) -> None:
        result = _Result("")
        self.assertEqual(refine.speech_from_result(result), "")

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


class _Response:
    def __init__(self, text: str) -> None:
        self.speech = {"plain": {"speech": text}}


class _Result:
    def __init__(self, text: str) -> None:
        self.response = _Response(text)


if __name__ == "__main__":
    unittest.main()
