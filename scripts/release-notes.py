#!/usr/bin/env python3
"""Print the CHANGELOG.md section for a CalVer tag.

GitHub release notes must not come from `git-cliff --latest` after a squash
whose subject is `chore(release):` — cliff.toml skips those commits, so the
published body is only the heading and footer.

HACS shows this GitHub body in the update dialog before install. Supervisor
shows `addon/CHANGELOG.md` and `addon-staging/CHANGELOG.md`. Breaking items
are hoisted to the top so they are visible before the install button.
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ADDON_CHANGELOGS = (
    ROOT / "addon" / "CHANGELOG.md",
    ROOT / "addon-staging" / "CHANGELOG.md",
)
HEADING = re.compile(r"^## \[?v?(?P<version>[0-9][0-9.]*)\]?.*$", re.M)
RC_SUFFIX = re.compile(r"-(?:rc|staging)\.[A-Za-z0-9]+$")
BREAKING_MARK = re.compile(r"\[\*\*breaking\*\*\]", re.I)
BANG_SUBJECT = re.compile(r"^(?:feat|fix|refactor|perf|chore)(?:\([^)]+\))?!:", re.I)
BREAKING_FOOTER = re.compile(r"BREAKING CHANGE", re.I)
BREAKING_GROUP = re.compile(r"^### Breaking Changes\s*$", re.I)
GROUP_HEADING = re.compile(r"^### ")
VERSION_HEADING = re.compile(r"^## ")


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


def is_breaking_line(line: str) -> bool:
    text = line.strip()
    if not text.startswith("-"):
        return False
    body = text.lstrip("-").strip()
    return bool(BREAKING_MARK.search(body) or BANG_SUBJECT.match(body) or BREAKING_FOOTER.search(body))


def breaking_items(text: str) -> list[str]:
    items: list[str] = []
    in_breaking_group = False
    for line in text.splitlines():
        if GROUP_HEADING.match(line):
            in_breaking_group = bool(BREAKING_GROUP.match(line))
            continue
        if VERSION_HEADING.match(line):
            in_breaking_group = False
            continue
        stripped = line.strip()
        if not stripped.startswith("-"):
            continue
        if in_breaking_group or is_breaking_line(stripped):
            items.append(stripped)
    return items


def hoist_breaking(notes: str) -> str:
    notes = notes.lstrip("\n")
    if notes.startswith("## Breaking Changes"):
        return notes
    items = breaking_items(notes)
    if not items:
        return notes
    unique = list(dict.fromkeys(items))
    banner = (
        "## Breaking Changes\n\n"
        "Read these before you install. They change behavior or require action.\n\n"
        + "\n".join(unique)
        + "\n\n"
    )
    return banner + notes


def commits_since(subjects: list[str], since: str) -> str:
    lines = [f"- {subject}" for subject in subjects if subject and not subject.startswith("Merge pull request")]
    if not lines:
        return ""
    return f"## Changes since `{since}`\n\n" + "\n".join(lines) + "\n"


def git_subjects(since: str) -> list[str]:
    out = subprocess.check_output(["git", "log", "--format=%s", f"{since}..HEAD"], cwd=ROOT, text=True)
    return [line for line in out.splitlines() if line.strip()]


def sync_addon_changelogs() -> None:
    """Supervisor reads CHANGELOG.md next to each add-on config.yaml."""
    source = ROOT / "CHANGELOG.md"
    text = source.read_text(encoding="utf-8")
    if not text.startswith("# Changelog"):
        raise SystemExit("CHANGELOG.md is missing the Keep a Changelog header")
    for path in ADDON_CHANGELOGS:
        path.write_text(text, encoding="utf-8")


def main(argv: list[str]) -> int:
    if len(argv) == 2 and argv[1] == "--sync-addons":
        sync_addon_changelogs()
        return 0
    if len(argv) == 3 and argv[1] == "--since":
        notes = hoist_breaking(commits_since(git_subjects(argv[2]), argv[2]))
        if not notes.strip():
            print(f"no non-merge commits since {argv[2]}", file=sys.stderr)
            return 1
        sys.stdout.write(notes)
        return 0
    if len(argv) != 2 or argv[1] in {"-h", "--help"}:
        print("usage: release-notes.py VERSION | --since REF | --sync-addons", file=sys.stderr)
        return 2
    version = argv[1].lstrip("vV")
    notes = hoist_breaking(section((ROOT / "CHANGELOG.md").read_text(encoding="utf-8"), version))
    if not notes or not re.search(r"^### ", notes, re.M):
        print(f"CHANGELOG.md has no grouped notes for {version}", file=sys.stderr)
        return 1
    sys.stdout.write(notes)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
