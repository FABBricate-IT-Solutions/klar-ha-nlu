"""Match src/parse/normalize/fold.rs so compiled tokens survive tokenize."""

from __future__ import annotations

import unicodedata

# German digraphs — same characters as Scandinavian äö, so real lexicon
# spelling ("kök") is required; ASCII "kok" will not round-trip (L-07).
_GERMAN = {
    "ä": "ae",
    "Ä": "ae",
    "ö": "oe",
    "Ö": "oe",
    "ü": "ue",
    "Ü": "ue",
    "ß": "ss",
    "æ": "ae",
    "œ": "oe",
}

_SPECIAL = {
    "ł": "l",
    "đ": "d",
    "ð": "d",
    "ø": "o",
    "ı": "i",
    "ſ": "s",
    "ŋ": "n",
    "ĸ": "k",
}

_APOS = {"\u2019", "\u2018", "\u02bc", "\u2032", "`"}
_DROP = {"\u00ad", "\u200b", "\u200c", "\u200d", "\ufeff", "\u0640"}


def fold_latin(text: str) -> str:
    out: list[str] = []
    for char in unicodedata.normalize("NFKC", text):
        for c in char.lower():
            _fold_char(c, out)
    return "".join(out)


def _fold_char(c: str, out: list[str]) -> None:
    # Unicode Nd → ASCII (L-11).
    try:
        digit = unicodedata.digit(c)
    except (TypeError, ValueError):
        digit = None
    if digit is not None:
        out.append(str(digit))
        return
    if c in _DROP:
        return
    if c in _APOS:
        out.append("'")
        return
    if c in _GERMAN:
        out.append(_GERMAN[c])
        return
    if _arabic(c):
        _fold_arabic(c, out)
        return
    if _hebrew(c):
        _fold_hebrew(c, out)
        return
    _fold_markless(c, out)


def _fold_markless(c: str, out: list[str]) -> None:
    if "\uff01" <= c <= "\uff5e":
        c = chr(ord(c) - 0xFEE0)
    if _combining(c) or c == "\u0640":
        return
    base = _strip_mark(c)
    if base is not None:
        out.append(base)
        return
    if c in _SPECIAL:
        out.append(_SPECIAL[c])
        return
    out.append(c)


def _strip_mark(c: str) -> str | None:
    parts = [ch for ch in unicodedata.normalize("NFKD", c) if not _combining(ch)]
    if len(parts) != 1 or parts[0] == c:
        return None
    return parts[0]


def _fold_arabic(c: str, out: list[str]) -> None:
    if c == "\u0640" or _arabic_mark(c):
        return
    mapped = {
        "أ": "ا",
        "إ": "ا",
        "آ": "ا",
        "ٱ": "ا",
        "ة": "ه",
        "ى": "ي",
        "ؤ": "و",
        "ئ": "ي",
    }.get(c, c)
    out.append(mapped)


def _fold_hebrew(c: str, out: list[str]) -> None:
    if _hebrew_mark(c):
        return
    out.append(c)


def _combining(c: str) -> bool:
    o = ord(c)
    return (
        0x0300 <= o <= 0x036F
        or 0x0483 <= o <= 0x0489
        or 0x1AB0 <= o <= 0x1AFF
        or 0x1DC0 <= o <= 0x1DFF
        or 0x20D0 <= o <= 0x20FF
        or 0xFE20 <= o <= 0xFE2F
    )


def _arabic(c: str) -> bool:
    o = ord(c)
    return (
        0x0600 <= o <= 0x06FF
        or 0x0750 <= o <= 0x077F
        or 0x08A0 <= o <= 0x08FF
        or 0xFB50 <= o <= 0xFDFF
        or 0xFE70 <= o <= 0xFEFF
    )


def _arabic_mark(c: str) -> bool:
    o = ord(c)
    return (
        0x0610 <= o <= 0x061A
        or 0x064B <= o <= 0x065F
        or o == 0x0670
        or 0x06D6 <= o <= 0x06ED
        or 0x08D3 <= o <= 0x08FF
    )


def _hebrew(c: str) -> bool:
    o = ord(c)
    return 0x0590 <= o <= 0x05FF or 0xFB1D <= o <= 0xFB4F


def _hebrew_mark(c: str) -> bool:
    return 0x0591 <= ord(c) <= 0x05C7
