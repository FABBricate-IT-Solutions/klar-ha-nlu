#!/usr/bin/env python3
"""Stdlib tests for LLM reply refinement helpers."""

from __future__ import annotations

import asyncio
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
const = _load("klar_nlu.const", "const.py")
voices = _load("refine_voices", "refine_voices.py")
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
        self.assertIn("ein Satz", prompt)
        self.assertIn("Uhrzeiten ohne Sekunden", prompt)
        self.assertIn("14:44 nicht 14:44:55", prompt)
        self.assertIn("Ein oder zwei Sätze.", prompt)
        self.assertIn("Offene Fragen", prompt)
        self.assertIn("Ist die Vorlage eine Frage, bleibt die Antwort eine Frage.", prompt)
        self.assertNotIn("Formel: Sehr wohl.", prompt)
        self.assertNotIn("Hänge immer an", prompt)
        self.assertNotIn("besorgt", prompt)
        self.assertNotIn("soweit gemeldet", prompt)
        self.assertNotIn("Butler", prompt)

    def test_empty_extra_uses_builtin_personality_voice(self) -> None:
        prompt = refine.refine_prompt("de", "butler", None)
        self.assertIn("Butler", prompt)
        self.assertIn("Klebe nicht jedes Mal dieselbe Eröffnung davor.", prompt)
        self.assertIn("Status", prompt)

    def test_stored_prompt_replaces_builtin_voice(self) -> None:
        prompt = refine.refine_prompt("de", "butler", "Stimme: Jarvis.\nBeispiele:\nX → Y")
        self.assertIn("Keine Home-Assistant-Werkzeuge", prompt)
        self.assertIn("Jarvis", prompt)
        self.assertNotIn("Butler", prompt)

    def test_personality_change_swaps_stored_prompt(self) -> None:
        butler = voices.editable_prompt("butler", "de")
        jarvis = voices.editable_prompt("jarvis", "de")
        self.assertIn("Butler", butler)
        self.assertIn("Jarvis", jarvis)
        self.assertIn("21,5 °C", butler)
        self.assertIn("Schlafzimmerlicht ist an.", butler)
        swapped = voices.resolve_stored_prompt("jarvis", "butler", butler, "de")
        self.assertEqual(swapped, jarvis)
        kept = voices.resolve_stored_prompt("butler", "butler", "mein stil", "de")
        self.assertEqual(kept, "mein stil")
        filled = voices.resolve_stored_prompt("grantig", "grantig", "", "de")
        self.assertEqual(filled, voices.editable_prompt("grantig", "de"))

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
            "jarvis": "Jarvis",
        }
        seen: set[str] = set()
        for name, marker in flavor.items():
            prompt = refine.refine_prompt("de", name, None)
            self.assertIn(marker, prompt, name)
            self.assertNotIn("Hänge immer an", prompt, name)
            self.assertRegex(prompt, r"→ .+\.", name)
            if name == "default":
                self.assertNotIn("Es ist eingeschaltet", prompt)
                self.assertNotIn("That is done.", refine.refine_prompt("en", "default", None))
            self.assertNotIn(prompt, seen)
            seen.add(prompt)

    def test_english_prompt_uses_personality(self) -> None:
        prompt = refine.refine_prompt("en", "locker", None)
        self.assertIn("Do not call Home Assistant tools", prompt)
        self.assertIn("casual", prompt)
        self.assertIn("Voice:", prompt)
        self.assertIn("open questions", prompt.lower())
        self.assertIn("Do not stamp the same opening every time.", prompt)
        self.assertIn("all set", prompt)
        self.assertIn("Clock times without seconds", prompt)
        self.assertIn("Do not translate into German", prompt)
        self.assertNotIn("Additional style instruction", prompt)

    def test_german_stored_extra_is_ignored_for_other_packs(self) -> None:
        prompt = refine.refine_prompt("en", "butler", "Stimme: Jarvis.\nSchalt-Bestätigungen")
        self.assertIn("Do not translate into German", prompt)
        self.assertIn("Voice:", prompt)
        self.assertNotIn("Stimme:", prompt)

    def test_nlu_home_turn_removed_because_every_reply_refines(self) -> None:
        self.assertFalse(hasattr(refine, "nlu_home_turn"))

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
        self.assertIsNone(
            refine.accept_refined("Wohnzimmer TV ist an.", "Das Licht im Wohnzimmer ist an.")
        )
        self.assertIsNone(
            refine.accept_refined("Der Fernseher ist gerade nicht erreichbar.", "Das Licht im Wohnzimmer ist an.")
        )
        self.assertTrue(refine.skip_rewrite("error"))
        self.assertIsNone(refine.accept_refined("Temperatur im Schlafzimmer.", "Wie ist die Temperatur im Schlafzimmer?"))
        self.assertEqual(
            refine.accept_refined("Meinst du Küche oder Wohnzimmer?", "Küche oder Wohnzimmer, Sir?"),
            "Küche oder Wohnzimmer, Sir?",
        )
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
        self.assertEqual(refine.clean_refined("Es ist 14:44:55."), "Es ist 14:44.")
        self.assertEqual(refine.accept_refined("Es ist 14:44.", "Es ist 14:44:55."), "Es ist 14:44.")

    def test_should_refine_any_spoken_reply(self) -> None:
        self.assertTrue(
            refine.should_refine(True, "conversation.llm", "Licht ist an.")
        )
        self.assertTrue(
            refine.should_refine(True, "conversation.llm", "Im Wohnzimmer sind es 21,5 °C.")
        )
        self.assertTrue(refine.should_refine(True, "conversation.llm", "Hallo"))
        self.assertTrue(refine.should_refine(True, "conversation.llm", "Die Nachrichten."))
        self.assertFalse(
            refine.should_refine(False, "conversation.llm", "Licht ist an.")
        )
        self.assertFalse(refine.should_refine(True, None, "Licht ist an."))
        self.assertFalse(refine.should_refine(True, "conversation.llm", ""))
        self.assertFalse(hasattr(refine, "_TIMEOUT"))
        self.assertFalse(hasattr(refine, "nlu_home_turn"))

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
            for name in const.PERSONALITIES:
                prompt = refine.refine_prompt(pack, name, None)
                spoken = speech.style("Licht ist an.", name, pack)
                self.assertNotIn(prompt, seen_prompts)
                seen_prompts.add(prompt)
                variants = list((speech._locale(pack).get("personality") or {}).get(name) or [""])
                expected = {f"{prefix}Licht ist an." for prefix in variants}
                expected.add("Licht ist an.")
                if name == "default":
                    self.assertEqual(spoken, "Licht ist an.")
                    continue
                self.assertTrue(variants, name)
                self.assertIn(spoken, expected, name)
        butler = refine.refine_prompt("de", const.resolve_personality("butler"), None)
        grantig = refine.refine_prompt("de", const.resolve_personality("grantig"), None)
        self.assertIn("Butler", butler)
        self.assertNotIn("grantig", butler)
        self.assertIn("grantig", grantig)
        self.assertNotIn("Butler", grantig)
        spoken = speech.style("Licht ist an.", "butler", "de")
        variants = set((speech._locale("de").get("personality") or {}).get("butler") or [])
        self.assertTrue(variants, "butler variants")
        self.assertIn(spoken, {f"{prefix}Licht ist an." for prefix in variants})

    def test_successful_refine_keeps_natural_line_without_restamping_cue(self) -> None:
        source = "Wohnzimmer Licht ist an."
        natural = "Das Licht im Wohnzimmer ist an. Ich habe es für Sie eingeschaltet."
        self.assertEqual(refine.accept_refined(source, natural), natural)
        spoken = speech.style(source, "butler", "de")
        variants = set((speech._locale("de").get("personality") or {}).get("butler") or [])
        self.assertIn(spoken, {f"{prefix}{source}" for prefix in variants})

    def test_other_packs_do_not_use_german_wrapper(self) -> None:
        for pack in ("fr", "nl", "ja"):
            prompt = refine.refine_prompt(pack, "butler", None)
            self.assertNotIn("Stimme:", prompt, pack)
            self.assertNotIn("Klebe nicht jedes Mal", prompt, pack)
            self.assertIn("same language", prompt.lower(), pack)
            self.assertIn("input line", prompt.lower(), pack)

    def test_accept_refined_rejects_bureaucratic_stamps(self) -> None:
        self.assertIsNone(refine.accept_refined("Licht ist an.", "Zur Kenntnis genommen. Licht ist an."))
        self.assertIsNone(refine.accept_refined("Licht ist an.", "Das ist besorgt."))
        self.assertIsNone(refine.accept_refined("Licht ist an.", "soweit gemeldet"))

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

    def test_skip_rewrite_for_llm_replies_only(self) -> None:
        self.assertTrue(refine.skip_rewrite("chat"))
        self.assertTrue(refine.skip_rewrite("llm"))
        self.assertTrue(refine.skip_rewrite("chime"))
        self.assertFalse(refine.skip_rewrite("execute"))
        self.assertFalse(refine.skip_rewrite("clarify"))
        self.assertFalse(refine.skip_rewrite("trigger"))
        self.assertFalse(refine.skip_rewrite(""))

    def test_isolated_ids_are_unique_and_prefixed(self) -> None:
        first = refine.isolated_conversation_id()
        second = refine.isolated_conversation_id()
        self.assertTrue(first.startswith("klar-nested-"))
        self.assertTrue(second.startswith("klar-nested-"))
        self.assertNotEqual(first, second)

    def test_nested_llm_session_never_targets_satellite(self) -> None:
        session = refine.nested_llm_session("conversation.llm", "de", "Stimme: Jarvis.")
        self.assertIsNone(session["device_id"])
        self.assertIsNone(session["satellite_id"])
        self.assertEqual(session["agent_id"], "conversation.llm")
        self.assertEqual(session["language"], "de")
        self.assertEqual(session["extra_system_prompt"], "Stimme: Jarvis.")

    def test_drop_same_turn_assistant_only_after_user(self) -> None:
        previous = types.SimpleNamespace(role="assistant", content="old")
        user = types.SimpleNamespace(role="user", content="hi")
        leaked = types.SimpleNamespace(role="assistant", content="unrefined")
        content = [previous, user, leaked]
        refine.drop_same_turn_assistant(content)
        self.assertEqual(content, [previous, user])
        refine.drop_same_turn_assistant(content)
        self.assertEqual(content, [previous, user])

    def test_speech_chunks_split_on_punctuation(self) -> None:
        chunks = refine.speech_chunks(
            "Natürlich, Sir. Im Wohnzimmer: Heizung ist 24,89. R2D2 ist pausiert."
        )
        self.assertEqual(
            chunks,
            [
                "Natürlich, Sir.",
                " Im Wohnzimmer: Heizung ist 24,89.",
                " R2D2 ist pausiert.",
            ],
        )
        self.assertEqual("".join(chunks), "Natürlich, Sir. Im Wohnzimmer: Heizung ist 24,89. R2D2 ist pausiert.")
        self.assertEqual(refine.speech_chunks("Licht ist an."), ["Licht ist an."])
        self.assertEqual(refine.speech_chunks(""), [])
        self.assertEqual(refine.speech_chunks("OK"), ["OK"])
        self.assertEqual(refine.speech_chunks("z.B. Licht ist an."), ["z.B. Licht ist an."])
        self.assertEqual(
            refine.speech_chunks("Set to 21.5 degrees. The light is on."),
            ["Set to 21.5 degrees.", " The light is on."],
        )

    def test_emit_streams_sentences_as_deltas(self) -> None:
        class Log:
            def __init__(self) -> None:
                self.content = [
                    types.SimpleNamespace(role="user", content="hi"),
                    types.SimpleNamespace(role="assistant", content="unrefined"),
                ]
                self.deltas: list[dict[str, str]] = []
                self.without: list[str] = []

            def async_add_delta_content_stream(self, agent_id: str | None, stream):
                del agent_id

                async def gen():
                    parts: list[str] = []
                    async for delta in stream:
                        self.deltas.append(delta)
                        parts.append(delta.get("content") or "")
                    self.content.append("".join(parts))
                    yield None

                return gen()

            def async_add_assistant_content_without_tools(self, body: str) -> None:
                self.without.append(body)

        log = Log()
        speech = "Natürlich, Sir. Das Licht im Wohnzimmer ist an."
        asyncio.run(refine.emit_assistant_speech(log, "conversation.klar_nlu", speech))
        self.assertEqual(log.content[-1], speech)
        self.assertEqual(log.without, [])
        self.assertEqual(log.deltas[0]["role"], "assistant")
        self.assertEqual(log.deltas[0]["content"], "Natürlich, Sir.")
        self.assertEqual(log.deltas[1], {"content": " Das Licht im Wohnzimmer ist an."})

    def test_emit_falls_back_without_delta_stream(self) -> None:
        class Log:
            def __init__(self) -> None:
                self.content = [types.SimpleNamespace(role="user", content="hi")]
                self.without: list[str] = []

            def async_add_assistant_content_without_tools(self, body: str) -> None:
                self.without.append(body)

        log = Log()
        asyncio.run(refine.emit_assistant_speech(log, "conversation.klar_nlu", "Licht ist an."))
        self.assertEqual(log.without, ["Licht ist an."])

    def test_tts_hears_first_published_line_not_later_rewrite(self) -> None:
        nlu = "Wohnzimmer Licht ist an."
        refined = "Natürlich, Sir. Das Licht im Wohnzimmer ist an."
        published = [nlu]
        self.assertNotEqual(published[-1], refined)
        published = []
        speech = nlu
        if not refine.skip_rewrite("execute"):
            speech = refined
        published.append(speech)
        self.assertEqual(published[-1], refined)
        llm = "Bereits im Jarvis-Ton."
        published = []
        speech = llm
        if not refine.skip_rewrite("chat"):
            speech = refined
        published.append(speech)
        self.assertEqual(published[-1], llm)

    def test_fallback_converse_must_not_reuse_voice_session(self) -> None:
        src = (PKG / "conversation.py").read_text()
        start = src.index("async def _fallback")
        end = src.index("def _preferred_area")
        body = src[start:end]
        self.assertIn("isolated_conversation_id", body)
        self.assertIn("nested_llm_session", body)
        self.assertIn("speak_tag(pack)", body)
        self.assertNotIn("user_input.conversation_id", body)
        self.assertNotIn("user_input.device_id", body)
        self.assertNotIn("record", body)
        spoken = src[src.index("async def _spoken") : src.index("async def _briefing")]
        self.assertIn("skip_rewrite", spoken)
        self.assertIn("emit_assistant_speech", spoken)
        self.assertNotIn("async_add_assistant_content_without_tools", spoken)


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
