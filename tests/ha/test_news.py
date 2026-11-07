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

    def test_custom_feed_replaces_only_that_language(self) -> None:
        feeds = [{"pack": "de", "url": "https://example.com/de.xml"}]
        self.assertEqual(news.choose_feed("de", "", feeds), ("custom", "https://example.com/de.xml"))
        kind, url = news.choose_feed("en", "", feeds)
        self.assertEqual(kind, "builtin")
        self.assertIn("bbci", url)
        self.assertEqual(news.custom_feed("de", [{"pack": "de", "url": "https://user:pass@example.com/a.xml"}]), "")
        self.assertEqual(news.custom_feed("de", [{"pack": "de", "url": "https://e"}]), "")
        self.assertEqual(news.custom_feed("de", [{"pack": "de", "url": "http://localhost/rss"}]), "http://localhost/rss")

    def test_topic_uses_section_or_search(self) -> None:
        kind, url = news.choose_feed("de", "sport", None)
        self.assertEqual(kind, "section")
        self.assertIn("tagesschau.de/sport", url)
        kind, url = news.choose_feed("de-AT", "Sport", None)
        self.assertEqual(kind, "section")
        kind, url = news.choose_feed("de", "berlin", None)
        self.assertEqual(kind, "search")
        self.assertIn("news.google.com/rss/search", url)
        self.assertIn("q=berlin", url)
        self.assertIn("hl=de", url)
        self.assertEqual(news.bare_topic("lese vor"), "")
        self.assertEqual(news.bare_topic("lese sport vor"), "sport")
        self.assertEqual(news.choose_feed("de", "lese vor", None)[0], "builtin")
        described = """<?xml version="1.0" encoding="UTF-8"?>
        <rss><channel>
          <item><title>Erste Meldung</title><description>Text eins.</description></item>
          <item><title>Hubig prüft das AfD-Verbotsverfahren</title><description>Die E-Mails werden untersucht.</description></item>
          <item><title>Letzte Meldung</title><description>&lt;p&gt;Dürre-Übung.&lt;/p&gt;</description></item>
        </channel></rss>"""
        stories = news.stories_from_xml(described)
        self.assertEqual(stories[2]["text"], "Dürre-Übung.")
        self.assertEqual(news.pick_story("lese die letzte nachricht vor", stories)["title"], "Letzte Meldung")
        self.assertEqual(news.pick_story("ja ueber das afd verbotsverfahren", stories)["title"], "Hubig prüft das AfD-Verbotsverfahren")
        self.assertIsNone(news.pick_story("lese nachrichten vor", stories))
        self.assertIn("E-Mails", news.spoken_story(stories[1]))
        page = """<html><script type="application/ld+json">{"@type":"NewsArticle","articleBody":"Erster Absatz. Zweiter Absatz der ganzen Meldung."}</script></html>"""
        self.assertEqual(news.article_body_from_html(page), "Erster Absatz. Zweiter Absatz der ganzen Meldung.")
        bbc = """<html><script data-rh="true" type="application/ld+json">{"@type":"ReportageNewsArticle","description":"Kurz."}</script>
        <article><p>In the aftermath of the vote the president elevated his adviser to chief of staff and prepared for a fight with Congress.</p>
        <p>Follow the twists of this term with the newsletter.</p>
        <p>On Friday he installed a partisan firebrand and signaled that compromise with the new majority was off the table.</p></article></html>"""
        body = news.article_body_from_html(bbc)
        self.assertIn("chief of staff", body)
        self.assertIn("firebrand", body)
        self.assertNotIn("newsletter", body)
        embedded = """<rss xmlns:content="http://purl.org/rss/1.0/modules/content/"><channel><item><title>Lang</title><description>Kurz.</description><content:encoded><![CDATA[<p>Das ist der volle Text der Meldung mit genug Länge für den Vorlese-Pfad und noch einem zweiten Satz.</p>]]></content:encoded></item></channel></rss>"""
        self.assertIn("volle Text", news.stories_from_xml(embedded)[0]["text"])
        google = "https://news.google.com/rss/articles/CBMiabc?oc=5"
        self.assertEqual(news.google_article_id(google), "CBMiabc")
        self.assertEqual(news.google_article_id("https://www.bbc.co.uk/news/articles/abc"), "")
        signed = '<div data-n-a-sg="sig-1" data-n-a-ts="1700000000000"></div>'
        self.assertEqual(news.google_signature(signed), ("sig-1", "1700000000000"))
        decoded = '[["wrb.fr","Fbv4je","[\\"garturlres\\",\\"https://www.tagesspiegel.de/berlin/live.html\\",1]"]]'
        self.assertEqual(news.google_decoded_url(decoded), "https://www.tagesspiegel.de/berlin/live.html")
        self.assertIn("ganzen Meldung", news.spoken_story(stories[1], news.article_body_from_html(page)))
        self.assertEqual(
            news.pick_story("lesen sie vor", stories, "ja ueber das afd verbotsverfahren")["title"],
            "Hubig prüft das AfD-Verbotsverfahren",
        )
        kind, url = news.choose_feed("de", "lese sport vor", None)
        self.assertEqual(kind, "section")
        self.assertIn("tagesschau.de/sport", url)

    def test_feedreader_titles_and_urls(self) -> None:
        class Entry:
            data = {"url": "https://example.com/mine.xml"}

        class Config:
            def async_entries(self, domain: str) -> list:
                self.domain = domain
                return [Entry()]

        class Hass:
            data = {"feedreader": {"titles": ["Aus dem Reader"]}}
            config_entries = Config()

        titles, urls = news.feedreader_sources(Hass())
        self.assertEqual(titles, ["Aus dem Reader"])
        self.assertEqual(urls, ["https://example.com/mine.xml"])

    def test_spoken_titles_when_the_model_is_silent(self) -> None:
        self.assertEqual(news.spoken_headlines(["Erste Meldung", "Zweite."]), "Erste Meldung. Zweite")
        spoken = news.compose_speech("Intro.", news.spoken_headlines(["Eine Meldung"]), "Mehr?", False)
        self.assertIn("Intro.", spoken)
        self.assertIn("Eine Meldung", spoken)
        self.assertIn("Mehr?", spoken)


if __name__ == "__main__":
    unittest.main()
