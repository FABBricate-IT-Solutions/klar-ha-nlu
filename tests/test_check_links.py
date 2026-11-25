#!/usr/bin/env python3
"""scripts/check_links.py: what it reports, and that the repository passes it."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def _load():
    path = ROOT / "scripts" / "check_links.py"
    spec = importlib.util.spec_from_file_location("klar_check_links", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


LINKS = _load()


class CheckTests(unittest.TestCase):
    def setUp(self) -> None:
        self._dir = tempfile.TemporaryDirectory()
        self.addCleanup(self._dir.cleanup)
        self.root = Path(self._dir.name)

    def write(self, name: str, text: str) -> Path:
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        return Path(name)

    def problems(self, *files: Path) -> list[str]:
        return LINKS.check(self.root, list(files))

    def test_links_that_resolve_are_accepted(self) -> None:
        self.write("docs/en/setup.md", "# Setup\n\n## First run\n\n## First run\n")
        self.write(".github/workflows/ci.yml", "name: CI\n")
        page = self.write(
            "docs/en/index.md",
            "# Index\n\n"
            "[setup](setup.md), [first](setup.md#first-run), [again](setup.md#first-run-1),\n"
            "[top](#index), [workflow](../../.github/workflows/ci.yml), [root](/docs/en/setup.md),\n"
            "[site](https://example.org/missing), <a href=\"mailto:a@example.org\">mail</a>\n\n"
            "[ref]: setup.md\n",
        )
        self.assertEqual(self.problems(page), [])

    def test_missing_file_is_reported_with_file_and_line(self) -> None:
        page = self.write("docs/en/releases.md", "# Releases\n\nSee [parity](../.github/workflows/parity.yml).\n")
        self.assertEqual(
            self.problems(page),
            ["docs/en/releases.md:3: ../.github/workflows/parity.yml does not exist"],
        )

    def test_missing_heading_is_reported(self) -> None:
        self.write("other.md", "# Other\n\n## Größe & Maße\n")
        page = self.write("page.md", "[ok](other.md#größe--maße) [bad](other.md#size) [here](#nowhere)\n")
        self.assertEqual(
            self.problems(page),
            [
                "page.md:1: other.md#size has no heading for #size",
                "page.md:1: #nowhere has no heading for #nowhere",
            ],
        )

    def test_reference_and_html_targets_are_checked(self) -> None:
        page = self.write("page.md", "[a][ref]\n\n[ref]: gone.md\n<img src=\"gone.png\">\n")
        self.assertEqual(
            self.problems(page),
            ["page.md:3: gone.md does not exist", "page.md:4: gone.png does not exist"],
        )

    def test_code_is_not_read_as_a_link(self) -> None:
        page = self.write("page.md", "`[x](gone.md)`\n\n```md\n[y](gone.md)\n# not a heading\n```\n")
        self.assertEqual(self.problems(page), [])
        self.assertEqual(LINKS.anchors((self.root / page).read_text(encoding="utf-8")), set())

    def test_link_out_of_the_repository_is_reported(self) -> None:
        page = self.write("page.md", "[up](../elsewhere.md)\n")
        self.assertEqual(self.problems(page), ["page.md:1: ../elsewhere.md points outside the repository"])


class RepositoryTests(unittest.TestCase):
    def test_the_checked_in_markdown_has_no_broken_link(self) -> None:
        files = LINKS.tracked_markdown(ROOT)
        self.assertGreater(len(files), 20)
        self.assertEqual(LINKS.check(ROOT, files), [])

    def test_changelogs_are_left_out(self) -> None:
        names = {path.name for path in LINKS.tracked_markdown(ROOT)}
        self.assertNotIn("CHANGELOG.md", names)
        self.assertIn("README.md", names)


if __name__ == "__main__":
    unittest.main()
