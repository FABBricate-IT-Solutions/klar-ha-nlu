"""Focus: freeze the non-core areas and measure how much fixing still goes there.

Klar is a deterministic Home Assistant NLU. The news briefing, Wikipedia answers, the
personality few-shots and the custom-voice interview are not that core. They stay in the code
with their defaults off, and they are frozen: a `fix` or `feat` commit whose changes sit mostly in
frozen paths has to say why in a `Frozen-area:` trailer. The same
classification gives the share of fixes that went to non-core paths, which the acceptance of
package 6.11 keeps under 5 %.
"""

from __future__ import annotations

import os
import re
import subprocess
from dataclasses import dataclass
from fnmatch import fnmatchcase
from pathlib import Path
from typing import Any

from .config import ROOT, GuardrailError, require_text

SUBJECT = re.compile(r"^(?P<type>[a-z]+)(?:\([^)]*\))?!?:")
CONTROLLED_TYPES = frozenset({"fix", "feat"})
RECORD, FIELD = "\x1e", "\x1f"


@dataclass(frozen=True)
class Area:
    name: str
    frozen: bool
    paths: tuple[str, ...]
    default_off: tuple[str, ...]


@dataclass(frozen=True)
class Commit:
    sha: str
    subject: str
    body: str
    files: tuple[str, ...]

    @property
    def kind(self) -> str:
        match = SUBJECT.match(self.subject)
        return match.group("type") if match else ""


def areas(config: dict[str, Any]) -> list[Area]:
    result = []
    for entry in config["focus"]["area"]:
        name = require_text(entry, "name", "focus.area")
        paths = tuple(entry.get("paths", ()))
        if not paths:
            raise GuardrailError(f"focus.area {name}: `paths` is empty")
        result.append(Area(name, bool(entry.get("frozen")), paths, tuple(entry.get("default_off", ()))))
    return result


def matches(path: str, area: Area) -> bool:
    return any(fnmatchcase(path, pattern) for pattern in area.paths)


def classify(commit: Commit, config: dict[str, Any]) -> tuple[bool, bool]:
    """(mostly_non_core, touches_frozen). A commit is mostly non-core when more than half of its
    files (docs and changelogs left out) sit in a non-core area, so a fix to the news briefing
    cannot hide behind one unrelated file. Measured on history since 2026-11-01 this is 7 % of
    fixes; "only" non-core files would say 0.7 % and "any" 16 %."""
    ignored = config["focus"]["ignored"]
    relevant = [name for name in commit.files if not any(fnmatchcase(name, pattern) for pattern in ignored)]
    found = [{area.name for area in areas(config) if matches(name, area)} for name in relevant]
    frozen = {area.name for area in areas(config) if area.frozen}
    mostly = 2 * sum(1 for names in found if names) > len(relevant)
    return mostly, any(names & frozen for names in found)


def acknowledged(commit: Commit, config: dict[str, Any]) -> bool:
    section = config["focus"]
    pattern = re.compile(rf"^{re.escape(section['ack_trailer'])}:\s*({'|'.join(map(re.escape, section['ack_reasons']))})\b", re.M)
    return bool(pattern.search(commit.body))


def git_commits(rev_range: list[str], root: Path = ROOT) -> list[Commit]:
    fmt = f"{RECORD}%H{FIELD}%s{FIELD}%b{FIELD}"
    out = subprocess.run(
        ["git", "log", "--no-merges", "--name-only", f"--format={fmt}", *rev_range],
        cwd=root,
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    commits = []
    for record in out.split(RECORD)[1:]:
        sha, subject, body, names = record.split(FIELD, 3)
        commits.append(Commit(sha, subject, body, tuple(line for line in names.splitlines() if line.strip())))
    return commits


def repo_files(root: Path) -> list[str]:
    """Every file under root, relative, without VCS, build and dependency directories."""
    skip = {".git", "node_modules", "target", "dist", "__pycache__"}
    names = []
    for current, dirs, files in os.walk(root):
        dirs[:] = [name for name in dirs if name not in skip]
        names.extend((Path(current) / name).relative_to(root).as_posix() for name in files)
    return names


def check_static(config: dict[str, Any], root: Path = ROOT) -> list[str]:
    """The freeze is real: every frozen path exists, is owned, and its default is off."""
    problems: list[str] = []
    settings = (root / config["focus"]["settings"]).read_text(encoding="utf-8")
    codeowners = (root / ".github" / "CODEOWNERS").read_text(encoding="utf-8")
    tracked = repo_files(root)
    for area in areas(config):
        for pattern in area.paths:
            if not any(fnmatchcase(name, pattern) for name in tracked):
                problems.append(f"focus area {area.name}: `{pattern}` matches no file. Update scripts/guardrails.toml.")
            if area.frozen and f"/{pattern}" not in codeowners:
                problems.append(
                    f"focus area {area.name}: `/{pattern}` is not in .github/CODEOWNERS. Frozen paths need an owner line."
                )
        for field in area.default_off:
            if not re.search(rf"\b{re.escape(field)}: false,", settings):
                problems.append(
                    f"focus area {area.name} is frozen with `{field}` off by default, but {config['focus']['settings']} "
                    f"no longer sets `{field}: false` in its default. The frozen areas ship off."
                )
    return problems


def check_commits(commits: list[Commit], config: dict[str, Any]) -> list[str]:
    section = config["focus"]
    problems = []
    for commit in commits:
        if commit.kind not in CONTROLLED_TYPES:
            continue
        mostly, touches_frozen = classify(commit, config)
        if mostly and touches_frozen and not acknowledged(commit, config):
            problems.append(
                f"{commit.sha[:9]} \"{commit.subject}\" mostly changes frozen non-core paths (news briefing, Wikipedia "
                "answers, personality few-shots, custom-voice interview). Those areas are frozen: put the effort into "
                f"the deterministic core instead, or add a `{section['ack_trailer']}: <reason>` trailer to the commit "
                f"message with one of {', '.join(section['ack_reasons'])}."
            )
    return problems


def share(commits: list[Commit], config: dict[str, Any]) -> tuple[int, int]:
    """(fixes that went mostly to news/LLM paths, all fixes)."""
    fixes = [commit for commit in commits if commit.kind == "fix"]
    return sum(1 for commit in fixes if classify(commit, config)[0]), len(fixes)


def share_report(commits: list[Commit], config: dict[str, Any]) -> tuple[float, str]:
    non_core, total = share(commits, config)
    percent = 100.0 * non_core / total if total else 0.0
    limit = config["focus"]["max_share_percent"]
    return percent, f"non-core fix share: {non_core} of {total} fix commits = {percent:.1f} % (limit {limit} %)"
