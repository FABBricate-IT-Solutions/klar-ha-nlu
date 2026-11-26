#!/usr/bin/env python3
"""Doc pins track Cargo.toml CalVer (package 5.5 / V-11)."""

from __future__ import annotations

import importlib.util
import re
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CALVER = re.compile(r"20\d\d\.(?:[1-9]|1[0-2])\.\d+")
IMAGE_PIN = re.compile(r"ghcr\.io/fabbricate-it-solutions/klar-nlu(?:-[a-z0-9]+)?:(20\d\d\.(?:[1-9]|1[0-2])\.\d+)\b")
TREE_PIN = re.compile(r"(?:Current tree|this tree|Aktueller Stand): \*\*(20\d\d\.(?:[1-9]|1[0-2])\.\d+)\*\*")

# Files that may state the current engine CalVer as a pin (not a historical example).
PIN_FILES = (
    "README.md",
    "README.de.md",
    "SECURITY.md",
    "docs/en/home-assistant.md",
    "docs/home-assistant.md",
    "addon/DOCS.md",
)


def _bump():
    path = ROOT / "scripts" / "bump-version.py"
    spec = importlib.util.spec_from_file_location("klar_bump_version_docs", path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def cargo_version() -> str:
    return tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]


class DocVersionTests(unittest.TestCase):
    def test_doc_pins_match_cargo_toml(self) -> None:
        expected = cargo_version()
        for name in PIN_FILES:
            with self.subTest(file=name):
                text = (ROOT / name).read_text(encoding="utf-8")
                pins = IMAGE_PIN.findall(text) + TREE_PIN.findall(text)
                self.assertTrue(pins, f"{name}: expected at least one current-tree / image pin")
                self.assertEqual(set(pins), {expected}, f"{name}: pins {pins} != {expected}")

    def test_no_stale_klar_nlu_image_tag_anywhere_in_markdown(self) -> None:
        expected = cargo_version()
        stale: list[str] = []
        for path in sorted(ROOT.rglob("*.md")):
            if "node_modules" in path.parts or path.name == "CHANGELOG.md":
                continue
            text = path.read_text(encoding="utf-8")
            for match in IMAGE_PIN.finditer(text):
                if match.group(1) != expected:
                    stale.append(f"{path.relative_to(ROOT)}:{match.group(0)}")
        self.assertEqual(stale, [], "stale image tags:\n" + "\n".join(stale))

    def test_bump_script_rewrites_doc_pins(self) -> None:
        bump = _bump()
        sample = (
            "Current tree: **2026.9.5**.\n"
            "  ghcr.io/fabbricate-it-solutions/klar-nlu:2026.9.5\n"
            "this tree: **2026.9.5**\n"
            "Aktueller Stand: **2026.9.5**.\n"
        )
        updated = bump.rewrite_doc_pins(sample, "2026.9.5", "2031.2.7")
        self.assertIn("Current tree: **2031.2.7**", updated)
        self.assertIn("klar-nlu:2031.2.7", updated)
        self.assertIn("this tree: **2031.2.7**", updated)
        self.assertIn("Aktueller Stand: **2031.2.7**", updated)
        self.assertNotIn("2026.9.5", updated)


if __name__ == "__main__":
    unittest.main()
