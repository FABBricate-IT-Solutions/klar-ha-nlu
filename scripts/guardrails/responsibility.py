"""Module responsibility instead of a line count.

A flat line limit made 22 files stop at 480-499 lines and produced commits whose only job was
to shuffle lines under the number. The rule here asks the question that matters: what is this
module for, and who owns it. A hand-written source file longer than `review_lines` has to be
listed in scripts/guardrails.toml with an owner and the one job it does. Nothing happens at
499 versus 501 other than that sentence. Only a logic module past `ceiling_lines` fails, and
the message asks for a split along the responsibilities the entry names. A `table` (data that
is long by nature) is exempt from the ceiling but still needs its entry.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .config import ROOT, GuardrailError, require_text

OWNERS = frozenset({"engine", "ha-glue", "operator-ui", "language-data", "tooling"})
KINDS = frozenset({"logic", "table"})
GENERATED = "@generated"
SOURCE_GLOBS = ("src/**/*.rs", "custom_components/klar_nlu/*.py", "scripts/**/*.py", "web/src/**/*.ts", "web/src/**/*.tsx")


def is_test_file(rel: str) -> bool:
    name = rel.rsplit("/", 1)[-1]
    return "/tests/" in f"/{rel}" or name in {"tests.rs", "tests.py"} or name.endswith("_tests.rs") or name.startswith("test_")


def is_generated(text: str) -> bool:
    return GENERATED in "\n".join(text.splitlines()[:5])


def line_count(text: str) -> int:
    return len(text.splitlines())


def sources(root: Path = ROOT) -> list[Path]:
    found: set[Path] = set()
    for pattern in SOURCE_GLOBS:
        found.update(path for path in root.glob(pattern) if path.is_file())
    return sorted(path for path in found if "node_modules" not in path.parts)


def check(config: dict[str, Any], root: Path = ROOT) -> list[str]:
    section = config["responsibility"]
    review, ceiling = section["review_lines"], section["ceiling_lines"]
    listed: dict[str, dict[str, Any]] = {}
    for entry in section.get("module", []):
        name = require_text(entry, "path", "responsibility.module")
        require_text(entry, "job", f"responsibility.module {name}")
        if entry.get("owner") not in OWNERS:
            raise GuardrailError(f"responsibility.module {name}: `owner` must be one of {sorted(OWNERS)}")
        if entry.get("kind") not in KINDS:
            raise GuardrailError(f"responsibility.module {name}: `kind` must be one of {sorted(KINDS)}")
        listed[name] = entry
    problems: list[str] = []
    for path in sources(root):
        rel = path.relative_to(root).as_posix()
        if is_test_file(rel):
            continue
        text = path.read_text(encoding="utf-8")
        lines = line_count(text)
        if lines <= review or is_generated(text):
            listed.pop(rel, None)
            continue
        entry = listed.pop(rel, None)
        if entry is None:
            problems.append(
                f"{rel} has {lines} lines and no entry in scripts/guardrails.toml. Add a [[responsibility.module]] "
                "with its owner and the one job it does. Do not split the file just to get under a number; split "
                "it when the entry would need the word \"and\" for a second, unrelated job."
            )
        elif entry["kind"] == "logic" and lines > ceiling:
            problems.append(
                f"{rel} has {lines} lines, past the {ceiling}-line ceiling for logic modules. Its entry says: "
                f"\"{entry['job']}\". Split along the separate responsibilities inside it, or, if it is data "
                "that is long by nature, set kind = \"table\"."
            )
    problems.extend(
        f"scripts/guardrails.toml lists {name}, which does not exist. Remove or move the entry."
        for name in sorted(listed)
        if not (root / name).is_file()
    )
    return problems
