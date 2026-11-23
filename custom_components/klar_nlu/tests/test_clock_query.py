"""Deterministic date/day/time answers must not go through the LLM."""
import sys
import unittest
from datetime import datetime
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from clock_query import clock_query_speech

NOW = datetime(2026, 11, 23, 13, 5)


class ClockQueryTests(unittest.TestCase):
    def test_date_phrasings(self):
        for phrase in ("what is the date", "what's the date", "whats the date",
                       "tell me the date", "what date is it", "what's today's date"):
            with self.subTest(phrase=phrase):
                self.assertEqual(clock_query_speech(phrase, "en", NOW), "Today is Monday, November 23rd, 2026.")

    def test_day_phrasings(self):
        for phrase in ("what day is it", "what day of the week is it", "what's the day", "whats today's day"):
            with self.subTest(phrase=phrase):
                self.assertEqual(clock_query_speech(phrase, "en", NOW), "Today is Monday.")

    def test_time_phrasings(self):
        for phrase in ("what time is it", "what's the time", "what is the time", "whats the current time"):
            with self.subTest(phrase=phrase):
                self.assertEqual(clock_query_speech(phrase, "en", NOW), "It's 1:05 PM.")

    def test_today_combines_date_and_time(self):
        self.assertEqual(
            clock_query_speech("what's today", "en", NOW),
            "Today is Monday, November 23rd, 2026. The time is 1:05 PM.",
        )

    def test_ordinals_and_clock_edges(self):
        cases = [
            (datetime(2026, 11, 1, 0, 5), "Today is Sunday, November 1st, 2026."),
            (datetime(2026, 11, 2, 12, 0), "Today is Monday, November 2nd, 2026."),
            (datetime(2026, 11, 3, 23, 59), "Today is Tuesday, November 3rd, 2026."),
            (datetime(2026, 11, 11, 9, 7), "Today is Wednesday, November 11th, 2026."),
            (datetime(2026, 11, 21, 1, 2), "Today is Saturday, November 21st, 2026."),
        ]
        for stamp, expected in cases:
            with self.subTest(stamp=stamp):
                self.assertEqual(clock_query_speech("what's the date", "en", stamp), expected)
        self.assertEqual(clock_query_speech("what time is it", "en", datetime(2026, 11, 23, 0, 0)), "It's 12:00 AM.")
        self.assertEqual(clock_query_speech("what time is it", "en", datetime(2026, 11, 23, 12, 0)), "It's 12:00 PM.")

    def test_non_matching_and_other_packs_return_none(self):
        self.assertIsNone(clock_query_speech("turn on the kitchen light", "en", NOW))
        self.assertIsNone(clock_query_speech("wie spät ist es", "de", NOW))
        self.assertIsNone(clock_query_speech("what's the time", "de", NOW))
        self.assertIsNone(clock_query_speech("what day is it", "", NOW))

    def test_punctuation_and_whitespace(self):
        self.assertEqual(clock_query_speech("  What's the time?  ", "en", NOW), "It's 1:05 PM.")


if __name__ == "__main__":
    unittest.main()
