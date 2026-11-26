"""Load scripts/guardrails.toml, the one place the guardrails keep their data."""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[2]
CONFIG = ROOT / "scripts" / "guardrails.toml"


class GuardrailError(Exception):
    """A guardrail failed. The message is what the contributor sees; it says how to fix it."""


def load(path: Path = CONFIG) -> dict[str, Any]:
    with path.open("rb") as handle:
        return tomllib.load(handle)


def require_text(entry: dict[str, Any], key: str, where: str) -> str:
    value = entry.get(key)
    if not isinstance(value, str) or not value.strip():
        raise GuardrailError(f"{where}: `{key}` must be a non-empty string in scripts/guardrails.toml")
    return value.strip()
