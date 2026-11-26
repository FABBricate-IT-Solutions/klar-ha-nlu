"""ADR 0003: no utterance-matching regexes in the Home Assistant integration.

docs/architecture/adr-0003-python-rust-boundary.md says Assist product logic, and with it
the reading of what a person said, belongs to the Rust engine. `custom_components/klar_nlu/`
is platform glue. The cheapest place to leak that logic back is a `re.compile` in Python, so
every call into `re` (or `regex`) in the integration must be on a short allowlist with a kind
and a reason. The allowance is a ceiling that can only go down: a new site fails, and so does
an allowance larger than the sites that remain.
"""

from __future__ import annotations

import ast
from pathlib import Path
from typing import Any

from .config import ROOT, GuardrailError, require_text

REGEX_MODULES = frozenset({"re", "regex"})
REGEX_CALLS = frozenset({"compile", "match", "search", "fullmatch", "sub", "subn", "findall", "finditer", "split"})
# What a regex in glue may be for. `debt` is utterance matching that still has to move.
KINDS = frozenset({"platform", "markup", "output", "token", "debt"})
ADR = "docs/architecture/adr-0003-python-rust-boundary.md"


def regex_sites(source: str) -> list[int]:
    """Line numbers of every call into `re`/`regex`, however the module was imported."""
    tree = ast.parse(source)
    modules: set[str] = set()
    functions: set[str] = set()
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            modules.update((alias.asname or alias.name) for alias in node.names if alias.name in REGEX_MODULES)
        elif isinstance(node, ast.ImportFrom) and node.module in REGEX_MODULES:
            functions.update((alias.asname or alias.name) for alias in node.names if alias.name in REGEX_CALLS)
    lines = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        func = node.func
        if isinstance(func, ast.Attribute) and func.attr in REGEX_CALLS:
            if isinstance(func.value, ast.Name) and func.value.id in modules:
                lines.append(node.lineno)
        elif isinstance(func, ast.Name) and func.id in functions:
            lines.append(node.lineno)
    return sorted(lines)


def check(config: dict[str, Any], root: Path = ROOT) -> list[str]:
    section = config["boundary"]
    base = root / section["root"]
    allowed: dict[str, dict[str, Any]] = {}
    problems: list[str] = []
    for entry in section.get("allow", []):
        name = require_text(entry, "file", "boundary.allow")
        require_text(entry, "reason", f"boundary.allow {name}")
        if entry.get("kind") not in KINDS:
            raise GuardrailError(f"boundary.allow {name}: `kind` must be one of {sorted(KINDS)}")
        if not isinstance(entry.get("sites"), int) or entry["sites"] < 1:
            raise GuardrailError(f"boundary.allow {name}: `sites` must be a positive integer")
        allowed[name] = entry
    for path in sorted(base.glob("*.py")):
        sites = regex_sites(path.read_text(encoding="utf-8"))
        name = path.name
        rel = path.relative_to(root).as_posix()
        entry = allowed.pop(name, None)
        if sites and entry is None:
            problems.append(
                f"{rel}:{sites[0]} calls the regex module ({len(sites)} site(s)). ADR 0003 ({ADR}): utterance "
                "matching and Assist product rules belong in the Rust engine (src/parse, src/lang/packs), not "
                "in Python regexes. Move the matcher into the engine. Only if this is platform glue that never "
                "reads what a person said, add a [[boundary.allow]] entry (kind and reason) to "
                "scripts/guardrails.toml."
            )
        elif entry is not None and len(sites) > entry["sites"]:
            problems.append(
                f"{rel} has {len(sites)} regex sites, the ADR 0003 allowance is {entry['sites']} "
                f"(lines {', '.join(map(str, sites))}). New matching belongs in the engine; do not raise the allowance."
            )
        elif entry is not None and len(sites) < entry["sites"]:
            problems.append(
                f"{rel} has {len(sites)} regex site(s), the allowance in scripts/guardrails.toml says "
                f"{entry['sites']}. Lower it to {len(sites)} (or drop the entry at 0) so the room stays closed."
            )
    problems.extend(
        f"scripts/guardrails.toml allows regexes in {name}, which does not exist in {section['root']}. Remove the entry."
        for name in sorted(allowed)
    )
    return problems
