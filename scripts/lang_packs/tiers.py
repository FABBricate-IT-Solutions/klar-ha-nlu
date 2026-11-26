"""How far each compiled locale is tested (package 4.3).

Reference: hand-written de/en and their regional packs that pass the full
on/off/negation minimal pairs (`tests/language_tiers.rs`).

Beta: locales with a gold natural-sentence gate and no opposite-action
failures. Missing negation words alone does not keep them experimental.

Experimental: everything else. Offered for a new selection only with
`Settings::experimental_languages`. A locale that is configured already
keeps working either way.
"""

from __future__ import annotations

REFERENCE = ("de", "en", "de-CH", "de-AT", "en-GB")
BETA = ("es", "nl", "pt", "pt-BR", "it", "fr", "id", "uk", "hi")


def tier_of(code: str) -> str:
    if code in REFERENCE:
        return "Reference"
    if code in BETA:
        return "Beta"
    return "Experimental"
