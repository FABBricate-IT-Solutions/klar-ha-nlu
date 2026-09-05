"""Operator UI chrome for every compiled Assist locale."""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from lang_packs.web_ui_asia import PACKS as ASIA
from lang_packs.web_ui_europe import PACKS as EUROPE
from lang_packs.web_ui_indic import PACKS as INDIC
from lang_packs.web_ui_keys import ALIASES, CATALOG_KEYS, CONSTANTS, expand
from lang_packs.web_ui_mena import PACKS as MENA
from lang_packs.web_ui_nordic import PACKS as NORDIC
from lang_packs.lanes import apply_lanes_copy
from lang_packs.lanes_i18n import PACKS as LANES
from lang_packs.web_ui_settings import apply_settings_copy
from lang_packs.web_ui_slavic import PACKS as SLAVIC
from lang_packs.web_ui_west import PACKS as WEST

ROOT = Path(__file__).resolve().parents[2]
EN_TS = ROOT / "web" / "src" / "i18n" / "en.ts"
DEST = ROOT / "web" / "src" / "i18n" / "messages"
HAND = {"de", "en"}

PACKS: dict[str, dict[str, str]] = {}
PACKS.update(EUROPE)
PACKS.update(NORDIC)
PACKS.update(WEST)
PACKS.update(SLAVIC)
PACKS.update(ASIA)
PACKS.update(MENA)
PACKS.update(INDIC)

_LAB_HINT = {
    "de": "Das Labor ist der Assist-Pfad für die gewählte Sprache. Entscheidung und Intents hier führt Klar aus. Satztrigger nur, wenn Klar nicht erreichbar ist.",
    "de-AT": "Das Labor ist der Assist-Pfad für die gewählte Sprache. Entscheidung und Intents hier führt Klar aus. Satztrigger nur, wenn Klar nicht erreichbar ist.",
    "de-CH": "S Labor isch de Assist-Pfad für d gwählt Sproch. Entscheid und Intents da führt Klar us. Satztrigger nur, wenn Klar nöd erreichbar isch.",
}
_LAB_PATH = {
    "de": "Klar-Parse, dann dieser Pfad. Assist startet keinen anderen Intent, keinen Satztrigger und keinen Wetter-Fallback.",
    "de-AT": "Klar-Parse, dann dieser Pfad. Assist startet keinen anderen Intent, keinen Satztrigger und keinen Wetter-Fallback.",
    "de-CH": "Klar-Parse, denn de Pfad. Assist startet kein andere Intent, kein Satztrigger und kein Wetter-Fallback.",
}
_LAB_HINT_EN = "Lab is the Assist path for the selected language. Decision and intents here are what Klar runs. Sentence triggers run only if Klar is unreachable."
_LAB_PATH_EN = "Klar parse, then that path. Assist does not run a different intent, a sentence trigger, or a weather fallback."
for _code, _fields in PACKS.items():
    _fields["parseHint"] = _LAB_HINT.get(_code, _LAB_HINT_EN)
    _fields["triggerFirst"] = _LAB_PATH.get(_code, _LAB_PATH_EN)
apply_settings_copy(PACKS)
apply_lanes_copy(PACKS, LANES)


def english_keys() -> list[str]:
    return re.findall(r"^\s+(\w+):", EN_TS.read_text(encoding="utf-8"), re.M)


def write_web_ui_translations() -> None:
    keys = english_keys()
    expected = set(CATALOG_KEYS) | set(ALIASES) | set(CONSTANTS)
    if set(keys) != expected:
        raise SystemExit(f"en.ts drift missing={sorted(expected - set(keys))} extra={sorted(set(keys) - expected)}")
    DEST.mkdir(parents=True, exist_ok=True)
    written: set[str] = set()
    for code, fields in PACKS.items():
        if code in HAND:
            raise SystemExit(f"{code} is hand-written TypeScript, do not emit JSON")
        missing = [key for key in CATALOG_KEYS if key not in fields]
        extra = [key for key in fields if key not in CATALOG_KEYS]
        if missing or extra:
            raise SystemExit(f"{code}: missing={missing} extra={extra}")
        flat = expand(fields)
        if set(flat) != set(keys):
            raise SystemExit(f"{code}: expand drift {sorted(set(keys) ^ set(flat))}")
        path = DEST / f"{code}.json"
        path.write_text(json.dumps(flat, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        written.add(code)
        print("wrote", path.relative_to(ROOT))
    stale = {item.stem for item in DEST.glob("*.json")} - written
    if stale:
        raise SystemExit(f"stale operator UI files: {sorted(stale)}")


if __name__ == "__main__":
    write_web_ui_translations()
