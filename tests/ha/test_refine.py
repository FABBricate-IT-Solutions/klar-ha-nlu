#!/usr/bin/env python3
"""Stdlib tests for LLM reply refinement helpers."""

from __future__ import annotations

import asyncio
import importlib.util
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PKG = ROOT / "custom_components" / "klar_nlu"
if str(PKG) not in sys.path:
    sys.path.insert(0, str(PKG))


def _load(name: str, rel: str):
    path = PKG / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


const = _load("klar_const", "const.py")
refine = _load("klar_refine", "refine.py")
speech = _load("klar_speech", "speech.py")


class RefineTests(unittest.TestCase):
    def test_prompt_keeps_hard_safety_rules(self) -> None:
        prompt = refine.refine_prompt("de", "butler", "Ein oder zwei Sätze.")
        self.assertIn("Keine Home-Assistant-Werkzeuge", prompt)
        self.assertIn("Ziffern bleiben Ziffern", prompt)
        self.assertIn("2 bleibt 2", prompt)
        self.assertIn("2 Lichter sind an, 3 Lichter sind aus.", prompt)
        self.assertIn("21,5 °C", prompt)
        self.assertIn("Keine neuen Zahlen", prompt)
        self.assertIn("Butler", prompt)
        self.assertIn("ein oder zwei Sätze", prompt)
        self.assertIn("Keine feste Eröffnungsformel", prompt)
        self.assertIn("Klebe nicht jedes Mal dieselbe Eröffnung davor.", prompt)
        self.assertNotIn("Formel: Sehr wohl.", prompt)
        self.assertNotIn("Hänge immer an", prompt)
        self.assertIn("Ein oder zwei Sätze.", prompt)

    def test_each_personality_has_its_own_voice(self) -> None:
        flavor = {
            "default": "schlicht",
            "butler": "Butler",
            "locker": "locker",
            "fuersorglich": "fürsorglich",
            "party": "euphorisch",
            "grantig": "grantig",
            "sarkastisch": "sarkastisch",
            "pirat": "piratenhaft",
            "hippie": "entspannt",
            "gollum": "gollumartig",
        }
        seen: set[str] = set()
        for name, marker in flavor.items():
            prompt = refine.refine_prompt("de", name, None)
            self.assertIn(marker, prompt, name)
            self.assertNotIn("Hänge immer an", prompt, name)
            self.assertRegex(prompt, r"→ .+\. .+", name)
            self.assertNotIn(prompt, seen)
            seen.add(prompt)

    def test_english_prompt_uses_personality(self) -> None:
        prompt = refine.refine_prompt("en", "locker", None)
        self.assertIn("Do not call Home Assistant tools", prompt)
        self.assertIn("casual", prompt)
        self.assertIn("Voice:", prompt)
        self.assertIn("Do not stamp the same opening every time.", prompt)
        self.assertIn("all set", prompt)
        self.assertNotIn("Additional style instruction", prompt)

    def test_input_wraps_speech_so_queries_are_not_answered(self) -> None:
        wrapped = refine.refine_input("Temperatur im Schlafzimmer.", "de")
        self.assertEqual(wrapped, "Temperatur im Schlafzimmer.")

    def test_accept_refined_keeps_facts_and_rejects_inventions(self) -> None:
        self.assertEqual(
            refine.accept_refined("Wohnzimmer Licht ist an.", "Das Licht im Wohnzimmer ist an."),
            "Das Licht im Wohnzimmer ist an.",
        )
        self.assertEqual(
            refine.accept_refined("Heizung Wohnzimmer auf 21 Grad.", "Die Heizung im Wohnzimmer auf 21 Grad."),
            "Die Heizung im Wohnzimmer auf 21 Grad.",
        )
        self.assertEqual(
            refine.accept_refined(
                "Better Thermostat Wohnzimmer ist 21,5 °C.",
                "Im Wohnzimmer sind es 21,5 °C.",
            ),
            "Im Wohnzimmer sind es 21,5 °C.",
        )
        self.assertIsNone(refine.accept_refined("Temperatur im Schlafzimmer.", "Die Temperatur im Schlafzimmer ist 20 Grad."))
        self.assertIsNone(refine.accept_refined("Temperatur im Schlafzimmer.", "Die Temperatur im Schlafzimmer ist zwanzig Grad."))
        self.assertIsNone(refine.accept_refined("Klimaanlage auf 19 Grad.", "Die Klimaanlage ist auf neunzehn Grad."))
        self.assertIsNone(refine.accept_refined("Erledigt: HassSetPosition.", "HassSetPosition ist erledigt."))
        self.assertIsNone(refine.accept_refined("Licht ist an.", "Licht ist an..."))
        self.assertIsNone(refine.accept_refined("Temperatur im Schlafzimmer.", "Wie ist die Temperatur im Schlafzimmer?"))
        self.assertEqual(
            refine.accept_refined(
                "Wohnzimmer Licht ist an.",
                "Das Licht im Wohnzimmer ist an. Ich habe es für Sie eingeschaltet.",
            ),
            "Das Licht im Wohnzimmer ist an. Ich habe es für Sie eingeschaltet.",
        )
        self.assertEqual(
            refine.clean_refined("Das Licht ist an.\nIch habe es eingeschaltet."),
            "Das Licht ist an. Ich habe es eingeschaltet.",
        )
        self.assertIsNone(refine.accept_refined("Licht ist an.", "Licht ist an. " + ("x" * 400)))

    def test_should_refine_home_status_and_control(self) -> None:
        self.assertTrue(
            refine.should_refine(True, "conversation.llm", "Licht ist an.", True)
        )
        self.assertTrue(
            refine.should_refine(True, "conversation.llm", "Im Wohnzimmer sind es 21,5 °C.", True)
        )
        self.assertFalse(
            refine.should_refine(False, "conversation.llm", "Licht ist an.", True)
        )
        self.assertFalse(refine.should_refine(True, None, "Licht ist an.", True))
        self.assertFalse(refine.should_refine(True, "conversation.llm", "", True))
        self.assertFalse(refine.should_refine(True, "conversation.llm", "Hallo", False))
        self.assertFalse(refine.should_refine(True, "conversation.llm", "News", False))
        self.assertFalse(hasattr(refine, "_TIMEOUT"))

    def test_extra_body_disables_thinking_for_ha_openai_client(self) -> None:
        self.assertEqual(
            refine.refine_extra_body(),
            {"chat_template_kwargs": {"enable_thinking": False}},
        )

    def test_completion_speech_reads_openai_choice(self) -> None:
        self.assertEqual(refine.speech_from_completion(_Completion("Licht ist an.")), "Licht ist an.")
        self.assertEqual(refine.speech_from_completion(_Completion("")), "")

    def test_empty_result_speech_is_ignored(self) -> None:
        result = _Result("")
        self.assertEqual(refine.speech_from_result(result), "")

    def test_options_personality_switches_refine_prompt_and_fallback_cue(self) -> None:
        self.assertEqual(const.resolve_personality("grantig"), "grantig")
        self.assertEqual(const.resolve_personality("nope"), "default")
        self.assertEqual(set(refine._PERSONALITY), set(const.PERSONALITIES))
        for pack in ("de", "en"):
            seen_prompts: set[str] = set()
            seen_cues: set[str] = set()
            for name in const.PERSONALITIES:
                prompt = refine.refine_prompt(pack, name, None)
                spoken = speech.style("Licht ist an.", name, pack)
                self.assertNotIn(prompt, seen_prompts)
                seen_prompts.add(prompt)
                if name == "default":
                    self.assertEqual(spoken, "Licht ist an.")
                    continue
                cue = spoken[: -len("Licht ist an.")].strip()
                self.assertTrue(cue, name)
                self.assertNotIn(cue, seen_cues)
                seen_cues.add(cue)
        butler = refine.refine_prompt("de", const.resolve_personality("butler"), None)
        grantig = refine.refine_prompt("de", const.resolve_personality("grantig"), None)
        self.assertIn("Butler", butler)
        self.assertNotIn("grantig", butler)
        self.assertIn("grantig", grantig)
        self.assertNotIn("Butler", grantig)
        self.assertEqual(
            speech.style("Licht ist an.", "butler", "de"),
            "Sehr wohl. Licht ist an.",
        )

    def test_successful_refine_keeps_natural_line_without_restamping_cue(self) -> None:
        source = "Wohnzimmer Licht ist an."
        natural = "Das Licht im Wohnzimmer ist an. Ich habe es für Sie eingeschaltet."
        self.assertEqual(refine.accept_refined(source, natural), natural)
        self.assertEqual(
            speech.style(source, "butler", "de"),
            "Sehr wohl. Wohnzimmer Licht ist an.",
        )

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


class _CompletionMessage:
    def __init__(self, text: str) -> None:
        self.content = text


class _CompletionChoice:
    def __init__(self, text: str) -> None:
        self.message = _CompletionMessage(text)


class _Completion:
    def __init__(self, text: str) -> None:
        self.choices = [_CompletionChoice(text)]


class _Response:
    def __init__(self, text: str) -> None:
        self.speech = {"plain": {"speech": text}}


class _Result:
    def __init__(self, text: str) -> None:
        self.response = _Response(text)


if __name__ == "__main__":
    unittest.main()
