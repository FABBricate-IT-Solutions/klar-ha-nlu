"""Clock replies: one sentence, local HH:MM, never seconds."""

from __future__ import annotations

import re
from datetime import datetime

_CLOCK_TIME = re.compile(r"\b(\d{1,2}):(\d{2})(?::(\d{2}))?\b")
_CLOCK_LINE = re.compile(r"(es ist|it is|die uhrzeit)\b", re.IGNORECASE)


def strip_clock_seconds(speech: str) -> str:
    return _CLOCK_TIME.sub(lambda match: f"{int(match[1]):02d}:{match[2]}", speech)


def finish_clock_speech(speech: str, pack: str, now: datetime | None = None) -> str:
    if not _CLOCK_TIME.search(speech or ""):
        return speech
    text = strip_clock_seconds(speech).strip()
    clockish = bool(_CLOCK_LINE.search(text)) or (len(text) < 48 and text.count(":") == 1)
    if not clockish:
        return text
    stamp = now
    if stamp is None:
        try:
            from homeassistant.util import dt as dt_util

            stamp = dt_util.now()
        except Exception:  # noqa: BLE001 — stdlib tests have no Home Assistant
            stamp = datetime.now()
    hhmm = f"{stamp.hour:02d}:{stamp.minute:02d}"
    text = _CLOCK_TIME.sub(hhmm, text, count=1)
    for sep in (". ", "! ", "? "):
        if sep in text:
            text = text.split(sep, 1)[0].strip()
            break
    text = text.rstrip(".!?") + "."
    if pack == "de" and _CLOCK_LINE.search(text):
        return f"Es ist {hhmm}."
    if pack != "de" and _CLOCK_LINE.search(text):
        return f"It is {hhmm}."
    return text
