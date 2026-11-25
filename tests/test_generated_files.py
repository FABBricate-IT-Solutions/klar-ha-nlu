#!/usr/bin/env python3
"""Generators write what is checked in: Home Assistant translations and the Wohnung suites.

Each generator runs into a scratch directory here; CI also runs all of them in place and
fails on any difference in the tree (scripts/check_generated.sh).
"""

from __future__ import annotations

import contextlib
import io
import json
import re
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from lang_packs.ha_ui import DEST, STRINGS, flatten, write_ha_translations  # noqa: E402
from voice_suite.gen_wohnung import main as write_wohnung  # noqa: E402

DATASETS = ROOT / "tests" / "datasets"
PLACEHOLDER = re.compile(r"\{(\w+)\}")


def generate(writer) -> Path:
    """Run a generator into a scratch directory and return that directory."""
    holder = tempfile.TemporaryDirectory()
    unittest.addModuleCleanup(holder.cleanup)
    out = Path(holder.name)
    with contextlib.redirect_stdout(io.StringIO()):
        writer(out)
    return out


def differing(generated: Path, checked_in: Path) -> list[str]:
    """Generated files that are missing from the tree or have other content there."""
    names = sorted(str(path.relative_to(generated)) for path in generated.rglob("*") if path.is_file())
    if not names:
        raise AssertionError("the generator wrote nothing")
    return [
        name
        for name in names
        if not (checked_in / name).is_file() or (checked_in / name).read_bytes() != (generated / name).read_bytes()
    ]


class TranslationTests(unittest.TestCase):
    def test_generator_writes_the_checked_in_translations(self) -> None:
        generated = generate(write_ha_translations)
        self.assertEqual(differing(generated, DEST), [])
        self.assertEqual(
            sorted(path.name for path in generated.iterdir()),
            sorted(path.name for path in DEST.glob("*.json")),
        )

    def test_every_translation_keeps_the_placeholders_of_the_english_text(self) -> None:
        english = flatten(json.loads(STRINGS.read_text(encoding="utf-8")))
        expected = {key: set(PLACEHOLDER.findall(text)) for key, text in english.items()}
        self.assertTrue(any(expected.values()), "strings.json has no placeholder; the check would be empty")
        wrong: list[str] = []
        for path in sorted(DEST.glob("*.json")):
            flat = flatten(json.loads(path.read_text(encoding="utf-8")))
            for key, names in expected.items():
                if set(PLACEHOLDER.findall(flat.get(key, ""))) != names:
                    wrong.append(f"{path.name}: {key}")
        self.assertEqual(wrong, [])


class WohnungSuiteTests(unittest.TestCase):
    def test_generator_writes_the_checked_in_suites(self) -> None:
        generated = generate(write_wohnung)
        self.assertEqual(
            sorted(path.name for path in generated.iterdir()), ["wohnung_en", "wohnung_mittel"]
        )
        self.assertEqual(differing(generated, DATASETS), [])


if __name__ == "__main__":
    unittest.main()
