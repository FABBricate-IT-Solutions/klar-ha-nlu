#!/usr/bin/env python3
"""One version everywhere: the crate, the lock file, the integration and the add-on."""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import re
import shutil
import tempfile
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CALVER = re.compile(r"^20\d\d\.([1-9]|1[0-2])\.\d+$")

CARGO_TOML = "Cargo.toml"
CARGO_LOCK = "Cargo.lock"
MANIFEST = "custom_components/klar_nlu/manifest.json"
CONST = "custom_components/klar_nlu/const.py"
ADDON = "addon/config.yaml"
FILES = (CARGO_TOML, CARGO_LOCK, MANIFEST, CONST, ADDON)


def _match(root: Path, name: str, pattern: str) -> str:
    found = re.findall(pattern, (root / name).read_text(encoding="utf-8"), flags=re.M)
    if len(found) != 1:
        raise AssertionError(f"{name}: expected one version, found {found}")
    return found[0]


def read_versions(root: Path) -> dict[str, str]:
    """The version each file states, read the way its consumer reads it."""
    package = tomllib.loads((root / CARGO_TOML).read_text(encoding="utf-8"))["package"]
    locked = [
        entry["version"]
        for entry in tomllib.loads((root / CARGO_LOCK).read_text(encoding="utf-8"))["package"]
        if entry["name"] == package["name"]
    ]
    if len(locked) != 1:
        raise AssertionError(f"{CARGO_LOCK}: expected one entry for {package['name']}, found {locked}")
    return {
        CARGO_TOML: package["version"],
        CARGO_LOCK: locked[0],
        MANIFEST: json.loads((root / MANIFEST).read_text(encoding="utf-8"))["version"],
        CONST: _match(root, CONST, r'^ENGINE_VERSION = "([^"]+)"$'),
        ADDON: _match(root, ADDON, r'^version: "([^"]+)"$'),
    }


def _bump_script():
    path = ROOT / "scripts" / "bump-version.py"
    spec = importlib.util.spec_from_file_location("klar_bump_version", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class VersionTests(unittest.TestCase):
    def copy_of_the_version_files(self) -> Path:
        holder = tempfile.TemporaryDirectory()
        self.addCleanup(holder.cleanup)
        root = Path(holder.name)
        for name in FILES:
            (root / name).parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, root / name)
        return root

    def test_every_file_states_the_same_calver_version(self) -> None:
        versions = read_versions(ROOT)
        self.assertEqual(len(set(versions.values())), 1, versions)
        self.assertRegex(versions[CARGO_TOML], CALVER)

    def test_a_file_left_behind_is_noticed(self) -> None:
        current = read_versions(ROOT)[CARGO_TOML]
        for name in FILES:
            with self.subTest(file=name):
                root = self.copy_of_the_version_files()
                path = root / name
                text = path.read_text(encoding="utf-8")
                self.assertIn(f'"{current}"', text)
                path.write_text(text.replace(f'"{current}"', '"2031.2.7"'), encoding="utf-8")
                versions = read_versions(root)
                self.assertEqual(versions[name], "2031.2.7")
                self.assertEqual(len(set(versions.values())), 2, versions)

    def test_the_bump_script_moves_every_file(self) -> None:
        root = self.copy_of_the_version_files()
        bump = _bump_script()
        bump.ROOT = root
        with contextlib.redirect_stdout(io.StringIO()):
            bump.write_version("2031.2.7")
        self.assertEqual(set(read_versions(root).values()), {"2031.2.7"})


if __name__ == "__main__":
    unittest.main()
