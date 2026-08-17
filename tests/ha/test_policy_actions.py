#!/usr/bin/env python3
"""Stdlib tests for user policy action helpers."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _load():
    path = ROOT / "custom_components" / "klar_nlu" / "policy_actions.py"
    spec = importlib.util.spec_from_file_location("klar_policy_actions", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


actions = _load()


class PolicyActionTests(unittest.TestCase):
    def test_reads_hit_and_payload(self) -> None:
        hit, payload = actions.hit_and_payload(
            {"policy_trace": {"hit": "template", "payload": "{{ states('sensor.x') }}"}}
        )
        self.assertEqual(hit, "template")
        self.assertIn("sensor.x", payload)

    def test_reply_skips_fallback(self) -> None:
        self.assertTrue(actions.skips_llm_fallback("reply"))
        self.assertTrue(actions.skips_llm_fallback("template"))
        self.assertFalse(actions.skips_llm_fallback("llm"))
        self.assertFalse(actions.skips_llm_fallback(""))


if __name__ == "__main__":
    unittest.main()
