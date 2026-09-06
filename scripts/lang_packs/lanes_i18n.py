"""Wizard chrome overlay for every compiled Assist locale."""

from __future__ import annotations

from lang_packs.lanes_east import PACKS as EAST
from lang_packs.lanes_script import PACKS as SCRIPT
from lang_packs.lanes_west import PACKS as WEST

PACKS: dict[str, dict[str, str]] = {}
PACKS.update(WEST)
PACKS.update(EAST)
PACKS.update(SCRIPT)
