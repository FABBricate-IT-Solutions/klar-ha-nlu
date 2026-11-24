"""Deterministic answers for spoken date, day and time questions.

No Home Assistant import. English only: other language packs return None so
the engine and LLM path still handles them.
"""

from __future__ import annotations

import re
from datetime import datetime

_DATE = re.compile(
    r"what(?:'s| is|s)? (?:the )?(?:current )?date(?: today)?|what date is it"
    r"|tell me the date|what(?:'s| is)? today's date"
)
_DAY = re.compile(
    r"what day is it|what day of the week is it|what(?:'s| is|s)? (?:the )?day"
    r"(?: of the week)?(?: today)?|what(?:'s| is|s)? today's day|tell me what day it is"
)
_TIME = re.compile(
    r"what time is it|what(?:'s| is|s)? (?:the )?(?:current )?time"
    r"(?: right now)?|tell me the time|do you know what time it is"
)
_TODAY = re.compile(r"what(?:'s| is|s)? today")


def _normalize(text: str) -> str:
    value = str(text or "").strip().lower().replace("\u2019", "'")
    return re.sub(r"\s+", " ", value).rstrip(".!?")


def _ordinal(day: int) -> str:
    if 11 <= day % 100 <= 13:
        suffix = "th"
    else:
        suffix = {1: "st", 2: "nd", 3: "rd"}.get(day % 10, "th")
    return f"{day}{suffix}"


def _clock(stamp: datetime) -> str:
    hour = int(stamp.strftime("%I"))
    return f"{hour}:{stamp.strftime('%M')} {stamp.strftime('%p')}"


def _now() -> datetime:
    try:
        from homeassistant.util import dt as dt_util

        return dt_util.now()
    except Exception:  # noqa: BLE001 — stdlib tests load helpers without Home Assistant
        return datetime.now()


def clock_query_speech(text: str, pack: str, now: datetime | None = None) -> str | None:
    """Return the spoken answer, or None when this is not an English clock query."""
    if not str(pack or "").startswith("en"):
        return None
    query = _normalize(text)
    for pattern, kind in ((_DATE, "date"), (_DAY, "day"), (_TIME, "time"), (_TODAY, "today")):
        if pattern.fullmatch(query):
            break
    else:
        return None

    stamp = now or _now()
    date_text = f"{stamp.strftime('%A')}, {stamp.strftime('%B')} {_ordinal(stamp.day)}, {stamp.year}"
    if kind == "date":
        return f"Today is {date_text}."
    if kind == "day":
        return f"Today is {stamp.strftime('%A')}."
    if kind == "time":
        return f"It's {_clock(stamp)}."
    return f"Today is {date_text}. The time is {_clock(stamp)}."
