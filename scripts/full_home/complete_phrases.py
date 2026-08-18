#!/usr/bin/env python3
"""Fill missing phrase kinds from existing templates and shared defaults."""

from __future__ import annotations

from pathlib import Path

import yaml

from realize import DEFAULTS, DE_EXTRAS, EN_EXTRAS

HERE = Path(__file__).resolve().parent
PHRASES = HERE / "phrases"
KINDS = list(DEFAULTS)
TARGET = 6


def _flip(templates: list[str], src: str, dest: str) -> list[str]:
    out = []
    for item in templates:
        if src in item:
            out.append(item.replace(src, dest))
        elif dest not in item:
            out.append(item)
    return list(dict.fromkeys(out)) or list(templates)


def _parent(code: str) -> dict[str, list[str]]:
    lowered = code.lower().replace("_", "-")
    if lowered == "de" or lowered.startswith("de-"):
        return DE_EXTRAS
    if lowered == "en" or lowered.startswith("en-"):
        return EN_EXTRAS
    return {}


def _derive(raw: dict, kind: str, parent: dict) -> list[str]:
    if parent.get(kind):
        return list(parent[kind])
    if kind == "off_area" and raw.get("on_area"):
        return _flip(raw["on_area"], "{on}", "{off}")
    if kind == "off_fixture" and raw.get("on_fixture"):
        return _flip(raw["on_fixture"], "{on}", "{off}")
    if kind == "fan_off" and raw.get("fan_on"):
        return _flip(raw["fan_on"], "{on}", "{off}")
    if kind == "switch_off" and raw.get("switch_on"):
        return _flip(raw["switch_on"], "{on}", "{off}")
    if kind == "media_off" and raw.get("media_on"):
        return _flip(raw["media_on"], "{on}", "{off}")
    if kind == "close_cover" and raw.get("open_cover"):
        return _flip(raw["open_cover"], "{open}", "{close}")
    if kind == "unlock" and raw.get("lock"):
        return _flip(raw["lock"], "{lock_v}", "{unlock}")
    if kind == "vac_dock" and raw.get("vac_start"):
        return _flip(raw["vac_start"], "{on}", "{off}")
    if kind == "floor_off" and raw.get("floor_on"):
        return _flip(raw["floor_on"], "{on}", "{off}")
    if kind == "floor_on" and raw.get("on_area"):
        return [item.replace("{room}", "{floor}") for item in raw["on_area"]]
    if kind == "set_pos" and raw.get("set_bright"):
        return [item.replace("{fixture}", "{cover}").replace("{light}", "{cover}") for item in raw["set_bright"]]
    if kind == "set_color" and raw.get("set_bright"):
        return [item.replace("{n}", "{color}") for item in raw["set_bright"]]
    if kind == "list_done" and raw.get("list_add"):
        return [item.replace("{add}", "{done}") for item in raw["list_add"]]
    if kind == "play_queue" and raw.get("play_search"):
        return list(raw["play_search"])
    if kind == "script" and raw.get("scene"):
        return list(raw["scene"])
    if kind == "timer_cancel" and raw.get("timer_start"):
        return _flip(raw["timer_start"], "{on}", "{off}")[:2] or list(DEFAULTS["timer_cancel"])
    if kind == "media_pause" and raw.get("media_on"):
        return [item.replace("{on}", "{pause}") for item in raw["media_on"]]
    if kind == "media_vol" and raw.get("set_bright"):
        return [item.replace("{fixture}", "{music}").replace("{light}", "{music}") for item in raw["set_bright"]]
    if kind == "media_next":
        return ["next track"]
    if kind == "media_prev":
        return ["previous track"]
    if kind == "fan_on" and raw.get("on_fixture"):
        return [item.replace("{fixture}", "{fan}") for item in raw["on_fixture"]]
    if kind == "fan_speed" and raw.get("set_bright"):
        return [item.replace("{fixture}", "{fan}").replace("{light}", "{fan}") for item in raw["set_bright"]]
    if kind == "switch_on":
        return ["{on} {appliance}", "{appliance} {on}"]
    if kind == "multi_off_lock" and raw.get("multi_and"):
        return _flip(raw["multi_and"], "{on}", "{off}")
    if kind == "multi_off" and raw.get("multi_and"):
        return _flip(raw["multi_and"], "{on}", "{off}")
    if kind == "all_except_on" and raw.get("all_except"):
        return _flip(raw["all_except"], "{off}", "{on}")
    if kind == "except_fixture" and raw.get("all_except"):
        return [
            item.replace("{except} {room}", "{except} {skip_fixture} {room}") if "{room}" in item else item
            for item in raw["all_except"]
        ]
    if kind == "multi_fixtures" and raw.get("on_fixture"):
        return ["{on} {room} {fixture} {and} {room2} {fixture2}"]
    if kind == "multi_fixtures_off" and raw.get("multi_fixtures"):
        return _flip(raw["multi_fixtures"], "{on}", "{off}")
    if kind == "multi_three" and raw.get("multi_and"):
        return [item.replace("{room2}", "{room2} {and} {room3}") for item in raw["multi_and"]]
    if kind == "multi_three_off" and raw.get("multi_three"):
        return _flip(raw["multi_three"], "{on}", "{off}")
    if kind == "except_in_area" and raw.get("all_except"):
        return [item.replace("{room}", "{room}").replace("{except} {room}", "{except} {skip_fixture}") if "{room}" in item else item for item in raw["all_except"]]
    if kind == "except_two" and raw.get("all_except"):
        return [item.replace("{room}", "{room} {and} {room2}") for item in raw["all_except"]]
    if kind == "except_fixture_on" and raw.get("except_fixture"):
        return _flip(raw["except_fixture"], "{off}", "{on}")
    if kind == "floor_except" and raw.get("all_except"):
        return [item.replace("{light}", "{light} {floor}") for item in raw["all_except"]]
    if kind == "multi_covers" and raw.get("open_cover"):
        return [item + " {and} {room2}" if "{room2}" not in item else item for item in raw["open_cover"]]
    if kind == "multi_covers_close" and raw.get("close_cover"):
        return [item + " {and} {room2}" if "{room2}" not in item else item for item in raw["close_cover"]]
    if kind == "multi_bright" and raw.get("set_bright"):
        return [item.replace("{room}", "{room} {and} {room2}") for item in raw["set_bright"]]
    if kind == "multi_color" and raw.get("set_color"):
        return [item.replace("{room}", "{room} {and} {room2}") for item in raw["set_color"]]
    if kind == "multi_climate" and raw.get("set_temp"):
        return [item.replace("{room}", "{room} {and} {room2}") for item in raw["set_temp"]]
    if kind == "query_two" and raw.get("query_entity"):
        return [item.replace("{room}", "{room} {and} {room2}") for item in raw["query_entity"]]
    if kind == "query_and_off":
        return ["{query} {climate} {room} {and} {off} {light} {room2}"]
    if kind == "multi_locks" and raw.get("lock"):
        return [item.replace("{room}", "{room} {and} {room2}") for item in raw["lock"]]
    if kind == "scene_and_off" and raw.get("scene"):
        return [f"{item} {{and}} {{off}} {{light}} {{room}}" for item in raw["scene"]]
    return list(DEFAULTS.get(kind) or ["{on} {light}"])


def _pad(values: list[str], kind: str) -> list[str]:
    out = list(dict.fromkeys(values))
    if kind == "play_search":
        roomed = [item for item in out if "{room}" in item]
        if roomed:
            out = roomed + [item for item in out if item not in roomed]
    for item in DEFAULTS.get(kind) or []:
        if len(out) >= TARGET:
            break
        if item not in out:
            if kind == "play_search" and "{room}" not in item and any("{room}" in x for x in out):
                continue
            out.append(item)
    return out


def complete(raw: dict, code: str) -> dict:
    parent = _parent(code)
    out = {key: list(val) for key, val in raw.items() if isinstance(val, list)}
    for kind in KINDS:
        if not out.get(kind):
            out[kind] = _derive(out, kind, parent)
        out[kind] = _pad(out[kind], kind)
    return {key: out[key] for key in KINDS if out.get(key)}


def _dump(path: Path, data: dict) -> None:
    path.write_text(yaml.safe_dump(data, allow_unicode=True, sort_keys=False), encoding="utf-8")


def main() -> None:
    PHRASES.mkdir(parents=True, exist_ok=True)
    _dump(PHRASES / "de.yaml", complete(DE_EXTRAS, "de"))
    _dump(PHRASES / "en.yaml", complete(EN_EXTRAS, "en"))
    print("wrote de.yaml en.yaml")
    for path in sorted(PHRASES.glob("*.yaml")):
        if path.name in {"de.yaml", "en.yaml"}:
            continue
        raw = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
        filled = complete(raw, path.stem)
        _dump(path, filled)
        print("completed", path.name, "kinds", len(filled), "templates", sum(len(v) for v in filled.values()))


if __name__ == "__main__":
    main()
