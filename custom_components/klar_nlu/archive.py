"""Checksum and member helpers for the Klar engine archive. No Home Assistant import."""

from __future__ import annotations

import hashlib
from pathlib import Path


def is_klar_binary_name(name: str) -> bool:
    path = Path(name)
    if ".." in path.parts:
        return False
    base = path.name
    return base == "klar" or base.startswith("klar-linux-")


def pick_klar_member(names: list[str]) -> str | None:
    candidates = [name for name in names if is_klar_binary_name(name)]
    for name in candidates:
        if Path(name).name == "klar":
            return name
    return candidates[0] if candidates else None


def require_sha256(digest: object, blob: bytes) -> None:
    if not isinstance(digest, str) or not digest.startswith("sha256:"):
        raise RuntimeError("Klar archive has no SHA-256 digest")
    got = hashlib.sha256(blob).hexdigest()
    if got != digest.split(":", 1)[1]:
        raise RuntimeError("Klar archive checksum mismatch")
