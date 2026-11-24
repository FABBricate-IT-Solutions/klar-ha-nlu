#!/usr/bin/env python3
"""Check the links between the Markdown files of this repository.

Every relative link must point at a file that exists, and every `#anchor` at a heading
of the file it names. Links to other hosts are not fetched: this check runs on every
pull request and must not depend on the network.
"""

from __future__ import annotations

import re
import subprocess
import sys
import unicodedata
from pathlib import Path
from urllib.parse import unquote

ROOT = Path(__file__).resolve().parents[1]

# Written by git-cliff from commit subjects; old entries name files that are gone.
SKIP_NAMES = {"CHANGELOG.md"}

FENCE = re.compile(r"^\s*(```|~~~)")
INLINE_CODE = re.compile(r"`[^`\n]*`")
INLINE_LINK = re.compile(r"!?\[[^\]\n]*\]\(\s*(<[^>\n]*>|[^)\s]+)(?:\s+\"[^\"\n]*\")?\s*\)")
REFERENCE = re.compile(r"^\s{0,3}\[[^\]\n]+\]:\s*(\S+)")
HTML_TARGET = re.compile(r"""\b(?:href|src)=["']([^"']+)["']""")
HTML_ANCHOR = re.compile(r"""<a\s+(?:id|name)=["']([^"']+)["']""")
HEADING = re.compile(r"^\s{0,3}#{1,6}\s+(.*?)\s*#*\s*$")
EXTERNAL = re.compile(r"^[a-zA-Z][a-zA-Z0-9+.-]*:")


def prose_lines(text: str) -> list[tuple[int, str]]:
    """Lines outside fenced code blocks, with inline code removed."""
    out: list[tuple[int, str]] = []
    fenced = False
    for number, line in enumerate(text.splitlines(), start=1):
        if FENCE.match(line):
            fenced = not fenced
            continue
        if not fenced:
            out.append((number, INLINE_CODE.sub("", line)))
    return out


def targets(text: str) -> list[tuple[int, str]]:
    """Every link target in a Markdown text, with its line number."""
    found: list[tuple[int, str]] = []
    for number, line in prose_lines(text):
        for pattern in (INLINE_LINK, HTML_TARGET):
            found.extend((number, match.group(1).strip("<>")) for match in pattern.finditer(line))
        reference = REFERENCE.match(line)
        if reference:
            found.append((number, reference.group(1).strip("<>")))
    return found


def slug(heading: str) -> str:
    """The anchor GitHub gives a heading."""
    text = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", heading)
    text = re.sub(r"[`*_]|<[^>]+>", "", text).strip().lower()
    kept = [char for char in text if char in " -_" or unicodedata.category(char)[0] in "LNM"]
    return "".join(kept).replace(" ", "-")


def anchors(text: str) -> set[str]:
    """Every anchor a Markdown text offers: its headings and explicit `<a id>` marks."""
    seen: dict[str, int] = {}
    out: set[str] = set()
    fenced = False
    for line in text.splitlines():
        if FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        out.update(HTML_ANCHOR.findall(line))
        heading = HEADING.match(line)
        if not heading:
            continue
        base = slug(heading.group(1))
        count = seen.get(base, 0)
        seen[base] = count + 1
        out.add(base if count == 0 else f"{base}-{count}")
    return out


def check(root: Path, files: list[Path]) -> list[str]:
    """Broken links in `files` (paths relative to `root`), one message each."""
    problems: list[str] = []
    anchor_cache: dict[Path, set[str]] = {}

    def anchors_of(path: Path) -> set[str]:
        if path not in anchor_cache:
            anchor_cache[path] = anchors(path.read_text(encoding="utf-8"))
        return anchor_cache[path]

    for relative in files:
        source = root / relative
        for number, raw in targets(source.read_text(encoding="utf-8")):
            if EXTERNAL.match(raw) or raw.startswith("//"):
                continue
            path_part, _, fragment = raw.partition("#")
            path_part = unquote(path_part.split("?", 1)[0])
            if path_part.startswith("/"):
                target = root / path_part.lstrip("/")
            elif path_part:
                target = source.parent / path_part
            else:
                target = source
            where = f"{relative}:{number}"
            try:
                target.resolve().relative_to(root.resolve())
            except ValueError:
                problems.append(f"{where}: {raw} points outside the repository")
                continue
            if not target.exists():
                problems.append(f"{where}: {raw} does not exist")
                continue
            if fragment and target.suffix == ".md" and unquote(fragment).lower() not in anchors_of(target):
                problems.append(f"{where}: {raw} has no heading for #{fragment}")
    return problems


def tracked_markdown(root: Path) -> list[Path]:
    listed = subprocess.run(
        ["git", "ls-files", "-z", "--", "*.md"], cwd=root, check=True, capture_output=True, text=True
    ).stdout
    paths = [Path(item) for item in listed.split("\0") if item]
    return sorted(path for path in paths if path.name not in SKIP_NAMES)


def main() -> int:
    files = tracked_markdown(ROOT)
    problems = check(ROOT, files)
    for problem in problems:
        print(problem, file=sys.stderr)
    if problems:
        print(f"{len(problems)} broken link(s) in {len(files)} Markdown files", file=sys.stderr)
        return 1
    print(f"links ok in {len(files)} Markdown files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
