"""Tests for fuzzy artist correction on failed music searches."""
import ast
import difflib
import re
import unittest
from pathlib import Path

SOURCE = Path(__file__).resolve().parents[1] / "dispatch_media.py"
TREE = ast.parse(SOURCE.read_text())
WANTED = {"clean_media_query", "_token_similarity", "choose_artist", "correction_for_query"}
NAMESPACE = {"difflib": difflib, "re": re, "_PUNCT": re.compile(r"[^\w\s]+", re.UNICODE),
             "_ROOM_FILLER": {"in", "on", "at", "the", "a", "an", "display", "speaker", "room", "bedroom", "kitchen", "living", "dining"}}
for node in TREE.body:
    if isinstance(node, ast.FunctionDef) and node.name in WANTED:
        exec(compile(ast.Module(body=[node], type_ignores=[]), str(SOURCE), "exec"), NAMESPACE)
choose_artist = NAMESPACE["choose_artist"]
clean_media_query = NAMESPACE["clean_media_query"]
correction_for_query = NAMESPACE["correction_for_query"]


class FuzzyArtistTests(unittest.TestCase):
    LIBRARY = [
        "Katelyn Tarver", "Calvin Harris", "Queen", "2WEI", "Christina Aguilera",
        "KT Tunstall", "New Order", "Art Garfunkel", "Joy Division", "SHAED",
    ]

    def test_clean_query_removes_trailing_room_words(self):
        self.assertEqual(clean_media_query("caitlin tarver in the bedroom"), "caitlin tarver")
        self.assertEqual(clean_media_query("caitlin tarver in"), "caitlin tarver")

    def test_matches_misspelled_full_name(self):
        self.assertEqual(choose_artist("caitlin tarver in", self.LIBRARY), "Katelyn Tarver")
        self.assertEqual(choose_artist("kaitlyn tarver", self.LIBRARY), "Katelyn Tarver")
        self.assertEqual(choose_artist("caitlyn tarver", self.LIBRARY), "Katelyn Tarver")

    def test_does_not_match_without_shared_significant_token(self):
        self.assertIsNone(choose_artist("caitlin underwood", self.LIBRARY))

    def test_single_word_requires_higher_score(self):
        self.assertEqual(choose_artist("quen", self.LIBRARY), "Queen")
        self.assertIsNone(choose_artist("que", self.LIBRARY))

    def test_no_candidates(self):
        self.assertIsNone(choose_artist("caitlin tarver", []))

    def test_matching_is_case_insensitive(self):
        self.assertEqual(choose_artist("KATELYN TARVER", self.LIBRARY), "Katelyn Tarver")
        self.assertEqual(choose_artist("cAiTlYn TaRvEr", self.LIBRARY), "Katelyn Tarver")

    def test_correction_payload(self):
        self.assertEqual(
            correction_for_query("caitlin tarver in", self.LIBRARY),
            {"media_id": "Katelyn Tarver", "media_type": "artist"},
        )

    def test_random_sample_misses_are_now_matched(self):
        cases = {
            "kt tunsall": "KT Tunstall",
            "new odrer": "New Order",
            "art garfunkl": "Art Garfunkel",
            "joy divisin": "Joy Division",
        }
        for query, expected in cases.items():
            self.assertEqual(choose_artist(query, self.LIBRARY), expected)

    def test_similar_artist_names_are_not_confused(self):
        self.assertEqual(choose_artist("shade", self.LIBRARY), "SHAED")
