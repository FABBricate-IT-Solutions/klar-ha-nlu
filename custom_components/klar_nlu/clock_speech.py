"""Strip seconds from engine-marked clock answers. Never rewrite the time or language."""

from __future__ import annotations

import re
from datetime import datetime

_CLOCK_TIME = re.compile(r"\b(\d{1,2}):(\d{2})(?::(\d{2}))?\b")


def strip_clock_seconds(speech: str) -> str:
    return _CLOCK_TIME.sub(lambda match: f"{int(match[1]):02d}:{match[2]}", speech)


def finish_clock_speech(speech: str, pack: str, now: datetime | None = None) -> str:
    """Keep the engine's wording; only drop seconds from HH:MM:SS."""
    del pack, now
    if not _CLOCK_TIME.search(speech or ""):
        return speech
    return strip_clock_seconds(speech).strip()
