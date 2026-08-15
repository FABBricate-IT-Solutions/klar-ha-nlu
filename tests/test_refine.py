#!/usr/bin/env python3
"""Stdlib tests for LLM reply refinement helpers."""

from __future__ import annotations

import asyncio
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def _load(name: str, rel: str):
    path = ROOT / "custom_components" / "klar_nlu" / rel
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
        prompt = refine.refine_prompt("de", "butler", "Maximal ein Satz.")
        self.assertIn("Keine Home-Assistant-Werkzeuge", prompt)
        self.assertIn("Ziffern bleiben Ziffern", prompt)
        self.assertIn("2 bleibt 2", prompt)
        self.assertIn("2 Lichter sind an, 3 Lichter sind aus.", prompt)
        self.assertIn("21,5 °C", prompt)
        self.assertIn("Keine neuen Zahlen", prompt)
        self.assertIn("butlerhaft", prompt)
        self.assertIn("Wohnzimmer Licht ist an. → Sehr wohl.", prompt)
        self.assertIn("Stimme (zwingend)", prompt)
        self.assertIn("Der Satz selbst muss in dieser Stimme klingen", prompt)
        self.assertIn("Maximal ein Satz.", prompt)

    def test_each_personality_has_its_own_cue_shots(self) -> None:
        cues = {
            "default": "Keine Extra-Formel",
            "butler": "Sehr wohl.",
            "locker": "Geht klar.",
            "fuersorglich": "Mache ich sofort.",
            "party": "Läuft!",
            "grantig": "Schon gut.",
            "sarkastisch": "Wie überraschend, wieder ein Befehl.",
            "pirat": "Aye.",
            "hippie": "Alles easy.",
            "gollum": "Ja, mein Schatz.",
        }
        flavor = {
            "default": "Keine Extra-Formel",
            "butler": "wie gewünscht",
            "locker": "passt",
            "fuersorglich": "alles gut",
            "party": "super!",
            "grantig": "na gut",
            "sarkastisch": "natürlich.",
            "pirat": "Käpt'n",
            "hippie": "ganz ruhig",
            "gollum": ", ja.",
        }
        for name, cue in cues.items():
            prompt = refine.refine_prompt("de", name, None)
            self.assertIn(cue, prompt, name)
            self.assertIn(flavor[name], prompt, name)

    def test_english_prompt_uses_personality(self) -> None:
        prompt = refine.refine_prompt("en", "locker", None)
        self.assertIn("Do not call Home Assistant tools", prompt)
        self.assertIn("casual", prompt)
        self.assertIn("Voice (mandatory)", prompt)
        self.assertIn("The cue alone is not enough", prompt)
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

    def test_options_personality_switches_refine_prompt_and_spoken_cue(self) -> None:
        self.assertEqual(const.resolve_personality("grantig"), "grantig")
        self.assertEqual(const.resolve_personality("nope"), "default")
        self.assertEqual(set(refine._PERSONALITY), set(const.PERSONALITIES))
        for pack in ("de", "en"):
            seen: set[str] = set()
            for name in const.PERSONALITIES:
                prompt = refine.refine_prompt(pack, name, None)
                spoken = speech.style("Licht ist an.", name, pack)
                if name == "default":
                    self.assertEqual(spoken, "Licht ist an.")
                    self.assertNotIn("Sehr wohl." if pack == "de" else "Very well.", prompt)
                    continue
                cue = spoken[: -len("Licht ist an.")].strip()
                self.assertTrue(cue, name)
                self.assertIn(cue, prompt, f"{name}/{pack}")
                self.assertNotIn(cue, seen)
                seen.add(cue)
        butler = refine.refine_prompt("de", const.resolve_personality("butler"), None)
        grantig = refine.refine_prompt("de", const.resolve_personality("grantig"), None)
        self.assertIn("Sehr wohl.", butler)
        self.assertNotIn("Schon gut.", butler)
        self.assertIn("Schon gut.", grantig)
        self.assertNotIn("Sehr wohl.", grantig)

    def test_ha_path_styles_before_refine_and_restores_dropped_cue(self) -> None:
        source = "Wohnzimmer Licht ist an."
        styled = speech.style(source, "butler", "de")
        self.assertEqual(styled, "Sehr wohl. Wohnzimmer Licht ist an.")
        dropped = refine.accept_refined(styled, "Das Licht im Wohnzimmer ist an.")
        self.assertEqual(dropped, "Das Licht im Wohnzimmer ist an.")
        self.assertEqual(
            speech.style(dropped, "butler", "de"),
            "Sehr wohl. Das Licht im Wohnzimmer ist an.",
        )
        kept = refine.accept_refined(styled, "Sehr wohl. Das Licht im Wohnzimmer ist an.")
        self.assertEqual(speech.style(kept, "butler", "de"), kept)

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
