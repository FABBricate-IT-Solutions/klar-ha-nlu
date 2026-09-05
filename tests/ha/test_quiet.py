#!/usr/bin/env python3
"""Quiet-ack eligibility and chime bytes."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _load():
    path = ROOT / "custom_components" / "klar_nlu" / "quiet.py"
    spec = importlib.util.spec_from_file_location("klar_quiet", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


quiet = _load()


def _ok(name: str, **slots: str) -> tuple[dict, list[dict]]:
    item = {"name": name, "slots": [{"name": key, "value": value} for key, value in slots.items()]}
    executed = {"outcome": "success", "steps": [{"intent": name, "status": "success"}]}
    return executed, [item]


class QuietAckTests(unittest.TestCase):
    def test_area_light_on_off(self) -> None:
        on = _ok("HassTurnOn", area="wohnzimmer", domain="light")
        off = _ok("HassTurnOff", entity_id="light.wohnzimmer")
        self.assertTrue(quiet.quiet_ack_applies(*on, True))
        self.assertFalse(quiet.quiet_ack_applies(*on, False))
        self.assertFalse(quiet.quiet_ack_applies(*on))
        self.assertFalse(quiet.quiet_ack_applies(*off))

    def test_rejects_queries_and_scenes(self) -> None:
        query = _ok("HassGetState", area="wohnzimmer")
        scene = _ok("HassTurnOn", entity_id="scene.filmabend")
        climate = _ok("HassTurnOn", domain="climate", area="wohnzimmer")
        two = (
            {"outcome": "success", "steps": [{"status": "success"}, {"status": "success"}]},
            [{"name": "HassTurnOn", "slots": [{"name": "domain", "value": "light"}]}],
        )
        failed = (
            {"outcome": "error", "steps": [{"status": "error"}]},
            [{"name": "HassTurnOff", "slots": [{"name": "domain", "value": "light"}]}],
        )
        self.assertFalse(quiet.quiet_ack_applies(*query))
        self.assertFalse(quiet.quiet_ack_applies(*scene))
        self.assertFalse(quiet.quiet_ack_applies(*climate))
        self.assertFalse(quiet.quiet_ack_applies(*two))
        self.assertFalse(quiet.quiet_ack_applies(*failed))
        self.assertFalse(quiet.quiet_ack_applies(None, None))

    def test_chime_is_wav(self) -> None:
        blob = quiet.chime_wav()
        self.assertTrue(blob.startswith(b"RIFF"))
        self.assertIn(b"WAVE", blob[:16])
        self.assertGreater(len(blob), 1000)
        self.assertIs(quiet.chime_wav(), blob)


if __name__ == "__main__":
    unittest.main()
