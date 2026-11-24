"""Tests for reminder number-word normalization (stdlib only)."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "custom_components" / "klar_nlu"))

from number_words import normalize_number_words  # noqa: E402


class NumberWordTests(unittest.TestCase):
    def test_single_digits_and_teens(self):
        for word, digit in (
            ("one", "1"), ("two", "2"), ("nine", "9"), ("ten", "10"),
            ("eleven", "11"), ("nineteen", "19"),
        ):
            text = f"remind me in {word} minutes to test item"
            self.assertEqual(
                normalize_number_words(text),
                f"remind me in {digit} minutes to test item",
            )

    def test_tens_and_compounds(self):
        for word, digit in (
            ("twenty", "20"), ("thirty", "30"), ("ninety", "90"),
            ("twenty five", "25"), ("forty two", "42"),
        ):
            text = f"remind me in {word} minutes to test item"
            self.assertEqual(
                normalize_number_words(text),
                f"remind me in {digit} minutes to test item",
            )

    def test_hyphenated_compound(self):
        self.assertEqual(
            normalize_number_words("remind me in twenty-five minutes to test"),
            "remind me in 25 minutes to test",
        )

    def test_non_reminder_unchanged(self):
        for text in (
            "play one dance track",
            "turn on the kitchen light",
            "what is the temperature in the kitchen",
        ):
            self.assertEqual(normalize_number_words(text), text)

    def test_reminder_without_number_unchanged(self):
        text = "remind me tomorrow morning to take the trash out"
        self.assertEqual(normalize_number_words(text), text)

    def test_digits_not_changed(self):
        text = "remind me in 10 minutes to feed the cats"
        self.assertEqual(normalize_number_words(text), text)


if __name__ == "__main__":
    unittest.main()
