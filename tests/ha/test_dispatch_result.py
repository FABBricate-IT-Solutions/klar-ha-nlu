#!/usr/bin/env python3
"""Intent-step success must not depend on a spoken sentence (H-17)."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _load():
    path = ROOT / "custom_components" / "klar_nlu" / "dispatch_result.py"
    name = "klar_dispatch_result_test"
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[name] = mod
    spec.loader.exec_module(mod)
    return mod


result = _load()


class DispatchResultTests(unittest.TestCase):
    def test_empty_speech_is_still_success(self) -> None:
        for speech in ("", None):
            step = result.ok(speech)
            self.assertTrue(step.ok)
            self.assertIsNone(step.error)
            self.assertIsNone(step.speech)

    def test_spoken_success_keeps_the_text(self) -> None:
        step = result.ok("Done.")
        self.assertTrue(step.ok)
        self.assertEqual(step.speech, "Done.")


if __name__ == "__main__":
    unittest.main()
