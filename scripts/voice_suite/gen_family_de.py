#!/usr/bin/env python3
"""Build a German suite comparable to family_home_medium_two_storey."""

from __future__ import annotations

import shutil
from pathlib import Path

import yaml

from .family.labels import AREA, COLOR, NAME, SCENE_CASES, SCRIPT_CASES
from .family.spoken_de import SPEECH_DE, SPOKEN_DE

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "tests" / "datasets" / "family_home_en"
DST = ROOT / "tests" / "datasets" / "familienhaus_de"


def area_label(aid: str) -> str:
    return AREA.get(aid, (aid, f"in {aid}", []))[0]


def area_prep(aid: str) -> str:
    return AREA.get(aid, (aid, f"in {aid}", []))[1]


def load_home() -> dict:
    return yaml.safe_load((SRC / "home_config.yaml").read_text())


def load_names() -> dict[str, str]:
    home = load_home()
    out = {}
    for d in home.get("devices") or []:
        out[d["id"]] = NAME.get(d["name"], d["name"])
    for key, domain in (("scenes", "scene"), ("scripts", "script"), ("timers", "timer"), ("lists", "todo")):
        for item in home.get(key) or []:
            eid = item["id"] if "." in item["id"] else f"{domain}.{item['id']}"
            out[eid] = NAME.get(item["name"], item["name"])
    return out


def load_entity_areas() -> dict[str, str]:
    home = load_home()
    return {d["id"]: d.get("area_id") for d in home.get("devices") or [] if d.get("area_id")}


def conds(case: dict) -> list[dict]:
    return [c for c in case.get("conditions") or [] if isinstance(c, dict)]


def first_action(case: dict) -> dict:
    cs = conds(case)
    return cs[0] if cs else {}


def attr(c: dict, key: str):
    attrs = c.get("attributes") or {}
    return attrs.get(key, c.get(key))


def de_sentences(
    case: dict, names: dict[str, str], areas_of: dict[str, str], multi: bool = False
) -> list:
    raw = case.get("sentences")
    if raw and isinstance(raw, list) and raw and isinstance(raw[0], list):
        return [de_turns(pair, case, names, areas_of) for pair in raw]
    name = str(case.get("name") or "")
    if name in SPOKEN_DE:
        return SPOKEN_DE[name]
    if name in SCENE_CASES:
        return templates({"entity_id": SCENE_CASES[name], "state": "on"}, names, [])
    if name in SCRIPT_CASES:
        return templates({"entity_id": SCRIPT_CASES[name], "state": "on"}, names, [])
    if multi and len(conds(case)) > 1:
        parts = [templates(c, names, [c], areas_of)[0] for c in conds(case)]
        parts = [p for p in parts if p]
        if parts:
            return [" und ".join(parts)]
    if "_and_" in name and len(conds(case)) > 1:
        parts = [templates(c, names, [c], areas_of)[0] for c in conds(case)]
        parts = [p for p in parts if p]
        if parts:
            return [" und ".join(parts)]
    c = first_action(case)
    return templates(c, names, conds(case), areas_of)


def de_turns(pair: list, case: dict, names: dict[str, str], areas_of: dict[str, str]) -> list[str]:
    c = first_action(case)
    eid = c.get("entity_id")
    if isinstance(eid, dict):
        eid = eid.get("id")
    area = c.get("area") or areas_of.get(eid or "")
    domain = c.get("domain") or (eid.split(".")[0] if isinstance(eid, str) and "." in eid else "light")
    all_areas = []
    for cond in conds(case):
        ce = cond.get("entity_id")
        if isinstance(ce, dict):
            ce = ce.get("id")
        aid = cond.get("area") or areas_of.get(ce or "")
        if aid and aid not in all_areas:
            all_areas.append(aid)
    room = " und ".join(area_label(a) for a in all_areas) if all_areas else (area_label(area) if area else "")
    follow_blob = " ".join(pair[1:]).lower() if len(pair) > 1 else ""
    number_follow = any(ch.isdigit() for ch in follow_blob) or "percent" in follow_blob or "fifty" in follow_blob
    light_word = "Lichter" if len(all_areas) > 1 or number_follow else "Licht"
    fallback = {
        "fan": "Lüfter",
        "cover": "Rollo",
        "lock": "Schloss",
        "light": light_word,
        "climate": "Heizung",
        "media_player": "Fernseher",
        "switch": "Gerät" if area == "laundry" else "Schalter",
    }.get(domain or "light", light_word)
    first_en = str(pair[0]).lower() if pair else ""
    persist_query = any(
        w in first_en
        for w in ("what", "how", "where", "tell", "check", "status", "is ", "are ", "what's")
    )
    state = str(c.get("state") or "")
    if persist_query:
        first = f"Status {fallback} {room}".strip() if room else f"Status {fallback}"
    else:
        first = f"Mach {fallback} {room} an".strip() if room else f"Mach {fallback} an"
    if any(w in follow_blob for w in ("ceiling", "decke")):
        follow = "die Decke"
    elif any(w in follow_blob for w in ("island", "insel")):
        follow = "die Insel"
    elif any(w in follow_blob for w in ("lamp", "lampe")):
        follow = "die Lampe"
    elif number_follow:
        n = (
            attr(c, "brightness")
            or attr(c, "temperature")
            or attr(c, "position")
            or attr(c, "percentage")
            or 21
        )
        follow = f"auf {n}"
    elif any(w in follow_blob for w in ("off", "aus", "close", "zu", "unlock")) or state in (
        "off",
        "closed",
        "unlocked",
    ):
        follow = "mach sie zu" if domain == "cover" else "mach sie aus"
    elif persist_query:
        follow = "mach sie auf" if domain == "cover" else "mach sie an"
    else:
        follow = "ja"
    return [first, follow]


def templates(
    c: dict, names: dict[str, str], all_conds: list[dict], areas_of: dict[str, str] | None = None
) -> list[str]:
    kind = c.get("type", "action")
    eid = c.get("entity_id")
    if isinstance(eid, dict):
        eid = eid.get("id")
    areas_of = areas_of or {}
    area = c.get("area") or areas_of.get(eid or "")
    domain = c.get("domain") or (eid.split(".")[0] if isinstance(eid, str) and "." in eid else None)
    state = str(c.get("state") or "")
    raw_name = names.get(eid or "", "")
    fallback = {
        "fan": "Lüfter",
        "cover": "Rollo",
        "lock": "Schloss",
        "light": "Licht",
        "climate": "Heizung",
        "media_player": "Fernseher",
    }.get(domain or "", "")
    where = area_prep(area) if area else ""
    room = area_label(area) if area else ""
    areas = []
    for x in all_conds:
        xe = x.get("entity_id")
        if isinstance(xe, dict):
            xe = xe.get("id")
        aid = x.get("area") or areas_of.get(xe or "")
        if aid and aid not in areas:
            areas.append(aid)
    rooms = " und ".join(area_label(a) for a in areas if a)
    name = raw_name or (" ".join(p for p in (fallback, room) if p).strip())
    if (eid or "").startswith("timer."):
        name = name or "Timer"
    bri = attr(c, "brightness")
    temp = attr(c, "temperature")
    color = attr(c, "color")
    pos = attr(c, "position")
    pct = attr(c, "percentage")
    minutes = c.get("minutes")
    seconds = c.get("seconds")
    hours = c.get("hours")
    item = c.get("item")

    muted = attr(c, "is_volume_muted")
    if muted is not None:
        if str(muted).lower() in ("true", "on", "yes"):
            return [f"{name} stumm", f"Stumm {name}", f"{name} lautlos"]
        return [f"{name} laut", f"Stummschaltung {name} aus"]

    if kind == "query" or (
        kind == "action"
        and not state
        and bri is None
        and temp is None
        and color is None
        and pos is None
        and pct is None
        and not item
        and minutes is None
        and seconds is None
        and hours is None
    ):
        if "warm" in str(c) or domain == "climate" or (eid or "").startswith("climate."):
            target = where or name or "zuhause"
            return [
                f"Wie warm ist es {target}",
                f"Temperatur {room or name}",
                f"Wie ist die Temperatur {target}",
                f"Wie kalt ist es {target}",
            ]
        if domain == "fan" or (eid or "").startswith("fan."):
            target = name if "Lüfter" in (name or "") else f"Lüfter {room or name}".strip()
        else:
            target = rooms or name or room
        where_all = " und ".join(area_prep(a) for a in areas if a) if len(areas) > 1 else where
        return [
            f"Ist {target} an",
            f"Status {target}",
            f"Wie ist der Zustand von {target}",
            f"Ist das {domain or 'Gerät'} {where_all} an" if where_all else f"Status {target}",
        ]

    if kind in ("shopping_list", "todo_list") or item:
        return [
            f"Setze {item} auf die Einkaufsliste",
            f"Füge {item} zur Liste hinzu",
            f"{item} auf die Liste",
        ]

    if minutes is not None or seconds is not None or hours is not None or (eid or "").startswith("timer."):
        label = name or "Timer"
        dur = minutes or hours or seconds or 2
        unit = "Sekunden" if seconds and not minutes else "Minuten"
        return [
            f"Starte den Timer {label} für {dur} {unit}",
            f"Timer {label} {dur} {unit}",
            f"Setze {label} auf {dur} {unit}",
        ]

    if temp is not None:
        place = rooms or room or name
        t = (" und ".join(area_prep(a) for a in areas if a) if len(areas) > 1 else where) or name
        return [
            f"Heizung {place} auf {temp} Grad",
            f"Stell die Temperatur {t} auf {temp}",
            f"Temperatur {place} {temp}",
            f"{place} auf {temp} Grad",
        ]

    if bri is not None:
        t = rooms or name or room
        return [
            f"Setze {t} auf {bri} Prozent",
            f"{t} Helligkeit {bri}",
            f"Dimme {t} auf {bri}",
            f"Lichter {t} {bri} Prozent",
        ]

    if color:
        farbe = COLOR.get(str(color), str(color))
        t = rooms or name or room
        return [
            f"Mach {t} {farbe}",
            f"Setze {t} auf {farbe}",
            f"{t} {farbe}",
            f"Lichter {t} {farbe}",
        ]

    if pos is not None:
        place = rooms or room or name
        return [
            f"Rollo {place} auf {pos} Prozent",
            f"Setze {name or 'das Rollo'} auf {pos}",
            f"{place} Position {pos}",
        ]

    if pct is not None:
        return [
            f"Lüfter {room or name} auf {pct} Prozent",
            f"Setze {name} auf {pct}",
            f"{name} {pct} Prozent",
        ]

    if domain == "cover" or (eid or "").startswith("cover."):
        place = rooms or room
        label = name if not rooms else f"Rollo {place}"
        label = label or f"Rollo {room}".strip()
        if state in ("closed", "off"):
            return [f"Mach {label} zu", f"Rollo {place} zu" if place else f"{label} zu", f"Schließ {label}"]
        return [f"Mach {label} auf", f"Rollo {place} auf" if place else f"{label} auf", f"Öffne {label}"]

    if domain == "lock" or (eid or "").startswith("lock."):
        if state in ("unlocked", "off"):
            return [f"Schließ {name} auf", f"{name} aufschließen", f"Tür {room} auf" if room else f"{name} öffnen"]
        return [f"Schließ {name} ab", f"{name} abschließen", f"Tür {room} zu" if room else f"{name} verriegeln"]

    if domain == "scene" or (eid or "").startswith("scene.") or (eid or "").startswith("script."):
        return [f"Szene {name}", f"Starte {name}", f"{name} Szene an", f"Aktiviere {name}"]

    if domain == "media_player" or (eid or "").startswith("media_player."):
        if state == "paused":
            return [f"Pause {name}", f"{name} pausieren"]
        if state in ("off",):
            return [f"Mach {name} aus", f"{name} aus"]
        if "mute" in str(c).lower() or state in ("muted", "on"):
            if "mute" in str(c).lower() and state != "on":
                return [f"{name} stumm", f"Stumm {name}"]
        return [f"Mach {name} an", f"{name} an", f"Fernseher {room} an" if room else f"{name} einschalten"]

    if domain == "switch" or (eid or "").startswith("switch."):
        verb = "aus" if state in ("off", "closed") else "an"
        if area:
            return [
                f"Schalt die Schalter {where} {verb}",
                f"Schalter {room} {verb}",
                f"Mach die Schalter {where} {verb}",
            ]
        return [f"Mach {name} {verb}", f"{name} {verb}"]

    if domain == "fan" or (eid or "").startswith("fan."):
        label = name or f"Lüfter {room}".strip()
        if state in ("off",):
            return [f"Lüfter {room} aus" if room else f"{label} aus", f"Mach {label} aus"]
        return [f"Lüfter {room} an" if room else f"{label} an", f"Mach {label} an"]

    if len(areas) > 1 and state in ("on", "off"):
        verb = "an" if state == "on" else "aus"
        rooms = " und ".join(area_prep(a) for a in areas)
        labels = " und ".join(area_label(a) for a in areas)
        return [
            f"Mach die Lichter {rooms} {verb}",
            f"Lichter {labels} {verb}",
            f"Schalt {labels} {verb}",
        ]

    if state in ("off", "closed"):
        if area and domain == "light":
            return [
                f"Mach die Lichter {where} aus",
                f"Lichter {room} aus",
                f"Schalt die Lichter {where} aus",
                f"{room} Lichter aus",
            ]
        return [f"Mach {name or room} aus", f"{name or room} aus", f"Schalt {name or room} aus"]

    if area and domain == "light":
        return [
            f"Mach die Lichter {where} an",
            f"Lichter {room} an",
            f"Schalt die Lichter {where} ein",
            f"{room} Lichter an",
            f"Kannst du die Lichter {where} anmachen",
        ]
    return [f"Mach {name or room} an", f"{name or room} an", f"Schalt {name or room} ein"]


def write_home() -> None:
    home = yaml.safe_load((SRC / "home_config.yaml").read_text())
    home["name"] = "Familienhaus Mittel"
    home["language"] = "de"
    for a in home.get("areas") or []:
        label, _prep, aliases = AREA.get(a["id"], (a["name"], a["name"], []))
        a["name"] = label
        a["aliases"] = aliases
    for d in home.get("devices") or []:
        d["name"] = NAME.get(d["name"], d["name"])
    for key in ("scenes", "scripts", "timers", "lists"):
        for item in home.get(key) or []:
            item["name"] = NAME.get(item["name"], item["name"])
    DST.mkdir(parents=True, exist_ok=True)
    (DST / "home_config.yaml").write_text(
        yaml.safe_dump(home, allow_unicode=True, sort_keys=False),
        encoding="utf-8",
    )


def convert_file(src: Path, names: dict[str, str], areas_of: dict[str, str]) -> None:
    data = yaml.safe_load(src.read_text())
    items = data if isinstance(data, list) else [data]
    out = []
    for case in items:
        if not isinstance(case, dict) or "sentences" not in case:
            continue
        case = dict(case)
        case["sentences"] = de_sentences(
            case, names, areas_of, multi="multiple_intents" in str(src)
        )
        for key in ("speech_has", "speech_forbids"):
            if key in case:
                case[key] = [SPEECH_DE.get(item, item) for item in case[key]]
        out.append(case)
    dest = DST / src.relative_to(SRC)
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_text(yaml.safe_dump(out, allow_unicode=True, sort_keys=False), encoding="utf-8")


def main() -> None:
    if DST.exists():
        shutil.rmtree(DST)
    write_home()
    names = load_names()
    areas_of = load_entity_areas()
    skip = {"home_config.yaml"}
    count = 0
    for path in sorted(SRC.rglob("*.yaml")):
        if path.name in skip:
            continue
        convert_file(path, names, areas_of)
        count += 1
    print(f"familienhaus_de: {count} Dateien")


if __name__ == "__main__":
    main()
