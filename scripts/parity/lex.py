"""Room aliases and compact lexemes for parity sentence generation."""

from __future__ import annotations

from pathlib import Path

import yaml

from lang_packs.extras import FAMILY, floors, pack_extras

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "tests" / "datasets"
PAIRS = [
    ("wohnzimmer", "living"),
    ("kuche", "kitchen"),
    ("esszimmer", "dining"),
    ("badezimmer", "main_bath"),
    ("flur", "hallway"),
    ("arbeitszimmer", "office"),
    ("balkon", "balcony"),
    ("wohnung", "home"),
]
CANON_AREAS = {key: [a, b] for a, b in PAIRS for key in (a, b)}
CANON_AREAS.update(
    {
        key: [val]
        for key, val in (
            ("arbeitszimmer", "arbeitszimmer"),
            ("office", "arbeitszimmer"),
            ("balkon", "balkon"),
            ("balcony", "balkon"),
            ("wohnung", "wohnung"),
            ("home", "wohnung"),
        )
    }
)
HOMES: dict[str, dict[str, dict[str, str]]] = {}


def lex_of(core: dict) -> dict:
    w = dict(core["w"])
    for key, val in pack_extras(core["code"]).items():
        w.setdefault(key, val)
    rooms: dict[str, list[str]] = {}
    for native, canon in core.get("rooms", []):
        rooms.setdefault(canon, []).append(native)
        for area in CANON_AREAS.get(canon, [canon]):
            rooms.setdefault(area, []).append(native)
    living = (rooms.get("wohnzimmer") or rooms.get("living") or [w.get("on", ["on"])[0]])[0]
    kitchen = (rooms.get("kuche") or rooms.get("kitchen") or [living])[0]
    bed = (rooms.get("schlafzimmer") or [living])[0]
    compact = bed.replace(" ", "")
    extra = FAMILY.get(
        core["code"],
        {
            "entryway": [f"{living}in"],
            "family_room": [f"{living}fam"],
            "laundry": [f"{kitchen}wash"],
            "powder_room": [f"{kitchen}wc"],
            "garage": ["garage"],
            "bedroom_2": [f"{compact}2"],
            "bedroom_3": [f"{compact}3"],
            "bedroom_4": [f"{compact}4"],
            "master_bath": rooms.get("badezimmer") or [kitchen],
            "kids": [f"{compact}kids"],
        },
    )
    for key in ("bedroom_2", "bedroom_3", "bedroom_4"):
        extra[key] = [name.replace(" ", "") for name in extra.get(key, [f"{compact}{key[-1]}"])]
    for area, names in extra.items():
        rooms.setdefault(area, []).extend(names)
    stem = extra.get("bedroom_2", [compact])[0]
    stem = stem[:-1] if stem[-1:].isdigit() else stem
    rooms.setdefault("schlafzimmer", []).append(stem)
    rooms["master_bedroom"] = [f"{stem}master"]
    rooms.setdefault("esszimmer", []).append((rooms.get("esszimmer") or rooms.get("dining") or [living])[0])
    rooms.setdefault("dining", []).append((rooms.get("esszimmer") or [living])[0])
    hall = (rooms.get("flur") or rooms.get("hallway") or extra.get("hallway") or [f"{(extra.get('entryway') or [living])[0]}gang"])[0]
    if hall in set(extra.get("entryway") or []):
        hall = f"{hall}gang"
    rooms.setdefault("flur", []).append(hall)
    rooms.setdefault("hallway", []).append(hall)
    rooms.setdefault("badezimmer", []).append((rooms.get("badezimmer") or extra.get("master_bath") or [kitchen])[0])
    office = rooms.get("arbeitszimmer") or rooms.get("office") or []
    if not office or office[0] == living:
        rooms["arbeitszimmer"] = ["office"]
        rooms.setdefault("office", ["office"])
    else:
        rooms.setdefault("arbeitszimmer", []).append(office[0])
    first = lambda key, fallback=None: (w.get(key) or fallback or w["set"])[0]
    ands = w.get("and") or ["and"]
    lights = w.get("light") or ["light"]
    lamp = (w.get("lamp") or [None])[0] or (lights[1] if len(lights) > 1 else "lamp")
    lex = {key: w[key][0] for key in ("on", "off", "open", "close", "query", "set", "light", "cover", "climate", "media", "lock", "door", "timer", "list", "fan", "vacuum", "scene", "and", "yes")}
    lex.update(
        {
            "code": core["code"],
            "except": (w.get("except") or ["except"])[0],
            "all": w["all"][0],
            "add": first("add", ["add"]),
            "done": first("done", ["done"]),
            "lock_v": first("lock_v", w.get("lock") or ["verrouille"]),
            "unlock": first("unlock", w.get("open") or ["open"]),
            "minutes": first("minutes", ["min"]),
            "pause": first("pause", ["pause"]),
            "island": first("island", ["island"]),
            "ceiling": first("ceiling", w.get("light") or ["light"]),
            "globe": first("globe", w.get("named") or ["globe"]),
            "bedside": first("bedside", ["bedside"]),
            "device": first("device", w.get("switch") or ["device"]),
            "chores": (w.get("list") or ["list"])[-1],
            "good_night": first("good_night", ["night"]),
            "leaving": first("leaving", ["leaving"]),
            "film": first("scenes", w.get("scene") or ["scene"]),
            "switch": first("switch", w.get("device") or ["switch"]),
            "dishwasher": first("dishwasher", w.get("device") or ["switch"]),
            "washer": first("washer", w.get("device") or ["switch"]),
            "dryer": first("dryer", ["dryer"]),
            "tv": first("tv", w.get("media") or ["tv"]),
            "lamp": lamp,
            "light_one": lights[2] if len(lights) > 2 else lights[0],
            "on2": w["on"][1] if len(w["on"]) > 1 else w["on"][0],
            "then": ands[1] if len(ands) > 1 else "then",
            "colors": {canon: native for native, canon in core.get("colors") or []},
            "rooms": {key: unique(vals) for key, vals in rooms.items()},
            "floors": floors(core["code"]),
            "reject": f"{(core.get('chat') or {}).get('world', ['weather'])[0]} france",
        }
    )
    return lex


def unique(values: list[str]) -> list[str]:
    seen: set[str] = set()
    return [item for item in values if item and not (item in seen or seen.add(item))]


def room(lex: dict, area: str | None) -> str:
    if not area:
        return ""
    names = lex["rooms"].get(area) or lex["rooms"].get(area.replace("_", "")) or [area.replace("_", " ")]
    return names[0]


def home_of(suite: str) -> dict[str, dict[str, str]]:
    if suite not in HOMES:
        path = DATA / suite / "home_config.yaml"
        raw = yaml.safe_load(path.read_text(encoding="utf-8")) if path.exists() else {}
        found: dict[str, dict[str, str]] = {}
        for item in raw.get("devices") or []:
            found[item["id"]] = {"name": item.get("name", item["id"]), "area": item.get("area_id") or ""}
        for kind, key in (("scene", "scenes"), ("script", "scripts")):
            for item in raw.get(key) or []:
                ident = item["id"] if "." in item["id"] else f"{kind}.{item['id']}"
                found[ident] = {"name": item.get("name", ident), "area": ""}
        HOMES[suite] = found
    return HOMES[suite]


def entity_info(suite: str, entity_id: str) -> dict[str, str]:
    found = home_of(suite).get(entity_id)
    if found:
        return found
    return {"name": entity_id.split(".")[-1].replace("_", " "), "area": ""}


def color_word(lex: dict, color: str) -> str:
    return lex["colors"].get(str(color), str(color))


def domain_of(cond: dict) -> str:
    if cond.get("domain"):
        return str(cond["domain"])
    attrs = cond.get("attributes") or {}
    if attrs.get("temperature") is not None:
        return "climate"
    if attrs.get("percentage") is not None:
        return "fan"
    if attrs.get("position") is not None:
        return "cover"
    if attrs.get("color") or attrs.get("brightness") is not None:
        return "light"
    entity = cond.get("entity_id")
    if entity:
        return str(entity).split(".")[0]
    return "light"


def domain_noun(lex: dict, domain: str | None) -> str:
    return {
        "light": lex["light"],
        "cover": lex["cover"],
        "climate": lex["climate"],
        "media_player": lex["media"],
        "lock": lex["lock"],
        "fan": lex["fan"],
        "vacuum": lex["vacuum"],
        "scene": lex["scene"],
        "script": lex["scene"],
        "timer": lex["timer"],
        "switch": lex["switch"],
    }.get(domain or "light", lex["light"])
