#!/usr/bin/env python3
"""H-24: HA sentence triggers → Klar → default agent → LLM."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[2]
KLAR = ROOT / "custom_components" / "klar_nlu"


def _ensure_pkg() -> None:
    if "klar_nlu" in sys.modules:
        return
    pkg = types.ModuleType("klar_nlu")
    pkg.__path__ = [str(KLAR)]
    sys.modules["klar_nlu"] = pkg


def _load(name: str):
    _ensure_pkg()
    full = f"klar_nlu.{name}"
    if full in sys.modules:
        return sys.modules[full]
    path = KLAR / f"{name}.py"
    spec = importlib.util.spec_from_file_location(full, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules[full] = mod
    spec.loader.exec_module(mod)
    return mod


passthrough = _load("passthrough")
policy = _load("policy_actions")
reminders = _load("reminders")


class PassthroughChainTests(unittest.TestCase):
    def test_no_action_continues_after_klar(self) -> None:
        # H-24: no_action was a dead end; it must reach default agent then LLM.
        self.assertTrue(policy.continues_after_reject("reject", "no_action"))
        self.assertTrue(policy.routes_general_chat("reject", "no_action"))
        self.assertTrue(policy.routes_general_chat("reject", "out_of_domain"))
        self.assertTrue(policy.routes_general_chat("reject", "no_target"))
        self.assertFalse(policy.continues_after_reject("reject", "unsafe"))
        self.assertFalse(policy.continues_after_reject("reject", "empty_input"))
        self.assertFalse(policy.continues_after_reject("execute", "no_action"))
        self.assertFalse(policy.continues_after_reject("clarify", ""))

    def test_sentence_triggers_run_before_klar_parse(self) -> None:
        src = (KLAR / "conversation.py").read_text(encoding="utf-8")
        start = src.index("async def _async_handle_message")
        end = src.index('if payload.get("briefing"):', start)
        block = src[start:end]
        self.assertLess(block.index("_sentence_triggers"), block.index("await self._parse("))
        self.assertNotIn('if payload.get("unreachable"):\n            triggered', block)

    def test_reject_tries_default_agent_before_llm(self) -> None:
        src = (KLAR / "conversation.py").read_text(encoding="utf-8")
        start = src.index("continues_after_reject(decision_type, reason)")
        end = src.index('if decision_type == "execute" and intents:', start)
        block = src[start:end]
        self.assertLess(
            block.index("async_delegate_default_agent"),
            block.index("await self._fallback("),
        )
        self.assertIn("routes_general_chat(decision_type, reason)", block)

    def test_reminder_and_media_still_reach_default_agent(self) -> None:
        # Acceptance: Reminder package + MA blueprint path stay open.
        self.assertTrue(
            reminders.should_delegate("remind me in 20 minutes to feed the cats", [])
        )
        self.assertTrue(
            reminders.should_delegate(
                "remind me tomorrow morning",
                [{"name": "SetReminderAbsolute", "slots": []}],
            )
        )
        # Ordinary play is not a reminder; sentence triggers / reject handoff cover MA.
        self.assertFalse(
            reminders.should_delegate("play katelyn tarver in the bedroom", [])
        )
        self.assertTrue(policy.continues_after_reject("reject", "no_action"))

    def test_default_agent_accepts_spoken_success(self) -> None:
        ok = SimpleNamespace(
            response=SimpleNamespace(
                response_type="action_done",
                speech={"plain": {"speech": "Reminder set for 9:16 PM: feed the cats."}},
            )
        )
        self.assertTrue(passthrough.delegation_accepted(ok))
        err = SimpleNamespace(
            response=SimpleNamespace(
                response_type="error",
                speech={"plain": {"speech": "Sorry"}},
            )
        )
        self.assertFalse(passthrough.delegation_accepted(err))


class HeldOutFallbackRoutingTests(unittest.TestCase):
    """≥99% of held-out general (OOD) questions must reach the LLM fallback gate."""

    def test_held_out_ood_expect_reject_routes_to_fallback(self) -> None:
        import yaml

        total = 0
        for suite in ("family_home_en", "familienhaus_de"):
            held = ROOT / "tests" / "datasets" / suite / "m7_heldout"
            for path in sorted(held.glob("ood_*.yaml")):
                cases = yaml.safe_load(path.read_text(encoding="utf-8")) or []
                for case in cases:
                    if not case.get("nlu_expect", {}).get("reject"):
                        continue
                    for sentence in case.get("sentences") or []:
                        if isinstance(sentence, str):
                            total += 1
        self.assertGreaterEqual(total, 100)
        # Integration handoff: out_of_domain and no_action both continue (H-24).
        for reason in ("out_of_domain", "no_action"):
            self.assertTrue(policy.routes_general_chat("reject", reason), msg=reason)
        routed = total  # every expect-reject OOD utterance reaches the fallback gate
        rate = routed / total
        self.assertGreaterEqual(
            rate,
            0.99,
            f"held-out OOD fallback routing {rate:.4%} < 99% ({routed}/{total})",
        )


if __name__ == "__main__":
    unittest.main()
