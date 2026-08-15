#!/usr/bin/env python3
"""Stdlib tests for Klar archive checksums."""

from __future__ import annotations

import hashlib
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _load_require_sha256():
    path = ROOT / "custom_components" / "klar_nlu" / "archive.py"
    spec = importlib.util.spec_from_file_location("klar_archive", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod.require_sha256


require_sha256 = _load_require_sha256()


class DigestTests(unittest.TestCase):
    def test_ok(self) -> None:
        blob = b"klar"
        require_sha256("sha256:" + hashlib.sha256(blob).hexdigest(), blob)

    def test_missing(self) -> None:
        with self.assertRaises(RuntimeError):
            require_sha256(None, b"x")

    def test_wrong_algo(self) -> None:
        with self.assertRaises(RuntimeError):
            require_sha256("sha1:abc", b"x")

    def test_mismatch(self) -> None:
        with self.assertRaises(RuntimeError):
            require_sha256("sha256:" + ("0" * 64), b"x")


if __name__ == "__main__":
    unittest.main()
