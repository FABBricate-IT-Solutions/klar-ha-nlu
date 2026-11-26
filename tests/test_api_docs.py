#!/usr/bin/env python3
"""Every HTTP route from the router appears in the API docs (package 5.5 / X-01)."""

from __future__ import annotations

import importlib.util
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def _load(name: str, relative: str):
    path = ROOT / relative
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    spec.loader.exec_module(module)
    return module


def _documented_paths(text: str) -> set[str]:
    found: set[str] = set()
    for match in re.finditer(r"`(/api/[A-Za-z0-9_\-/{}]+|/)`", text):
        found.add(match.group(1).rstrip("/") or "/")
    # Table cells and headings also use bare /api/... without always wrapping /.
    for match in re.finditer(r"(?<![`\w])(/api/[A-Za-z0-9_\-/{}]+)(?![`\w])", text):
        found.add(match.group(1).rstrip("/"))
    return found


class ApiDocsTests(unittest.TestCase):
    def test_every_router_route_is_documented_in_english_and_german(self) -> None:
        gen = _load("klar_gen_api_routes", "scripts/gen_api_routes.py")
        routes = gen.router_routes()
        for relative in ("docs/en/api.md", "docs/api.md"):
            with self.subTest(doc=relative):
                text = (ROOT / relative).read_text(encoding="utf-8")
                documented = _documented_paths(text)
                missing = sorted(path for path in routes if path not in documented and path != "/")
                # GET / is documented as the operator UI heading.
                if "/" in routes and "/" not in documented and "GET `/`" not in text and "`/`" not in text:
                    missing.append("/")
                self.assertEqual(missing, [], f"{relative} missing routes: {missing}")

    def test_generated_route_index_is_current(self) -> None:
        gen = _load("klar_gen_api_routes_check", "scripts/gen_api_routes.py")
        self.assertEqual(gen.main(["--check"]), 0)

    def test_testing_docs_do_not_recommend_cargo_test_or_thread_caps(self) -> None:
        banned = re.compile(r"cargo\s+test\b|--test-threads")
        for relative in ("docs/en/testing.md", "docs/testing.md", "docs/en/languages.md", "docs/languages.md"):
            with self.subTest(doc=relative):
                text = (ROOT / relative).read_text(encoding="utf-8")
                # Allow mentioning the ban itself.
                hits = [
                    line
                    for line in text.splitlines()
                    if banned.search(line) and "never" not in line.lower() and "nie" not in line.lower()
                ]
                self.assertEqual(hits, [], f"{relative} still recommends cargo test / --test-threads:\n" + "\n".join(hits))


if __name__ == "__main__":
    unittest.main()
