#!/usr/bin/env python3
"""Set the project version in Cargo.toml, add-on config, and the HA manifest."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path: Path, pattern: str, repl: str) -> None:
    text = path.read_text()
    updated, n = re.subn(pattern, repl, text, count=1, flags=re.M)
    if n != 1:
        raise SystemExit(f"{path}: expected 1 version replacement, got {n}")
    path.write_text(updated)


def main() -> None:
    if len(sys.argv) != 2 or not re.fullmatch(r"\d+\.\d+\.\d+", sys.argv[1]):
        raise SystemExit("usage: bump-version.py X.Y.Z")
    version = sys.argv[1]
    replace_once(ROOT / "Cargo.toml", r'^version = "[^"]+"', f'version = "{version}"')
    replace_once(
        ROOT / "Cargo.lock",
        r'(?<=name = "klar-nlu"\nversion = ")[^"]+',
        version,
    )
    replace_once(ROOT / "config.yaml", r'^version: "[^"]+"', f'version: "{version}"')
    replace_once(ROOT / "addon/config.yaml", r'^version: "[^"]+"', f'version: "{version}"')
    replace_once(
        ROOT / "custom_components/klar_nlu/manifest.json",
        r'"version": "[^"]+"',
        f'"version": "{version}"',
    )
    print(version)


if __name__ == "__main__":
    main()
