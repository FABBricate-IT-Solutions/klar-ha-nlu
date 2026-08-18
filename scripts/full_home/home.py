"""Language-neutral full smart-home graph with native names per locale."""

from __future__ import annotations

from lex import floor_word, room

FLOORS = (("ground", 0), ("upper", 1), ("basement", -1))

AREAS = (
    ("entryway", "ground"),
    ("living", "ground"),
    ("dining", "ground"),
    ("kitchen", "ground"),
    ("family_room", "ground"),
    ("laundry", "ground"),
    ("powder_room", "ground"),
    ("garage", "ground"),
    ("office", "ground"),
    ("garden", "ground"),
    ("master_bedroom", "upper"),
    ("bedroom_2", "upper"),
    ("bedroom_3", "upper"),
    ("bedroom_4", "upper"),
    ("master_bath", "upper"),
    ("main_bath", "upper"),
    ("hallway", "upper"),
    ("basement", "basement"),
)

# id, area, kind, extra aliases key
DEVICES = (
    ("light.entryway_light", "entryway", "light", None),
    ("light.living_ceiling", "living", "ceiling", None),
    ("light.living_lamp", "living", "lamp", None),
    ("light.dining_pendant", "dining", "light", None),
    ("light.kitchen_ceiling", "kitchen", "ceiling", None),
    ("light.kitchen_island", "kitchen", "island", None),
    ("light.family_ceiling", "family_room", "ceiling", None),
    ("light.laundry_light", "laundry", "light", None),
    ("light.powder_room_light", "powder_room", "light", None),
    ("light.garage_light", "garage", "light", None),
    ("light.office_ceiling", "office", "ceiling", None),
    ("light.garden", "garden", "light", None),
    ("light.master_ceiling", "master_bedroom", "ceiling", None),
    ("light.master_ensuite", "master_bedroom", "light", "ensuite"),
    ("light.master_bedside_left", "master_bedroom", "bedside", "left"),
    ("light.master_bedside_right", "master_bedroom", "bedside", "right"),
    ("light.bedroom2_ceiling", "bedroom_2", "ceiling", None),
    ("light.bedroom3_ceiling", "bedroom_3", "ceiling", None),
    ("light.bedroom4_ceiling", "bedroom_4", "ceiling", None),
    ("light.master_bath_light", "master_bath", "light", None),
    ("light.main_bath_light", "main_bath", "light", None),
    ("light.hallway_light", "hallway", "light", None),
    ("light.basement", "basement", "light", None),
    ("cover.living_blinds", "living", "cover", None),
    ("cover.master_blinds", "master_bedroom", "cover", None),
    ("cover.bedroom2_blinds", "bedroom_2", "cover", None),
    ("cover.bedroom3_blinds", "bedroom_3", "cover", None),
    ("cover.garage_door", "garage", "cover", None),
    ("climate.ground_thermostat", "living", "climate", None),
    ("climate.upper_thermostat", "master_bedroom", "climate", None),
    ("climate.master_ac", "master_bedroom", "climate", "ac"),
    ("fan.family_fan", "family_room", "fan", None),
    ("fan.master_fan", "master_bedroom", "fan", None),
    ("fan.bedroom2_fan", "bedroom_2", "fan", None),
    ("lock.front_door", "entryway", "lock", None),
    ("lock.garage_entry", "garage", "lock", None),
    ("switch.dishwasher", "kitchen", "dishwasher", None),
    ("switch.rangehood", "kitchen", "switch", "rangehood"),
    ("switch.washing_machine", "laundry", "washer", None),
    ("switch.dryer", "laundry", "dryer", None),
    ("switch.master_bath_fan", "master_bath", "fan", "bathfan"),
    ("switch.main_bath_fan", "main_bath", "fan", "bathfan"),
    ("media_player.living_tv", "living", "tv", None),
    ("media_player.family_tv", "family_room", "tv", None),
    ("media_player.living_music", "living", "music", "mass"),
    ("media_player.kitchen_music", "kitchen", "music", "mass"),
    ("media_player.office_music", "office", "music", "mass"),
    ("vacuum.robot", "living", "vacuum", None),
    ("binary_sensor.front_door_sensor", "entryway", "door", "doorsensor"),
    ("binary_sensor.living_window", "living", "window", "windowsensor"),
    ("binary_sensor.kitchen_motion", "kitchen", "motion", "motion"),
    ("sensor.living_temperature", "living", "climate", "tempsensor"),
    ("sensor.living_humidity", "living", "climate", "humidity"),
    ("weather.home", "garden", "weather", None),
)

SCENES = (("movie_night", "film"), ("dinner_time", "dinner"), ("kids_bedtime", "kids"), ("good_morning", "morning"))
SCRIPTS = (("good_night", "good_night"), ("leaving_home", "leaving"))
TIMERS = (("oven", "timer"), ("laundry", "timer"), ("abstract", "timer"))
LISTS = (("chores", "list"), ("shopping", "list"))

KIND_KEY = {
    "light": "light",
    "ceiling": "ceiling",
    "lamp": "lamp",
    "island": "island",
    "bedside": "bedside",
    "cover": "cover",
    "climate": "climate",
    "fan": "fan",
    "lock": "lock",
    "dishwasher": "dishwasher",
    "switch": "switch",
    "washer": "washer",
    "dryer": "dryer",
    "tv": "tv",
    "music": "music",
    "vacuum": "vacuum",
    "door": "door",
    "window": "window",
    "motion": "motion",
    "weather": "weather",
    "scene": "scene",
    "timer": "timer",
    "list": "list",
    "rangehood": "rangehood",
    "bathfan": "bathfan",
    "ensuite": "ensuite",
    "humidity": "humidity",
    "doorsensor": "doorsensor",
    "windowsensor": "windowsensor",
    "tempsensor": "tempsensor",
}

QUERY_BAN = {"quel", "nani", "co", "was", "what", "wie", "quoi", "quelle", "quels", "is"}


def _blocked(lex: dict, word: str) -> bool:
    low = word.lower().strip()
    return not low or low == str(lex.get("query") or "").lower() or low in QUERY_BAN


def _aliases(words: list[str], lex: dict) -> list[str]:
    return list(dict.fromkeys(word for word in words if word and not _blocked(lex, word)))


def _title(lex: dict, ident: str, kind: str, area: str | None, extra: str | None) -> str:
    noun = lex.get(KIND_KEY.get(kind, kind), kind)
    loc = room(lex, area) if area else ""
    if extra == "ac":
        return f"{lex.get('ac', 'ac')} {loc}".strip()
    if extra == "rangehood":
        return f"{loc} {lex.get('rangehood', noun)}".strip()
    if extra == "bathfan":
        return f"{loc} {lex.get('bathfan', noun)}".strip()
    if extra == "ensuite":
        return f"{loc} {lex.get('ensuite', noun)}".strip()
    if extra == "humidity":
        return f"{loc} {lex.get('humidity', noun)}".strip()
    if extra == "motion":
        return f"{loc} {lex.get('motion', 'motion')}".strip()
    if extra in {"doorsensor", "windowsensor", "tempsensor"}:
        return f"{loc} {lex.get(extra, noun)}".strip()
    if kind == "lock" and "front_door" in ident:
        return lex.get("front_door") or f"front {lex.get('door', noun)}"
    if kind == "lock" and ("garage_entry" in ident or "garage" in ident):
        return lex.get("garage_door") or f"garage {lex.get('lock', noun)}"
    if kind == "cover" and ident.endswith("garage_door"):
        return f"{loc} {lex.get('cover', noun)}".strip()
    if kind == "island":
        return f"{loc} {noun}".strip()
    if kind in {"ceiling", "lamp", "bedside"}:
        side = extra if extra in {"left", "right"} else ""
        return " ".join(part for part in (loc, side, noun) if part)
    return f"{loc} {noun}".strip() or noun


def build_home(lex: dict) -> dict:
    code = lex["code"]
    areas = []
    entry_names = set(lex["rooms"].get("entryway") or [])
    for area_id, floor_id in AREAS:
        name = room(lex, area_id)
        aliases = list(lex["rooms"].get(area_id, [name]))
        if area_id == "hallway":
            aliases = [
                item
                for item in aliases
                if item not in entry_names and not any(entry and entry in item for entry in entry_names)
            ] or [name if name not in entry_names else "hallway"]
            name = aliases[0]
        areas.append({"id": area_id, "name": name.title() if name.isascii() else name, "floor": floor_id, "aliases": aliases})
    devices = []
    tagged = {"ac", "rangehood", "bathfan", "ensuite", "humidity", "motion", "doorsensor", "windowsensor", "tempsensor"}
    for ident, area, kind, extra in DEVICES:
        name = _title(lex, ident, kind, area, extra)
        if kind == "lock":
            tag = "front" if "front_door" in ident else "garage"
            aliases = _aliases([name, lex.get("front_door" if tag == "front" else "garage_door", name)], lex)
            device = {"id": ident, "area_id": area, "name": name, "aliases": aliases, "tags": [tag]}
        elif extra in tagged:
            tag = lex.get(extra, extra)
            aliases = _aliases([name, tag] + (["ac"] if extra == "ac" else []), lex)
            device = {"id": ident, "area_id": area, "name": name, "aliases": aliases or [name], "tags": [tag]}
        else:
            aliases = _aliases([name, lex.get(KIND_KEY.get(kind, kind), kind)], lex)
            device = {"id": ident, "area_id": area, "name": name, "aliases": aliases or [name]}
        if extra == "mass":
            device["platform"] = "music_assistant"
            device["tags"] = [lex.get("music", "music")]
        if kind in {"washer", "dryer"}:
            extras = _aliases([kind, lex.get(kind, kind)], lex)
            device["aliases"] = _aliases((device.get("aliases") or []) + extras, lex)
            device["tags"] = list(dict.fromkeys((device.get("tags") or []) + extras))
        devices.append(device)
    scenes = [{"id": ident, "name": lex.get(key, ident.replace("_", " "))} for ident, key in SCENES]
    scripts = [{"id": ident, "name": lex.get(key, ident.replace("_", " "))} for ident, key in SCRIPTS]
    timers = [{"id": ident, "name": f"{ident} {lex['timer']}"} for ident, _ in TIMERS]
    lists = [{"id": "shopping", "name": lex.get("list", "shopping")}]
    return {
        "name": f"Full Home {code}",
        "language": code,
        "difficulty": "full",
        "policy": {"preferred_climate": "climate.upper_thermostat", "timer_hints": {90: "laundry"}},
        "floors": [
            {"id": floor_id, "name": floor_word(lex, floor_id), "level": level, "aliases": list(lex["floors"].get(floor_id, [floor_id]))}
            for floor_id, level in FLOORS
        ],
        "areas": areas,
        "devices": devices,
        "scenes": scenes,
        "scripts": scripts,
        "timers": timers,
        "lists": lists,
    }
