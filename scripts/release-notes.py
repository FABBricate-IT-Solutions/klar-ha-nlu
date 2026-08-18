#!/usr/bin/env python3
"""Print the CHANGELOG.md section for a CalVer tag.

GitHub release notes must not come from `git-cliff --latest` after a squash
whose subject is `chore(release):` — cliff.toml skips those commits, so the
published body is only the heading and footer.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HEADING = re.compile(r"^## \[?v?(?P<version>[0-9][0-9.]*)\]?.*$", re.M)
RC_SUFFIX = re.compile(r"-(?:rc|staging)\.[A-Za-z0-9]+$")


def calver_base(version: str) -> str:
    return RC_SUFFIX.sub("", version.lstrip("vV"))


def section(text: str, version: str) -> str:
    version = calver_base(version)
    matches = list(HEADING.finditer(text))
    for i, match in enumerate(matches):
        if match.group("version") != version:
            continue
        end = matches[i + 1].start() if i + 1 < len(matches) else len(text)
        return text[match.start() : end].strip() + "\n"
    return ""


def commits_since(subjects: list[str], since: str) -> str:
    lines = [f"- {subject}" for subject in subjects if subject and not subject.startswith("Merge pull request")]
    if not lines:
        return ""
    return f"## Changes since `{since}`\n\n" + "\n".join(lines) + "\n"


def git_subjects(since: str) -> list[str]:
    out = subprocess.check_output(["git", "log", "--format=%s", f"{since}..HEAD"], cwd=ROOT, text=True)
    return [line for line in out.splitlines() if line.strip()]


def main(argv: list[str]) -> int:
    if len(argv) == 3 and argv[1] == "--since":
        notes = commits_since(git_subjects(argv[2]), argv[2])
        if not notes:
            print(f"no non-merge commits since {argv[2]}", file=sys.stderr)
            return 1
        sys.stdout.write(notes)
        return 0
    if len(argv) != 2 or argv[1] in {"-h", "--help"}:
        print("usage: release-notes.py VERSION | --since REF", file=sys.stderr)
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
