#!/usr/bin/env python3
"""Factual questions are answered from a fetched source, not model memory."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PATH = ROOT / "custom_components" / "klar_nlu" / "lookup.py"
SPEC = importlib.util.spec_from_file_location("klar_lookup_test", PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load {PATH}")
lookup = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(lookup)

SOURCE = "Präsident der Vereinigten Staaten. Der Amtsinhaber ist Donald Trump."


class LookupTests(unittest.TestCase):
    def test_query_uses_the_question(self) -> None:
        self.assertEqual(lookup.lookup_query("  Wer   ist der Präsident der USA? "), "Wer ist der Präsident der USA?")
        self.assertEqual(lookup.wiki_lang("de-AT"), "de")
        url = lookup.search_url("de", "Wer ist der Präsident der USA")
        self.assertTrue(url.startswith("https://de.wikipedia.org/w/api.php?"))
        self.assertIn("Pr%C3%A4sident", url)
        self.assertIn("srlimit=5", url)
        self.assertTrue(lookup.extract_url("en", "President of the United States").startswith("https://en.wikipedia.org/"))

    def test_parsers_keep_the_intro(self) -> None:
        title = lookup.search_title({"query": {"search": [{"title": "Präsident der Vereinigten Staaten"}]}})
        self.assertEqual(title, "Präsident der Vereinigten Staaten")
        skipped = lookup.search_title(
            {"query": {"search": [{"title": "Liste der Präsidenten der Vereinigten Staaten"}, {"title": title}]}}
        )
        self.assertEqual(skipped, title)
        self.assertEqual(lookup.search_title({"query": {"search": []}}), "")
        intro = lookup.intro_text({"query": {"pages": {"1": {"extract": "Der Amtsinhaber ist Donald Trump."}}}})
        self.assertIn("Donald Trump", intro)
        self.assertEqual(lookup.intro_text({"query": {"pages": {"-1": {"missing": ""}}}}), "")

    def test_ungrounded_or_echoed_answers_are_rejected(self) -> None:
        asked = "Wer ist der Präsident der USA"
        self.assertTrue(lookup.echoes_question(asked, "Wer ist der Präsident der USA?"))
        self.assertFalse(lookup.grounded(asked, "Wer ist der Präsident der USA?", SOURCE))
        self.assertFalse(lookup.grounded(asked, "Der Präsident ist Joe Biden.", SOURCE))
        self.assertTrue(lookup.grounded(asked, "Der Amtsinhaber ist Donald Trump.", SOURCE))
        self.assertFalse(lookup.grounded("Wie viele Einwohner hat Deutschland?", "Deutschland hat 99 Millionen Einwohner.", "etwa 83 Millionen"))

    def test_missing_source_is_a_short_reply(self) -> None:
        self.assertEqual(lookup.lookup_miss("de"), "Dazu habe ich keine aktuelle Quelle.")
        self.assertIn("kein eigenes Wissen", lookup.lookup_instruction("de", [SOURCE]))
        self.assertTrue(lookup.source_speech([SOURCE]).startswith("Präsident"))

    def test_conversation_looks_up_before_the_model(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(encoding="utf-8")
        start = src.index('reason == "out_of_domain"')
        block = src[start : src.index("fallback = await self._fallback", start)]
        self.assertIn("return await self._sourced_answer", block)
        body = src[src.index("async def _sourced_answer") : src.index("async def _briefing")]
        self.assertIn("lookup_facts", body)
        self.assertIn("lookup_instruction", body)
        self.assertIn("grounded", body)
        self.assertIn("source_speech", body)


if __name__ == "__main__":
    unittest.main()
