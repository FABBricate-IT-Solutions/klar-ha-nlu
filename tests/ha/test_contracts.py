#!/usr/bin/env python3
"""Stdlib tests for the V2 response boundary."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PATH = ROOT / "custom_components" / "klar_nlu" / "contracts.py"
SPEC = importlib.util.spec_from_file_location("klar_contracts_test", PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {PATH}")
contracts = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = contracts
SPEC.loader.exec_module(contracts)


def _evidence() -> dict[str, object]:
    return {"kind": "action", "source": "exact", "value": "lock", "score": 1.0, "exact": True}


def _plan() -> dict[str, object]:
    evidence = [_evidence()]
    return {
        "confidence": 1.0,
        "margin": 1.0,
        "evidence": evidence,
        "steps": [
            {
                "index": 0,
                "intent": {
                    "name": "HassTurnOn",
                    "slots": [{"name": "entity_id", "value": "lock.wohnungstuer"}],
                },
                "confidence": 1.0,
                "evidence": evidence,
            }
        ],
    }


def _payload(decision: dict[str, object], plan: dict[str, object] | None = None) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "2.0",
        "text": "lock the door",
        "conversation_id": "c1",
        "decision": decision,
        "speech": "Confirm?",
        "confidence": 1.0,
        "margin": 1.0,
        "candidates": [],
        "evidence": [],
        "trace": {"stages": [], "discarded": []},
        "briefing": False,
    }
    if plan is not None:
        value["plan"] = plan
        value["selected_candidate_id"] = "selected-000"
        value["candidates"] = [
            {
                "id": "selected-000",
                "plan": plan,
                "score": 1.0,
                "margin": 1.0,
                "policy": "test",
                "precedence": 0,
                "evidence": [],
            }
        ]
    return value


class ContractTests(unittest.TestCase):
    def test_confirm_never_extracts_intents(self) -> None:
        payload = contracts.validate_v2_payload(
            _payload({"type": "confirm", "prompt": "Confirm?", "candidate_id": "selected-000"})
        )
        self.assertEqual(contracts.executable_intents(payload), [])
        self._assert_no_executable_shape(payload)

    def test_affirmed_execute_extracts_intent(self) -> None:
        payload = contracts.validate_v2_payload(_payload({"type": "execute"}, _plan()))
        self.assertEqual(contracts.executable_intents(payload)[0]["name"], "HassTurnOn")

    def test_rejects_plan_on_confirm(self) -> None:
        with self.assertRaises(ValueError):
            contracts.validate_v2_payload(
                _payload({"type": "confirm", "prompt": "Confirm?", "candidate_id": "selected-000"}, _plan())
            )

    def test_rejects_candidate_plan_on_confirm(self) -> None:
        payload = _payload({"type": "confirm", "prompt": "Confirm?", "candidate_id": "selected-000"})
        payload["candidates"] = [
            {
                "id": "selected-000",
                "plan": _plan(),
                "score": 1.0,
                "margin": 1.0,
                "policy": "test",
                "precedence": 0,
                "evidence": [],
            }
        ]
        with self.assertRaises(ValueError):
            contracts.validate_v2_payload(payload)

    def test_execute_result_accepts_partial_and_rejects_false_success(self) -> None:
        partial = {
            "outcome": "partial",
            "speech": "Living is on. One step failed.",
            "steps": [
                {"index": 0, "intent": "HassTurnOn", "status": "success", "speech": "Living is on.", "error": None},
                {"index": 1, "intent": "HassTurnOn", "status": "error", "speech": None, "error": "failed"},
            ],
        }
        self.assertEqual(contracts.validate_execute_result(partial)["outcome"], "partial")
        fake_success = {
            "outcome": "success",
            "speech": "Done.",
            "steps": [{"index": 0, "intent": "HassTurnOn", "status": "error", "speech": None, "error": "failed"}],
        }
        with self.assertRaises(ValueError):
            contracts.validate_execute_result(fake_success)

    def test_accepts_privacy_trace_tokens(self) -> None:
        payload = _payload({"type": "chat"})
        payload["trace"] = {
            "stages": [],
            "discarded": [],
            "tokens": ["ist", "die", "kugel", "an"],
            "normalized": "ist die kugel an",
        }
        validated = contracts.validate_v2_payload(payload)
        self.assertEqual(validated["trace"]["tokens"], ["ist", "die", "kugel", "an"])
        self.assertEqual(validated["trace"]["normalized"], "ist die kugel an")

    def test_rejects_retrieval_on_confirm(self) -> None:
        payload = _payload({"type": "confirm", "prompt": "Confirm?", "candidate_id": "selected-000"})
        payload["retrieval"] = {"entities": [{"entity_id": "light.x", "name": "X", "domain": "light"}]}
        with self.assertRaises(ValueError):
            contracts.validate_v2_payload(payload)

    def test_accepts_retrieval_on_chat(self) -> None:
        payload = _payload({"type": "chat"})
        payload["retrieval"] = {
            "entities": [{"entity_id": "light.x", "name": "X", "domain": "light"}],
            "areas": ["wohnzimmer"],
        }
        validated = contracts.validate_v2_payload(payload)
        self.assertEqual(validated["retrieval"]["areas"], ["wohnzimmer"])

    def test_policy_trace_accepts_action_payload(self) -> None:
        payload = _payload({"type": "chat"})
        payload["policy_trace"] = {
            "matched_rule": "night",
            "hit": "reply",
            "compiled_risky": False,
            "payload": "Schlaf schön.",
        }
        validated = contracts.validate_v2_payload(payload)
        self.assertEqual(validated["policy_trace"]["payload"], "Schlaf schön.")

    def test_rejects_oversized_candidates(self) -> None:
        payload = _payload({"type": "chat"})
        payload["candidates"] = [{}] * 65
        with self.assertRaises(ValueError):
            contracts.validate_v2_payload(payload)

    def _assert_no_executable_shape(self, value: object) -> None:
        if isinstance(value, dict):
            for forbidden in ("plan", "intent", "slots", "selected_candidate_id"):
                self.assertNotIn(forbidden, value)
            for child in value.values():
                self._assert_no_executable_shape(child)
        elif isinstance(value, list):
            for child in value:
                self._assert_no_executable_shape(child)


if __name__ == "__main__":
    unittest.main()
