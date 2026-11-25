"""Which locales are offered without the experimental switch.

A stable locale passes its on/off minimal pairs (`tests/language_tiers.rs`): basic
commands act, never the opposite way, and neither a question nor a negated command
switches anything. The test decides: it fails for a locale listed here that does not pass
and for a locale that passes and is not listed. Every other compiled locale is
experimental; it keeps working where it is configured and is hidden from a new selection.
"""

from __future__ import annotations

STABLE = ("de", "en", "de-CH", "de-AT", "en-GB")


def tier_of(code: str) -> str:
    return "Stable" if code in STABLE else "Experimental"
