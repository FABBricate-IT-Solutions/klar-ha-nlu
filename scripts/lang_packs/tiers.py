"""How far each compiled locale is tested (package 4.3).

Reference: the locale passes the full on/off/negation minimal pairs
(`tests/language_tiers.rs`): every basic command acts the right way round, and a
question or a negated command never switches anything. Every compiled locale
earns it, so every compiled locale is listed here.

Beta: the gold natural-sentence gate and no opposite-action failures, without
the negation check. Empty today.

Experimental: a locale that is new or does not earn Reference yet. A locale not
listed here is Experimental: offered for a new selection only with
`Settings::experimental_languages`. A locale that is configured already
keeps working either way.
"""

from __future__ import annotations

REFERENCE = (
    "de", "en", "fr", "nl", "es", "it", "pt", "ca",
    "ro", "da", "nb", "sv", "fi", "de-CH", "de-AT", "en-GB",
    "pt-BR", "af", "cs", "sk", "pl", "hu", "hr", "sl",
    "bg", "el", "sr", "sr-Latn", "uk", "zh-CN", "zh-TW", "zh-HK",
    "ar", "he", "fa", "ur", "tr", "th", "ko", "ja",
    "cy", "et", "eu", "ga", "gl", "is", "lb", "kw",
    "lt", "lv", "id", "ms", "sw", "vi", "hi", "bn",
    "gu", "kn", "ml", "mr", "ta", "te", "pa", "ne",
    "hy", "ka", "mn",
)
BETA: tuple[str, ...] = ()


def tier_of(code: str) -> str:
    if code in REFERENCE:
        return "Reference"
    if code in BETA:
        return "Beta"
    return "Experimental"
