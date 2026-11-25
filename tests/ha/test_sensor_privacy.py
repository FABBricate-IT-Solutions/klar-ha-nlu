"""D-20: diagnostic sensors must not keep the spoken utterance by default."""

from __future__ import annotations

import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class SensorPrivacyTests(unittest.TestCase):
    def test_conversation_gates_sensor_on_raw_text_setting(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "conversation.py").read_text(encoding="utf-8")
        self.assertIn('user_input.text if self._store_raw_text() else ""', src)
        self.assertIn('speech if self._store_raw_text() else ""', src)
        self.assertIn('self._engine_settings().get("support_bundle_raw_text")', src)

    def test_remember_turn_writes_last_heard_from_argument(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "sensor.py").read_text(encoding="utf-8")
        self.assertIn('"last_heard": (text or "")[:255]', src)
        self.assertIn("def remember_turn(", src)


if __name__ == "__main__":
    unittest.main()
