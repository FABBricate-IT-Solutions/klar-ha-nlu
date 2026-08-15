#!/usr/bin/env python3
"""Set the project version (Home Assistant CalVer: YYYY.M.PATCH)."""

from __future__ import annotations

import re
import sys
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def parse_version(raw: str) -> tuple[int, int, int]:
    raw = raw.lstrip("vV")
    parts = raw.split(".")
    if len(parts) != 3 or not all(part.isdigit() for part in parts):
        raise ValueError(f"expected YYYY.M.PATCH, got {raw!r}")
    year, month, patch = (int(part) for part in parts)
    if month < 1 or month > 12:
        raise ValueError(f"month must be 1-12, got {month}")
    if patch < 0:
        raise ValueError(f"patch must be >= 0, got {patch}")
    return year, month, patch


def format_version(year: int, month: int, patch: int) -> str:
    return f"{year}.{month}.{patch}"


def next_version(current: str, today: date) -> str:
    year, month, patch = parse_version(current)
    if year < 2000:
        return format_version(today.year, today.month, 0)
    if (today.year, today.month) > (year, month):
        return format_version(today.year, today.month, 0)
    return format_version(year, month, patch + 1)


def current_version() -> str:
    for line in (ROOT / "Cargo.toml").read_text().splitlines():
        if line.startswith("version = "):
            return line.split('"', 2)[1]
    raise SystemExit("Cargo.toml: no version")


def replace_once(path: Path, pattern: str, repl: str) -> None:
    text = path.read_text()
    updated, n = re.subn(pattern, repl, text, count=1, flags=re.M)
    if n != 1:
        raise SystemExit(f"{path}: expected 1 version replacement, got {n}")
    path.write_text(updated)


def write_version(version: str) -> None:
    year, month, patch = parse_version(version)
    version = format_version(year, month, patch)
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
    replace_once(
        ROOT / "custom_components/klar_nlu/const.py",
        r'^ENGINE_VERSION = "[^"]+"',
        f'ENGINE_VERSION = "{version}"',
    )
    print(version)


def self_test() -> None:
    assert parse_version("2026.8.0") == (2026, 8, 0)
    assert parse_version("v2026.8.2") == (2026, 8, 2)
    assert parse_version("2026.08.1") == (2026, 8, 1)
    assert format_version(2026, 8, 0) == "2026.8.0"
    assert next_version("0.1.10", date(2026, 8, 15)) == "2026.8.0"
    assert next_version("2026.8.0", date(2026, 8, 15)) == "2026.8.1"
    assert next_version("2026.8.3", date(2026, 9, 1)) == "2026.9.0"
    assert next_version("2026.12.2", date(2027, 1, 3)) == "2027.1.0"
    assert next_version("2026.9.0", date(2026, 8, 15)) == "2026.9.1"
    print("ok")


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: bump-version.py next|YYYY.M.PATCH|--self-test")
    arg = sys.argv[1]
    if arg == "--self-test":
        self_test()
        return
    if arg == "next":
        print(next_version(current_version(), date.today()))
        return
    write_version(arg)


if __name__ == "__main__":
    main()
