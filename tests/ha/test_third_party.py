#!/usr/bin/env python3
"""Stdlib tests for third-party license notices."""

from __future__ import annotations

import importlib.util
import tarfile
import tempfile
import unittest
from io import BytesIO
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _load():
    path = ROOT / "scripts" / "third-party-notices.py"
    spec = importlib.util.spec_from_file_location("klar_notices", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


notices = _load()


class NoticeTests(unittest.TestCase):
    def test_skips_root_and_sorts(self) -> None:
        meta = {
            "packages": [
                {"id": "root", "name": "klar-nlu", "version": "1", "license": "MIT"},
                {"id": "a", "name": "axum", "version": "0.8.9", "license": "MIT"},
                {
                    "id": "b",
                    "name": "sync_wrapper",
                    "version": "1.0.2",
                    "license": "Apache-2.0",
                },
            ],
            "resolve": {
                "root": "root",
                "nodes": [
                    {"id": "root", "deps": [{"pkg": "a"}, {"pkg": "b"}]},
                    {"id": "a", "deps": []},
                    {"id": "b", "deps": []},
                ],
            },
        }
        crates = notices.crates_from_metadata(meta)
        self.assertEqual(
            crates,
            [("axum", "0.8.9", "MIT"), ("sync_wrapper", "1.0.2", "Apache-2.0")],
        )

    def test_render_has_required_texts(self) -> None:
        text = notices.render([("matchit", "0.8.4", "MIT AND BSD-3-Clause")])
        self.assertIn("matchit 0.8.4  MIT AND BSD-3-Clause", text)
        self.assertIn("===== Apache-2.0 =====", text)
        self.assertIn("===== BSD-3-Clause =====", text)
        self.assertIn("===== Unicode-3.0 =====", text)
        self.assertIn("See LICENSE", text)

    def test_tarball_keeps_binary_and_adds_notices(self) -> None:
        license_text = (ROOT / "LICENSE").read_text(encoding="utf-8")
        self.assertIn("MIT License", license_text)
        with tempfile.TemporaryDirectory() as tmp:
            dest = Path(tmp) / "klar"
            dest.write_bytes(b"binary")
            license_path = Path(tmp) / "LICENSE"
            third = Path(tmp) / "THIRD_PARTY"
            license_path.write_text(license_text, encoding="utf-8")
            third.write_text(notices.render([("axum", "0.8.9", "MIT")]), encoding="utf-8")
            blob = BytesIO()
            with tarfile.open(fileobj=blob, mode="w:gz") as tar:
                tar.add(dest, arcname="klar")
                tar.add(license_path, arcname="LICENSE")
                tar.add(third, arcname="THIRD_PARTY")
            blob.seek(0)
            with tarfile.open(fileobj=blob, mode="r:gz") as tar:
                names = {Path(item.name).name for item in tar.getmembers() if item.isfile()}
        self.assertEqual(names, {"klar", "LICENSE", "THIRD_PARTY"})


if __name__ == "__main__":
    unittest.main()
