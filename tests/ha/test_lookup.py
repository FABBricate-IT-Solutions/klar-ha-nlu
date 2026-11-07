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

    def test_country_facts_come_from_wikidata(self) -> None:
        self.assertEqual(lookup.fact_kind("Wie groß ist Deutschland?"), "area")
        self.assertEqual(lookup.fact_kind("wie gross ist deutschland"), "area")
        self.assertEqual(lookup.fact_kind("Wer ist der Präsident von Deutschland?"), "head")
        self.assertEqual(lookup.fact_subject("Wer ist der Präsident von Deutschland?", "head"), "deutschland")
        self.assertEqual(lookup.fact_subject("Wie viele Einwohner hat Deutschland?", "population"), "deutschland")
        self.assertEqual(lookup.fact_kind("How big is France?"), "area")
        self.assertEqual(lookup.fact_subject("How big is France?", "area"), "france")
        self.assertEqual(lookup.fact_subject("Wer ist der Bundeskanzler?", "government"), "deutschland")
        self.assertEqual(lookup.fact_subject("wie groß ist denn deutschland", "area"), "deutschland")
        search = {
            "search": [
                {"id": "Q1299", "description": "song by Rammstein"},
                {"id": "Q183", "description": "country in Central Europe"},
                {"id": "Q7318", "description": "Germany from 1933 to 1945"},
            ]
        }
        self.assertEqual(lookup.pick_country(search), "Q183")
        self.assertEqual(
            lookup.pick_country({"search": [{"id": "Q980", "description": "federated state in the south of Germany"}, {"id": "Q183", "description": "country in Central Europe"}]}),
            "Q183",
        )
        self.assertEqual(lookup.pick_country({"search": [{"id": "Q980", "description": "federated state in the south of Germany"}]}), "Q980")
        self.assertEqual(
            lookup.entity_label({"entities": {"Q22686": {"labels": {"mul": {"value": "Donald Trump"}}}}}, "Q22686", "de"),
            "Donald Trump",
        )
        self.assertEqual(lookup.spoken_number("83467117", "de"), "83,5 Millionen")
        self.assertEqual(lookup.spoken_number("357588", "de"), "357588")
        claims = {
            "claims": {
                "P1082": [
                    {"rank": "normal", "mainsnak": {"snaktype": "value", "datavalue": {"value": {"amount": "+70945000", "unit": "1"}}}},
                    {"rank": "preferred", "mainsnak": {"snaktype": "value", "datavalue": {"value": {"amount": "+83467117", "unit": "1"}}}},
                ],
                "P2046": [
                    {"rank": "preferred", "mainsnak": {"snaktype": "value", "datavalue": {"value": {"amount": "+357587.77", "unit": "km"}}}},
                ],
                "P35": [
                    {"rank": "preferred", "mainsnak": {"snaktype": "value", "datavalue": {"value": {"id": "Q76658"}}}},
                ],
            }
        }
        self.assertEqual(lookup.plain_amount(lookup.preferred_value(claims, "P1082")), "83467117")
        self.assertEqual(lookup.plain_amount(lookup.preferred_value(claims, "P2046")), "357588")
        labels = {"entities": {"Q183": {"labels": {"de": {"value": "Deutschland"}}}, "Q76658": {"labels": {"de": {"value": "Frank-Walter Steinmeier"}}}}}
        self.assertEqual(
            lookup.measure_sentence("de", "area", lookup.entity_label(labels, "Q183", "de"), "357588"),
            "Deutschland hat eine Fläche von 357588 Quadratkilometern.",
        )
        self.assertEqual(
            lookup.office_sentence("de", "head", "Deutschland", lookup.entity_label(labels, "Q76658", "de")),
            "Staatsoberhaupt von Deutschland ist Frank-Walter Steinmeier.",
        )
        self.assertIn("P2046", lookup.wikidata_claims_url("Q183", "P2046"))
        ranked = lookup.search_title(
            {"query": {"search": [{"title": "Vincent Gross"}, {"title": "Deutschland"}]}},
            "wie gross ist deutschland",
        )
        self.assertEqual(ranked, "Deutschland")
        priced = lookup.search_title(
            {"query": {"search": [{"title": "Großer Preis von Deutschland"}, {"title": "Deutschland"}]}},
            "wie groß ist denn deutschland",
        )
        self.assertEqual(priced, "Deutschland")

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
