"""Settings / LLM / trainer chrome for every compiled Assist locale."""

from __future__ import annotations

from lang_packs.settings_east import PACKS as EAST
from lang_packs.settings_hints_asia import PACKS as HINTS_ASIA
from lang_packs.settings_hints_indic import PACKS as HINTS_INDIC
from lang_packs.settings_hints_north import PACKS as HINTS_NORTH
from lang_packs.settings_hints_west import PACKS as HINTS_WEST
from lang_packs.settings_script import PACKS as SCRIPT
from lang_packs.settings_west import PACKS as WEST

PACKS: dict[str, dict[str, str]] = {}
PACKS.update(WEST)
PACKS.update(EAST)
PACKS.update(SCRIPT)

HINTS = {**HINTS_WEST, **HINTS_NORTH, **HINTS_ASIA, **HINTS_INDIC}
if set(HINTS) != set(PACKS):
    raise SystemExit(
        f"settings hint locale drift missing={sorted(set(PACKS) - set(HINTS))} extra={sorted(set(HINTS) - set(PACKS))}"
    )
for code, extra in HINTS.items():
    PACKS[code].update(extra)

LOCALES = [(code, fields, {}) for code, fields in PACKS.items()]
