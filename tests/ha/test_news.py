#!/usr/bin/env python3
"""Stdlib tests for news briefing helpers."""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path
from urllib.parse import parse_qs, urlparse

ROOT = Path(__file__).resolve().parents[2]
PKG = ROOT / "custom_components" / "klar_nlu"
if str(PKG) not in sys.path:
    sys.path.insert(0, str(PKG))


def _load(name: str, rel: str):
    path = ROOT / "custom_components" / "klar_nlu" / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


news = _load("klar_news", "news.py")
PACKS = _load("klar_languages", "languages.py").SUPPORTED_LANGUAGES

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
        for closing in (
            "Tre nyheter i dag. Vill du höra mer om någon?",
            "ثلاثة أخبار. هل تريد المزيد؟",
            "Τρεις ειδήσεις. Θέλεις περισσότερα για κάποια;",
            "三条新闻。想听哪一条？",
            'Kolme uutista. "Haluatko kuulla lisää?"',
            "Երեք լուր։ Ուզո՞ւմ ես ավելին։",
        ):
            self.assertTrue(news.asked_for_more(closing), closing)
        self.assertFalse(news.asked_for_more("Vad hände? Tre skolor stängdes i dag."))
        self.assertFalse(news.asked_for_more("Tre nyheter i dag."))
        self.assertFalse(news.asked_for_more("Ի՞նչ եղավ։ Երեք դպրոց փակվեց։"))

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

    def test_every_pack_reads_and_asks_in_its_own_language(self) -> None:
        self.assertEqual(len(PACKS), 67)
        english = {news.feed_url("en"), news.feed_url("en-GB"), news.feed_url("kw")}
        for pack in PACKS:
            self.assertTrue(news.nudge(pack).strip(), f"{pack} has no nudge")
            url = news.feed_url(pack)
            self.assertTrue(url.startswith("https://"), pack)
            if pack not in {"en", "en-GB", "kw"}:
                self.assertNotIn(url, english, f"{pack} reads the English feed")
            if not pack.startswith("de"):
                self.assertNotIn("tagesschau", url, pack)
                self.assertNotEqual(news.nudge(pack), news.nudge("de"), pack)
        self.assertEqual(news.nudge("xx"), "")
        self.assertIn("höra", news.nudge("sv"))
        for pack, hl, ceid in (("it", "it", "IT:it"), ("nb", "no", "NO:no"), ("pt", "pt-PT", "PT:pt-150"), ("sr-Latn", "sr", "RS:sr")):
            found = parse_qs(urlparse(news.feed_url(pack)).query)
            self.assertEqual((found["hl"], found["ceid"]), ([hl], [ceid]), pack)
        self.assertIn("persian", news.feed_url("fa"))
        self.assertIn("marathi", news.feed_url("mr"))

    def test_topic_search_uses_the_edition_of_the_pack(self) -> None:
        for pack, hl, gl, ceid in (
            ("sv", "sv", "SE", "SE:sv"),
            ("zh-CN", "zh-CN", "CN", "CN:zh-Hans"),
            ("pt-BR", "pt-BR", "BR", "BR:pt-419"),
            ("de-AT", "de", "AT", "AT:de"),
            ("is", "is", "US", "US:is"),
        ):
            found = parse_qs(urlparse(news.search_url(pack, "tesla")).query)
            self.assertEqual((found["q"], found["hl"], found["gl"], found["ceid"]), (["tesla"], [hl], [gl], [ceid]), pack)

    def test_packs_without_an_edition_search_their_own_feed(self) -> None:
        stories = [
            {"title": "Tesla innkallar bíla", "text": "Um 100 þúsund bílar.", "link": ""},
            {"title": "Veðrið í dag", "text": "Rigning.", "link": ""},
        ]
        self.assertEqual([story["title"] for story in news.about(stories, "tesla")], ["Tesla innkallar bíla"])
        self.assertEqual(news.about(stories, "fótbolti"), [])
        self.assertEqual(news.about(stories, ""), [])

    def test_google_titles_lose_the_publisher(self) -> None:
        raw = """<rss><channel>
          <item><title>Tesla återkallar bilar - Aftonbladet</title>
            <description>&lt;a href="https://a.se/1"&gt;Tesla återkallar bilar&lt;/a&gt;&amp;nbsp;&lt;font&gt;Aftonbladet&lt;/font&gt;</description>
            <source url="https://a.se">Aftonbladet</source></item>
          <item><title>Regn - och mer regn</title><description>Hela veckan.</description></item>
        </channel></rss>"""
        self.assertEqual(news.headlines_from_xml(raw), ["Tesla återkallar bilar", "Regn - och mer regn"])
        stories = news.stories_from_xml(raw)
        self.assertEqual(news.spoken_story(stories[0]), "Tesla återkallar bilar")
        self.assertEqual(news.spoken_story(stories[1]), "Regn - och mer regn. Hela veckan.")

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
        named = [
            {"title": "Ilse Aigner im Porträt", "text": "Sie spricht über Krisen und wird bald Staatsoberhaupt.", "link": ""},
            {"title": "Digitaler Neustart der Kreisverwaltung OSL", "text": "Die Kreisverwaltung startet ihre Verwaltung neu.", "link": ""},
            {"title": "Eine andere Sicherheitslücke", "text": "Experten warnen vor einer alten Lücke.", "link": ""},
        ]
        self.assertEqual(
            news.pick_story("kannst du mir den artikel ueber den neustart der kreisverwaltung osl vorlesen", named)["title"],
            "Digitaler Neustart der Kreisverwaltung OSL",
        )
        self.assertIsNone(news.pick_story("kannst du mir den artikel darueber vorlesen", named))
        self.assertEqual(
            news.pick_story("lese die nachricht ueber die sicherheitsluecke vor", named)["title"],
            "Eine andere Sicherheitslücke",
        )
        self.assertIsNone(
            news.pick_story("lese die nachricht ueber die sicherheitsluecke phptoshell vor", [named[2]])
        )
        self.assertEqual(news.story_query("lese mir die nachricht ueber den neustart der kreisverwaltung osl vor"), "neustart kreisverwaltung osl")
        schools = [
            {"title": "Drohmails gegen Schulen", "text": "Die Polizei vermutet Destabilisierung.", "link": ""},
            {"title": "DAX fällt zurück", "text": "Tech-Aktien sinken.", "link": ""},
        ]
        self.assertEqual(news.pick_story("zu den schulen bitte mehr", schools)["title"], "Drohmails gegen Schulen")
        self.assertEqual(news.story_query("gib mir dazu alle aktuellen nachrichten"), "")
        urls = news.story_feed_urls("de", "", None, ["https://example.com/mine.xml"])
        self.assertEqual(urls[0], "https://example.com/mine.xml")
        self.assertIn("tagesschau", urls[1])
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

    def test_briefing_is_one_reply_and_does_not_reopen_the_microphone(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text()
        briefing = src[src.index("async def _briefing") : src.index("def _nlu_rag")]
        self.assertNotIn("announce(", briefing)
        self.assertEqual(briefing.count(', False, "chat")'), 2)
        self.assertNotIn(', True, "chat")', briefing)

    def test_spoken_titles_when_the_model_is_silent(self) -> None:
        self.assertEqual(news.spoken_headlines(["Erste Meldung", "Zweite."]), "Erste Meldung. Zweite")
        spoken = news.compose_speech("Intro.", news.spoken_headlines(["Eine Meldung"]), "Mehr?", False)
        self.assertIn("Intro.", spoken)
        self.assertIn("Eine Meldung", spoken)
        self.assertIn("Mehr?", spoken)

    def test_h19_entities_atom_google_url_and_followups(self) -> None:
        rss = """<?xml version="1.0"?><rss><channel>
          <item><title><![CDATA[Trump &amp; Xi: &#8220;Deal&#8221; reached]]></title>
            <link>https://example.com/a</link>
            <description>&lt;p&gt;Body &amp;amp; more&lt;/p&gt;</description></item>
        </channel></rss>"""
        self.assertEqual(news.headlines_from_xml(rss), ['Trump & Xi: “Deal” reached'])
        atom = """<?xml version="1.0"?><feed xmlns="http://www.w3.org/2005/Atom">
          <entry><title>Story</title>
            <link rel="replies" type="application/atom+xml" href="https://blog.example/feeds/1/comments/default"/>
            <link rel="alternate" type="text/html" href="https://blog.example/2026/11/story.html"/>
            <summary>short</summary></entry></feed>"""
        self.assertEqual(news.stories_from_xml(atom)[0]["link"], "https://blog.example/2026/11/story.html")
        google = (
            "https://news.google.com/rss/articles/"
            "CBMiogFBVV95cUxQdWpXY2ZfU3hZT1pCQ0xQb2FfS1pXZ2xqR0pSbm9lS1dYZ3R4N2x0bV9qQnFtV1Bf"
            "N3hLMk1yZDR0c1lZb0tQc2ZxX3VmVU5sTjZfWGNvaHZBb0lCQ3NhT2s2bEVTcG5xbUF4cWZnT2NZT1pC"
            "SWxKZ0ZJRl9pUGxxVUpaNU1sV2xFc1ZCWXJDVlBYZ1k0T2c?oc=5"
        )
        self.assertGreater(len(google), 200)
        self.assertTrue(news._http_url(google))
        self.assertFalse(news.wants_read("was gibt es neues vor ort"))
        stories = [{"title": f"Story {i}", "text": "", "link": ""} for i in range(1, 6)]
        self.assertIsNone(news.pick_story("wait a second", stories))
        self.assertIsNone(news.pick_story("tell me more about the last election", stories))
        self.assertEqual(news.pick_story("lese die zweite nachricht vor", stories)["title"], "Story 2")

    def test_r22_article_html_is_capped(self) -> None:
        body = "<article>" + "".join(f"<p>{'Satz mit einigen Woertern. ' * 6}{i}</p>" for i in range(400)) + "</article>"
        self.assertGreater(len(body), 60_000)
        out = news.article_body_from_html(body)
        self.assertLessEqual(len(out), 12_000)
        self.assertLessEqual(news._MAX_FETCH_BYTES, 256 * 1024)
        self.assertTrue(callable(news.load_article))


if __name__ == "__main__":
    unittest.main()
