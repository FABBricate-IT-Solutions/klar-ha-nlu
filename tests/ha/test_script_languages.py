#!/usr/bin/env python3
"""Every non-Latin pack speaks and parses in its own script."""

from __future__ import annotations

import importlib.util
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SURFACES = json.loads((ROOT / "scripts" / "lang_packs" / "native_surfaces.json").read_text(encoding="utf-8"))
sys.path.insert(0, str(ROOT / "scripts"))
from lang_packs.lexicons import ALL_CORES


def _load(name: str, rel: str):
    path = ROOT / "custom_components" / "klar_nlu" / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def _has_letters(text: str) -> bool:
    return any(char.isalpha() and not char.isascii() for char in text)


class ScriptLanguageTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.news = _load("klar_news_script", "news.py")
        cls.say = _load("klar_calendar_say_script", "calendar_say.py")

    def test_overlay_keeps_romanized_room_synonyms(self) -> None:
        ja = next(core for core in ALL_CORES if core["code"] == "ja")
        rooms = {token for token, _ha in ja["rooms"]}
        self.assertIn("リビング", rooms)
        self.assertIn("ribingu", rooms)

    def test_overlay_covers_script_families(self) -> None:
        needed = {
            "zh-CN",
            "zh-TW",
            "zh-HK",
            "ja",
            "ko",
            "th",
            "hi",
            "bn",
            "gu",
            "kn",
            "ml",
            "mr",
            "ta",
            "te",
            "pa",
            "ne",
            "hy",
            "ka",
            "mn",
        }
        self.assertEqual(needed, set(SURFACES))
        for code, row in SURFACES.items():
            with self.subTest(code=code):
                self.assertTrue(_has_letters(row["on"][0]), f"{code} on is still romanized")
                self.assertTrue(_has_letters(row["light"][0]), f"{code} light is still romanized")
                self.assertTrue(_has_letters(row["living"]), f"{code} living is still romanized")

    def test_assist_smokes_use_native_script(self) -> None:
        datasets = ROOT / "tests" / "datasets" / "assist"
        for code in SURFACES:
            path = datasets / code / "representative.yaml"
            raw = path.read_text(encoding="utf-8")
            with self.subTest(code=code):
                self.assertTrue(_has_letters(raw), f"{path} has no native letters")
                self.assertNotIn("tsukete", raw)
                self.assertNotIn("dakai", raw)
                self.assertNotIn("jalao", raw)
                self.assertNotIn("kyeo", raw)

    def test_calendar_say_and_llm_are_native(self) -> None:
        for code in SURFACES:
            item, created, deleted, moved, empty, today, tomorrow, *_rest = self.say.SAY[code]
            instruct, heading = self.say.LLM[code]
            with self.subTest(code=code):
                for label, text in (
                    ("item", item),
                    ("created", created),
                    ("empty", empty),
                    ("today", today),
                    ("llm", instruct),
                    ("heading", heading),
                ):
                    self.assertTrue(_has_letters(text), f"{code} {label} is romanized: {text}")

    def test_news_does_not_fall_back_to_german(self) -> None:
        self.assertIn("tagesschau", self.news.feed_url("de"))
        self.assertNotIn("tagesschau", self.news.feed_url("ja"))
        self.assertNotIn("tagesschau", self.news.feed_url("hi"))
        self.assertNotIn("tagesschau", self.news.feed_url("zh-CN"))
        self.assertNotIn("tagesschau", self.news.feed_url("sw"))
        self.assertEqual(self.news.feed_url("sw"), self.news.feed_url("en"))
        self.assertNotIn("Möchtest", self.news.nudge("ja"))
        self.assertNotIn("Möchtest", self.news.nudge("hi"))
        self.assertTrue(_has_letters(self.news.nudge("ja")))
        self.assertTrue(self.news.asked_for_more("もっと詳しく"))
        self.assertTrue(self.news.asked_for_more("想了解详情"))


if __name__ == "__main__":
    unittest.main()
