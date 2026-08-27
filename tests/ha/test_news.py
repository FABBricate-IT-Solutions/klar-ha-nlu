#!/usr/bin/env python3
"""Stdlib tests for news briefing helpers."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _load(name: str, rel: str):
    path = ROOT / "custom_components" / "klar_nlu" / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


news = _load("klar_news", "news.py")
fallback = _load("klar_fallback", "fallback.py")

RSS = """<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Feed</title>
    <item><title>Erste Meldung</title></item>
    <item><title>Zweite Meldung</title></item>
    <item><title>Dritte Meldung</title></item>
  </channel>
</rss>
"""


class NewsTests(unittest.TestCase):
    def test_headlines_from_rss(self) -> None:
        titles = news.headlines_from_xml(RSS, limit=2)
        self.assertEqual(titles, ["Erste Meldung", "Zweite Meldung"])

    def test_empty_and_bad_xml(self) -> None:
        self.assertEqual(news.headlines_from_xml(""), [])
        self.assertEqual(news.headlines_from_xml("<not>xml"), [])

    def test_asked_for_more(self) -> None:
        self.assertTrue(news.asked_for_more("Möchtest du zu einer mehr erfahren?"))
        self.assertTrue(news.asked_for_more("Möchten Sie mehr über eine dieser Meldungen erfahren?"))
        self.assertTrue(news.asked_for_more("Would you like to hear more?"))
        self.assertFalse(news.asked_for_more("Hier die drei wichtigsten Meldungen."))

    def test_compose_keeps_intro_until_announced(self) -> None:
        spoken = news.compose_speech("Intro.", "LLM-Text.", "Nachhaken?", False)
        self.assertIn("Intro.", spoken)
        self.assertIn("LLM-Text.", spoken)
        self.assertIn("Nachhaken?", spoken)
        announced = news.compose_speech("Intro.", "LLM-Text.", "", True)
        self.assertEqual(announced, "LLM-Text.")

    def test_unknown_pack_uses_english_feed_not_german(self) -> None:
        self.assertNotIn("tagesschau", news.feed_url("ja"))
        self.assertEqual(news.feed_url("xx"), news.feed_url("en"))
        self.assertNotIn("Möchtest", news.nudge("ja"))

    def test_news_prompt_lists_headlines(self) -> None:
        prompt = fallback.news_prompt("de", ["Erste Meldung"], "Sei kurz.")
        self.assertIn("Sei kurz.", prompt)
        self.assertIn("Erste Meldung", prompt)
        self.assertIn("keine Home-Assistant-Werkzeuge", prompt)
        follow = fallback.news_followup_prompt("de", None)
        self.assertIn("Nachrichtenthema", follow)


if __name__ == "__main__":
    unittest.main()
