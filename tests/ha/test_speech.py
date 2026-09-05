#!/usr/bin/env python3
"""Stdlib tests for spoken personality wrap after an intent."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PKG = ROOT / "custom_components" / "klar_nlu"
if str(PKG) not in sys.path:
    sys.path.insert(0, str(PKG))


def _load(name: str, rel: str):
    path = ROOT / "custom_components" / "klar_nlu" / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


speech = _load("speech", "speech.py")
sys.modules["klar_speech"] = speech


class SpeechTests(unittest.TestCase):
    def test_clock_speech_is_one_sentence_without_seconds(self) -> None:
        now = datetime(2026, 8, 29, 16, 2, 55)
        self.assertEqual(speech.finish_clock_speech("Es ist 14:44:55.", "de", now), "Es ist 16:02.")
        self.assertEqual(
            speech.finish_clock_speech("Es ist 14:44:55. Die genaue Uhrzeit.", "de", now),
            "Es ist 16:02.",
        )
        self.assertEqual(speech.finish_clock_speech("It is 14:44:55.", "en", now), "It is 16:02.")
        self.assertEqual(speech.strip_clock_seconds("Es ist 14:44:55."), "Es ist 14:44.")
        self.assertEqual(speech.finish_clock_speech("Licht ist an.", "de", now), "Licht ist an.")

    def test_default_personality_does_not_wrap(self) -> None:
        self.assertEqual(speech.style("Licht ist an.", "default", "de"), "Licht ist an.")
        self.assertEqual(speech.style("Licht ist an.", "", "de"), "Licht ist an.")

    def test_butler_wraps_factual_line(self) -> None:
        spoken = speech.style("Licht ist an.", "butler", "de")
        variants = set((speech._locale("de").get("personality") or {}).get("butler") or [])
        self.assertTrue(variants)
        self.assertIn(spoken, {f"{prefix}Licht ist an." for prefix in variants})

    def test_from_handled_templates_are_gone(self) -> None:
        self.assertFalse(hasattr(speech, "from_handled"))
        self.assertFalse(hasattr(speech, "queue_speech"))
        self.assertFalse(hasattr(speech, "media_state_speech"))


if __name__ == "__main__":
    unittest.main()
