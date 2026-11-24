"""Words that answer an open question in a generated pack.

Yes is the pack's own yes list. No is the word the yes/no chips already carry for
that locale. Cancel phrases and the wording of the confirmation question are not
derived: a list nobody checked stays empty and the open question expires instead.
"""

from __future__ import annotations

from lang_packs.fold import fold_latin
from lang_packs.lotse_yesno import YESNO


def answers_for(code: str, words: dict) -> dict:
    chips = YESNO.get(code) or YESNO.get(code.split("-")[0])
    no = fold_latin(chips[1]).strip() if chips else ""
    yes = [word for word in words.get("yes", []) if word]
    return {"yes": yes, "no": [no] if no and no not in yes else [], "cancel": []}
