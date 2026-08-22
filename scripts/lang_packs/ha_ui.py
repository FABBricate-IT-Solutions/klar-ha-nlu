"""Home Assistant UI strings for every compiled Assist locale."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from lang_packs.ha_ui_catalog import HA_FILE, PACKS, expand

ROOT = Path(__file__).resolve().parents[2]
STRINGS = ROOT / "custom_components" / "klar_nlu" / "strings.json"
DEST = ROOT / "custom_components" / "klar_nlu" / "translations"


def flatten(obj: object, prefix: str = "") -> dict[str, str]:
    if isinstance(obj, dict):
        out: dict[str, str] = {}
        for key, value in obj.items():
            path = f"{prefix}.{key}" if prefix else str(key)
            out.update(flatten(value, path))
        return out
    return {prefix: str(obj)}


def nest(flat: dict[str, str]) -> dict:
    out: dict = {}
    for path, value in flat.items():
        cursor = out
        parts = path.split(".")
        for part in parts[:-1]:
            cursor = cursor.setdefault(part, {})
        cursor[parts[-1]] = value
    return out


def english_keys() -> list[str]:
    return list(flatten(json.loads(STRINGS.read_text(encoding="utf-8"))))


def write_ha_translations() -> None:
    keys = english_keys()
    DEST.mkdir(parents=True, exist_ok=True)
    written: set[str] = set()
    for code, fields in PACKS.items():
        name = HA_FILE.get(code, code)
        if name is None:
            continue
        flat = expand(fields)
        missing = [key for key in keys if key not in flat]
        extra = [key for key in flat if key not in keys]
        if missing or extra:
            raise SystemExit(f"{code}: missing={missing} extra={extra}")
        path = DEST / f"{name}.json"
        path.write_text(json.dumps(nest(flat), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        written.add(name)
        print("wrote", path.relative_to(ROOT))
    stale = {item.stem for item in DEST.glob("*.json")} - written
    if stale:
        raise SystemExit(f"stale translation files: {sorted(stale)}")


if __name__ == "__main__":
    write_ha_translations()
