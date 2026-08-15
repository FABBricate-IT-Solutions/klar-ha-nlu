#!/usr/bin/env python3
"""Print the CHANGELOG.md section for a CalVer tag.

GitHub release notes must not come from `git-cliff --latest` after a squash
whose subject is `chore(release):` — cliff.toml skips those commits, so the
published body is only the heading and footer.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HEADING = re.compile(r"^## \[?v?(?P<version>[0-9][0-9.]*)\]?.*$", re.M)


def section(text: str, version: str) -> str:
    version = version.lstrip("vV")
    matches = list(HEADING.finditer(text))
    for i, match in enumerate(matches):
        if match.group("version") != version:
            continue
        end = matches[i + 1].start() if i + 1 < len(matches) else len(text)
        return text[match.start() : end].strip() + "\n"
    return ""


def main(argv: list[str]) -> int:
    if len(argv) != 2 or argv[1] in {"-h", "--help"}:
        print("usage: release-notes.py VERSION", file=sys.stderr)
        return 2
    version = argv[1].lstrip("vV")
    notes = section((ROOT / "CHANGELOG.md").read_text(encoding="utf-8"), version)
    if not notes or not re.search(r"^### ", notes, re.M):
        print(f"CHANGELOG.md has no grouped notes for {version}", file=sys.stderr)
        return 1
    sys.stdout.write(notes)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
