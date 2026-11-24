"""Tests for preserving parsed reminder speech."""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from reminders import keep_parsed_speech_for_reminders  # noqa: E402


class ReminderSpeechTests(unittest.TestCase):
    def test_keeps_engine_speech_for_reminder_intent(self):
        intents = [{"name": "SetReminderRelative", "slots": []}]
        self.assertTrue(keep_parsed_speech_for_reminders(intents, "Reminder: take my medicine."))

    def test_does_not_keep_when_engine_speech_missing(self):
        intents = [{"name": "SetReminderRelative", "slots": []}]
        self.assertFalse(keep_parsed_speech_for_reminders(intents, ""))

    def test_does_not_keep_for_non_reminder_intent(self):
        intents = [{"name": "HassTurnOn", "slots": []}]
        self.assertFalse(keep_parsed_speech_for_reminders(intents, "Done."))


if __name__ == "__main__":
    unittest.main()
