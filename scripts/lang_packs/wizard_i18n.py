"""Emit first-run wizard JSON for every compiled Assist locale."""

from __future__ import annotations

import json
from pathlib import Path

from lang_packs.wizard import DEST, EN_CONSOLE, EN_LLM, KEYS
from lang_packs.wizard_east import PACKS as EAST
from lang_packs.wizard_script import PACKS as SCRIPT
from lang_packs.wizard_west import PACKS as WEST

PACKS: dict[str, dict[str, str]] = {}
PACKS.update(WEST)
PACKS.update(EAST)
PACKS.update(SCRIPT)

EXPECTED = {
    "fr", "nl", "es", "it", "pt", "ca", "ro", "pt-BR", "gl",
    "de-CH", "de-AT", "en-GB", "af", "lb", "cy", "eu", "ga", "kw",
    "da", "nb", "sv", "fi", "is", "et", "lt", "lv",
    "cs", "sk", "pl", "hu", "hr", "sl", "bg", "sr", "sr-Latn", "uk",
    "zh-CN", "zh-TW", "zh-HK", "ja", "ko", "th", "vi", "id", "ms", "mn",
    "ar", "he", "fa", "ur", "tr", "el", "hy", "ka",
    "hi", "bn", "gu", "kn", "ml", "mr", "ta", "te", "pa", "ne", "sw",
}


def write_wizard_translations() -> None:
    if set(PACKS) != EXPECTED:
        raise SystemExit(f"wizard locale drift missing={sorted(EXPECTED - set(PACKS))} extra={sorted(set(PACKS) - EXPECTED)}")
    DEST.mkdir(parents=True, exist_ok=True)
    written: set[str] = set()
    for code, fields in PACKS.items():
        absent = [key for key in KEYS if key not in fields]
        if absent:
            raise SystemExit(f"{code}: wizard missing {absent}")
        if code != "en-GB":
            if fields["whatConsole"] == EN_CONSOLE:
                raise SystemExit(f"{code}: whatConsole still English")
            if fields["missLlmBody"] == EN_LLM:
                raise SystemExit(f"{code}: missLlmBody still English")
        if "{count}" not in fields["phrasesMapping"]:
            raise SystemExit(f"{code}: phrasesMapping needs {{count}}")
        path = DEST / f"{code}.json"
        path.write_text(json.dumps(fields, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        written.add(code)
        print("wrote", path.relative_to(DEST.parents[2]))
    stale = {item.stem for item in DEST.glob("*.json")} - written
    if stale:
        raise SystemExit(f"stale wizard files: {sorted(stale)}")


if __name__ == "__main__":
    write_wizard_translations()
