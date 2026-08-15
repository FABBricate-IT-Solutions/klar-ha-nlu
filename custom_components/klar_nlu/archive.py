"""Checksum helpers for the Klar engine archive. No Home Assistant import."""

from __future__ import annotations

import hashlib


def require_sha256(digest: object, blob: bytes) -> None:
    if not isinstance(digest, str) or not digest.startswith("sha256:"):
        raise RuntimeError("Klar archive has no SHA-256 digest")
    got = hashlib.sha256(blob).hexdigest()
    if got != digest.split(":", 1)[1]:
        raise RuntimeError("Klar archive checksum mismatch")
