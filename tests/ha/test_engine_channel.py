#!/usr/bin/env python3
"""Channel helpers for bundled-engine stable vs staging downloads."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]


def _load_const():
    languages = types.ModuleType("klar_channel_test.languages")
    languages.LANGUAGE_VARIANTS = {}
    languages.SUPPORTED_LANGUAGES = ("de", "en")
    package = types.ModuleType("klar_channel_test")
    package.__path__ = []
    with patch.dict(
        sys.modules,
        {
            "klar_channel_test": package,
            "klar_channel_test.languages": languages,
        },
    ):
        path = ROOT / "custom_components" / "klar_nlu" / "const.py"
        spec = importlib.util.spec_from_file_location("klar_channel_test.const", path)
        if spec is None or spec.loader is None:
            raise RuntimeError(f"cannot load {path}")
        module = importlib.util.module_from_spec(spec)
        sys.modules["klar_channel_test.const"] = module
        spec.loader.exec_module(module)
        return module


const = _load_const()


class EngineChannelTests(unittest.TestCase):
    def test_resolve_channel_defaults_stable(self) -> None:
        self.assertEqual(const.resolve_channel(None), const.CHANNEL_STABLE)
        self.assertEqual(const.resolve_channel("stable"), const.CHANNEL_STABLE)
        self.assertEqual(const.resolve_channel("nightly"), const.CHANNEL_STABLE)
        self.assertEqual(const.resolve_channel("staging"), const.CHANNEL_STAGING)

    def test_pick_staging_skips_latest_and_non_staging(self) -> None:
        releases = [
            {"tag_name": "2026.8.30", "prerelease": False, "name": "stable"},
            {"tag_name": "2026.8.30-rc.1", "prerelease": True, "name": "other"},
            {
                "tag_name": "2026.8.30-staging.abc1234",
                "prerelease": True,
                "name": "wanted",
            },
        ]
        chosen = const.pick_staging_release(releases)
        self.assertIsNotNone(chosen)
        self.assertEqual(chosen["name"], "wanted")

    def test_pick_staging_requires_prerelease_flag(self) -> None:
        self.assertIsNone(
            const.pick_staging_release(
                [{"tag_name": "2026.8.30-staging.deadbee", "prerelease": False}]
            )
        )
        self.assertIsNone(const.pick_staging_release("nope"))


if __name__ == "__main__":
    unittest.main()
