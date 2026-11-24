"""Convert spoken number words to digits in reminder phrases.

Klar's reminder parser slots digit durations but rejects spelled-out ones.
Home Assistant's default agent accepts both, so normalization is scoped to
reminder-shaped text and passed only to Klar's parse call. Other voice
commands keep their original wording.
"""

from __future__ import annotations

import re

_ONES = {
    "zero": 0,
    "one": 1,
    "two": 2,
    "three": 3,
    "four": 4,
    "five": 5,
    "six": 6,
    "seven": 7,
    "eight": 8,
    "nine": 9,
    "ten": 10,
    "eleven": 11,
    "twelve": 12,
    "thirteen": 13,
    "fourteen": 14,
    "fifteen": 15,
    "sixteen": 16,
    "seventeen": 17,
    "eighteen": 18,
    "nineteen": 19,
}

_TENS = {
    "twenty": 20,
    "thirty": 30,
    "forty": 40,
    "fifty": 50,
    "sixty": 60,
    "seventy": 70,
    "eighty": 80,
    "ninety": 90,
}

_SINGLES = "|".join(sorted(_ONES, key=len, reverse=True))
_TENS_WORDS = "|".join(sorted(_TENS, key=len, reverse=True))
_NUMBER_RE = re.compile(
    rf"\b(?:(?P<tens>{_TENS_WORDS})(?:[\s-]+(?P<unit>one|two|three|four|five|six|seven|eight|nine))?|(?P<single>{_SINGLES}))\b",
    re.IGNORECASE,
)
_REMINDER_RE = re.compile(r"\b(remind(?:er|ers)?|erinner\w*)\b", re.IGNORECASE)


def _replace(match: re.Match[str]) -> str:
    tens = match.group("tens")
    if tens:
        unit = match.group("unit")
        value = _TENS[tens.lower()]
        if unit:
            value += _ONES[unit.lower()]
        return str(value)
    return str(_ONES[match.group("single").lower()])


def normalize_number_words(text: str) -> str:
    """Return text with number words replaced by digits, for reminders only."""
    if not isinstance(text, str) or not _REMINDER_RE.search(text):
        return text
    return _NUMBER_RE.sub(_replace, text)
