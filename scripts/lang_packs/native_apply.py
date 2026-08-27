"""Apply native-script overlays onto generated lexicon cores."""

from __future__ import annotations

import json
from pathlib import Path

from lang_packs.core import living_kitchen, nums

_WORD_KEYS = (
    "on",
    "off",
    "open",
    "close",
    "query",
    "set",
    "light",
    "cover",
    "climate",
    "media",
    "lock",
    "door",
    "timer",
    "list",
    "fan",
    "vacuum",
    "scene",
    "fillers",
    "and",
    "or",
    "yes",
    "all",
)

_SURFACES: dict[str, dict] | None = None


def _unique(values: list[str]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for item in values:
        if item and item not in seen:
            seen.add(item)
            out.append(item)
    return out


def surfaces() -> dict[str, dict]:
    global _SURFACES
    if _SURFACES is None:
        path = Path(__file__).with_name("native_surfaces.json")
        _SURFACES = json.loads(path.read_text(encoding="utf-8"))
    return _SURFACES


def is_script_pack(core: dict) -> bool:
    return core.get("script", "Latn") != "Latn"


def apply_native(core: dict) -> None:
    row = surfaces().get(core["code"])
    if not row:
        return
    w = core["w"]
    for key in _WORD_KEYS:
        if key in row:
            w[key] = _unique(list(row[key]) + list(w.get(key) or []))
    for key, val in (row.get("extra_w") or {}).items():
        w[key] = _unique(list(val) + list(w.get(key) or []))
    living = row.get("living")
    kitchen = row.get("kitchen")
    if living and kitchen:
        old_rooms = list(core.get("rooms") or [])
        merged = []
        seen: set[tuple[str, str]] = set()
        for pair in living_kitchen(living, kitchen) + old_rooms:
            if pair in seen:
                continue
            seen.add(pair)
            merged.append(pair)
        core["rooms"] = merged
        w["kitchen"] = _unique([kitchen] + list(w.get("kitchen") or []))
    if row.get("numbers"):
        core["numbers"] = nums(list(row["numbers"]))
    if row.get("colors"):
        core["colors"] = [tuple(pair) for pair in row["colors"]] + list(core.get("colors") or [])
    speech = core.setdefault("speech", {})
    for key, value in (row.get("speech") or {}).items():
        speech[key] = value
    extra = list(core.get("extra_verbs") or [])
    extra.extend((word, kind) for word, kind in row.get("extra_verbs") or [])
    core["extra_verbs"] = extra
    on0, off0, light0 = w["on"][0], w["off"][0], w["light"][0]
    liv = living or core["rooms"][0][0]
    kit = kitchen or (w.get("kitchen") or [liv])[0]
    core["smoke"] = [
        (f"{on0} {light0} {liv}", "HassTurnOn"),
        (f"{off0} {light0} {kit}", "HassTurnOff"),
    ]


def apply_calendar(calendar: dict) -> None:
    data = json.loads(Path(__file__).with_name("native_calendar.json").read_text(encoding="utf-8"))
    for code, patch in data.items():
        row = calendar.get(code)
        if not row:
            continue
        for key in ("nouns", "query", "create", "today", "tomorrow", "when", "resume", "delete", "move"):
            if key in patch:
                row[key] = _unique(list(patch[key]) + list(row.get(key) or []))
        for key in ("list_smoke", "create_smoke", "delete_smoke", "move_smoke"):
            if key in patch:
                row[key] = patch[key]
        speech = row.setdefault("speech", {})
        for key, value in (patch.get("speech") or {}).items():
            speech[key] = value
