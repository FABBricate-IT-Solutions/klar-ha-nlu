"""Turn a compact lexicon into a full LanguagePack dict.

Spoken lists come from the locale lexicon (or stay empty). Do not inject
German fillers (bitte, die, der, das), home-graph tokens (aufgabenliste,
klimaanlage, insel), or speech scaffolding (leuchte, filmabend) unless
that locale's lexicon actually lists them. de-CH/de-AT may keep German.
"""

from __future__ import annotations

from lang_packs.calendar_lex import calendar_for
from lang_packs.convo import with_warm_white
from lang_packs.extras import pack_extras
from lang_packs.native_apply import apply_native, is_script_pack
from lang_packs.voices import normalize_personality

ROOMS = (
    ("wohnzimmer", "living"), ("esszimmer", "dining"), ("schlafzimmer", "bedroom"),
    ("kuche", "kitchen"), ("badezimmer", "bathroom"), ("arbeitszimmer", "office"),
    ("flur", "hallway"), ("balkon", "balcony"), ("wohnung", "home"),
)

def expand(core: dict) -> dict:
    apply_native(core)
    w = core["w"]
    for key, val in pack_extras(core["code"]).items():
        w.setdefault(key, val)
    cal = calendar_for(core["code"])
    w.setdefault("calendar", list(cal["nouns"]))
    w.setdefault("calendar_query", list(cal["query"]))
    w.setdefault("calendar_create", list(cal["create"]))
    w.setdefault("calendar_today", list(cal["today"]))
    w.setdefault("calendar_tomorrow", list(cal["tomorrow"]))
    w.setdefault("calendar_when", list(cal["when"]))
    w.setdefault("calendar_delete", list(cal.get("delete") or []))
    w.setdefault("calendar_move", list(cal.get("move") or []))
    w.setdefault("resume", list(cal["resume"]))
    speech = dict(core["speech"])
    for key, value in cal["speech"].items():
        if key == "unknown":
            speech["unknown"] = value
        else:
            speech.setdefault(key, value)
    chat = dict(core["chat"])
    chat["news_intro"] = speech["unknown"]
    colors = with_warm_white(core["code"], list(core.get("colors") or default_colors()))
    numbers = list(core.get("numbers", []))
    rooms = list(core.get("rooms", []))
    synonyms = [(native, canon) for native, canon in rooms]
    synonyms += list(core.get("synonyms", []))
    synonyms += device_synonym_rows(w, core["code"])
    on = w["on"]
    off = w["off"]
    light = w["light"]
    cover = distinct_from(w["cover"], light, "cover")
    climate = distinct_from(w["climate"], light, "climate")
    media = distinct_from(w["media"], light, "media")
    lock = distinct_from(w["lock"], light, "lock")
    door = distinct_from(w["door"], light, "door")
    timer = distinct_from(w["timer"], light, "timer")
    lst = distinct_from(w["list"], light, "list")
    fan = distinct_from(w["fan"], light, "fan")
    vacuum = distinct_from(w["vacuum"], light, "vacuum")
    scene = distinct_from(w["scene"], light, "scene")
    query = w["query"]
    sett = w["set"]
    opn = w["open"]
    close = w["close"]
    verbs = []
    extra_on = ["active"] if not is_script_pack(core) else []
    extra_play = ["play"] if not is_script_pack(core) else []
    extra_next = ["next"] if not is_script_pack(core) else []
    for word in unique(on + extra_on):
        verbs.append((word, "On"))
    for word in off:
        verbs.append((word, "Off"))
    for word in opn:
        verbs.append((word, "Open"))
    for word in close:
        verbs.append((word, "Close"))
    for word in query:
        verbs.append((word, "Query"))
    for word in sett:
        verbs.append((word, "Set"))
    for word in w.get("lock_v", lock):
        verbs.append((word, "Lock"))
    for word in w.get("unlock", []):
        verbs.append((word, "Unlock"))
    for word in timer:
        verbs.append((word, "Timer"))
    for word in lst:
        verbs.append((word, "List"))
    for word in w.get("add", []):
        verbs.append((word, "Add"))
    for word in lexemes(w.get("done", [])):
        verbs.append((word, "ListComplete"))
    for word in fan:
        verbs.append((word, "FanNoun"))
    for word in vacuum:
        verbs.append((word, "VacuumNoun"))
    for word in unique(list(w.get("dock") or [])):
        verbs.append((word, "Dock"))
    for word in scene:
        verbs.append((word, "Scene"))
    for word, _canon in colors:
        verbs.append((word, "Color"))
    for word in w.get("percent", []):
        verbs.append((word, "Percent"))
    for word in unique(list(w.get("dim") or []) + list(w.get("medium") or [])):
        verbs.append((word, "Dim"))
    for word in w.get("stop", off[:1]):
        verbs.append((word, "Stop"))
    for word in unique(list(w.get("play") or []) + extra_play):
        verbs.append((word, "Play"))
    for word in unique(list(w.get("next") or []) + extra_next):
        verbs.append((word, "Next"))
    for word in w.get("pause", []):
        verbs.append((word, "Pause"))
    verbs.extend(core.get("extra_verbs", []))

    domain = []
    for word in unique(light):
        domain.append((word, "light"))
    for word in climate:
        domain.append((word, "climate"))
    for word in media:
        domain.append((word, "media_player"))
    for word in lock:
        domain.append((word, "lock"))
    for word in timer:
        domain.append((word, "timer"))
    for word in lst:
        domain.append((word, "todo"))
    for word in unique(w.get("calendar", [])):
        domain.append((word, "calendar"))
    for word in vacuum:
        domain.append((word, "vacuum"))
    for word in fan:
        domain.append((word, "fan"))
    for word in cover:
        domain.append((word, "cover"))
    for word in scene:
        domain.append((word, "scene"))
    for word in unique(w.get("switch", [])):
        domain.append((word, "switch"))

    kitchen = w.get("kitchen", rooms_named(rooms, "kitchen"))
    generic = unique(
        light + cover + climate + media + lock + door + timer + lst + fan + vacuum + scene + [r[0] for r in rooms] + kitchen
    )
    talk_q = query + w.get("status", [])
    return {
        "code": core["code"],
        "mod": core["mod"],
        "const": core.get("const", core["mod"].title().replace("_", "")),
        "native": core["native"],
        "script": core.get("script", "Latn"),
        "variants": core["variants"],
        "path": f"packs::{core['mod']}::PACK",
        "verbs": verbs,
        "speech": speech,
        "room_names": [(canon, native) for native, canon in rooms[:3]],
        "loc_der_rooms": [r[0] for r in rooms[:2]],
        "personality": normalize_personality(core.get("personality")),
        "talk": {
            "fillers": unique(w["fillers"]),
            "action_keep": unique(on[:2] + off[:2] + opn[:1] + close[:1]),
            "conjunctions": unique(w["and"] + ["then"]),
            "particles": unique(
                list(w.get("on_particles") or [])
                + list(w.get("off_particles") or [])
                + [word for word in on[:2] + off[:2] if len(word) <= 4]
            ),
            "affirm": w["yes"],
            "or_words": w["or"],
            "except_words": unique((w.get("except") or pack_extras(core["code"]).get("except", [])) + ["except"]),
            "all_words": unique(w["all"]),
            "query_hint": talk_q,
            "question_starts": talk_q,
            "question_words": talk_q,
            "correction": w.get("wrong", []),
            "clarify_pick": w["yes"][:2],
        },
        "nouns": {
            "light_nouns": light,
            "light_singular": unique(light[:3]),
            "light_plural": unique(light[3:]),
            "cover_nouns": cover,
            "curtain_nouns": cover[:2],
            "fan_nouns": fan,
            "climate_nouns": unique(climate),
            "media_nouns": media,
            "lock_nouns": lock,
            "door_nouns": door,
            "garage_words": w.get("garage", []),
            "garage_cover": door[:1],
            "timer_nouns": timer,
            "list_nouns": unique(lst),
            "calendar_nouns": unique(w.get("calendar", [])),
            "vacuum_nouns": vacuum,
            "scene_nouns": scene,
            "script_words": w.get("script", scene[:1]),
            "switch_plural": unique([word for word in w.get("switch", []) if word not in (w.get("device") or [])]),
            "device_side": unique(light + fan + media[:1] + cover[:1] + lock[:1] + scene[:1] + w.get("washer", []) + w.get("dryer", [])),
            "named_device": unique(w.get("named", []) + w.get("globe", [])),
        },
        "fixtures": {
            "island": w.get("island", []),
            "ceiling": w.get("ceiling", []),
            "lamp_fixture": unique(w.get("lamp", []) or light[1:2]),
            "pendant": w.get("pendant", []),
            "bedside": w.get("bedside", []),
            "left": w.get("left", []),
            "right": w.get("right", []),
            "sides": w.get("left", []) + w.get("right", []),
            "clarify_trigger": [],
            "clarify_pairs": [],
            "singular_lamp": [],
            "singular_lamp_block": unique(light[:2]),
        },
            "fixture_aliases": fixture_alias_rows(w, core["code"]),
        "cues": {
            "power_words": unique(on + off + w.get("stop", [])),
            "command_hedges": w.get("hedge", []),
            "skip_light": unique(
                [
                    word
                    for word in fan
                    + media[:1]
                    + w.get("switch", [])
                    + w.get("washer", [])
                    + w.get("dishwasher", [])
                    + w.get("dryer", [])
                    + w.get("tv", [])
                    if word not in light
                ]
            ),
            "laundry_area": w.get("laundry", []),
            "laundry_machines": unique(w.get("washer", []) + w.get("dryer", [])),
            "kitchen": kitchen,
            "open_words": opn,
            "close_words": close,
            "roll_close": close[:1] + w.get("down", []),
            "unlock_follow": opn[:1] + off[:1],
            "cover_open_follow": opn[:1],
            "garage_lock_block": lock,
            "on_words": unique(on[:2] + w.get("on_particles", [])),
            "off_words": unique(off[:2] + w.get("off_particles", [])),
            "scene_named": unique(w.get("scenes", []) + w.get("good_night", []) + w.get("leaving", []) + scene_lexemes(core)),
            "temp_query": climate[:3],
            "timer_query": query[:3],
            "brightness": w.get("bright", []),
            "start_words": on[:1],
            "replay_on_off": unique(on[:1] + off[:1]),
            "replay_off": off[:1],
            "sensor_words": unique(w.get("sensor", []) + ["sensor"]),
            "lock_verbs": unique(list(w.get("lock_v") or []) + lock + w.get("unlock", [])),
            "entry_words": w.get("entry", []),
            "oven": unique(w.get("oven", []) + ["oven"]),
            "laundry_timer": unique(w.get("laundry", [])),
            "illuminate": [],
            "list_down": lst,
            "chores": unique(lst[-1:]),
            "weak_scene": w["fillers"][:3],
            "timer_cancel": unique(off[:1] + w.get("stop", [])),
            "timer_pause": w.get("pause", []),
            "timer_add": w.get("add", []),
            "list_complete": lexemes(w.get("done", [])),
            "playback_resume": unique(list(w.get("play") or []) + list(w.get("resume") or [])),
            "calendar_query": unique(w.get("calendar_query", [])),
            "calendar_create": unique(w.get("calendar_create", [])),
            "calendar_today": unique(w.get("calendar_today", [])),
            "calendar_tomorrow": unique(w.get("calendar_tomorrow", [])),
            "calendar_when": unique(w.get("calendar_when", [])),
            "calendar_delete": unique(w.get("calendar_delete", [])),
            "calendar_move": unique(w.get("calendar_move", [])),
            "vacuum_start": on[:1],
            "hours": w.get("hours", []),
            "minutes": w.get("minutes", []),
            "seconds": w.get("seconds", []),
            "list_skip": unique(
                w["fillers"][:4]
                + w.get("add", [])
                + lexemes(w.get("done", []))
                + ["add", "done", "fuege", "hinzu", "erledigt", "fertig"]
            ),
            "shopping_names": lst[1:2] or lst[:1],
            "status_words": w.get("status", query[:1]),
            "window_words": w.get("window", cover[:1]),
            "open_close": unique(opn[:1] + close[:1]),
            "laundry_hint": w.get("laundry", []),
            "bare_switch": unique(w.get("washer", []) + w.get("dryer", []) + ["machine", "appliance"]),
            "outlet_words": w.get("outlet", []),
            "tv_words": media[:1],
            "climate_cool": climate[-1:],
            "climate_heat": climate[:1],
            "role_light": light[:2],
            "role_climate": climate[:2],
            "role_media": media[:2],
            "role_fan": fan[:1],
            "generic": generic,
            "room_level": light[:1],
            "extra_device_nouns": unique(
                w.get("device", [])
                + w.get("switch", [])
                + w.get("washer", [])
                + w.get("dishwasher", [])
                + w.get("dryer", [])
                + w.get("tv", [])
                + w.get("globe", [])
                + w.get("named", [])
            ),
            "synonym_pairs": synonyms,
            "scene_synonyms": scene_synonym_rows(core, w),
            "article_one": w.get("one", []),
            "strip_pairs": core.get("strip_pairs", []),
        },
        "maps": {
            "domain_map": domain,
            "colors": colors,
            "numbers": numbers,
            "number_style": core.get("number_style", "ListedOnly"),
            "room_index_nouns": [r[0] for r in rooms if r[1] in ("bedroom", "bathroom")] or [r[0] for r in rooms[:1]],
        },
        "chat": chat,
        "household": household_from(core, w, speech),
        "smoke": smoke_rows(core, on) + calendar_smokes(cal) + music_smokes(w),
    }


def household_from(core: dict, w: dict, speech: dict) -> dict:
    extra = pack_extras(core["code"])
    query = w.get("query") or []
    sett = w.get("set") or w.get("on") or []
    off = w.get("off") or []
    climate = w.get("climate") or []
    timer = w.get("timer") or []
    door = w.get("door") or w.get("light") or []
    q0, s0, o0 = (query[:1] or [""])[0], (sett[:1] or [""])[0], (off[:1] or [""])[0]
    climate0, timer0, door0 = (climate[-1:] or [""])[0], (timer[:1] or [""])[0], (door[:1] or [""])[0]
    add = lambda key, extra_word: unique(w.get(key, []) + extra.get(key, []) + ([extra_word] if extra_word else []))
    teach = add("teach", f"{s0} {door0} " if s0 and door0 else "")
    explain = add("explain", f"{q0} {o0} {s0}".strip() if q0 and o0 and s0 else "")
    undo = add("undo", f"{o0} {door0}".strip() if o0 and door0 else "")
    clock = add("clock", f"{q0} {timer0}".strip() if q0 and timer0 else "")
    weather = add("weather", f"{q0} {climate0}".strip() if q0 and climate0 else "")
    return {
        "teach": teach,
        "explain": explain,
        "undo": undo,
        "clock": clock,
        "weather": weather,
        "clock_skip": unique(w.get("clock_skip", []) + extra.get("clock_skip", []) + timer[:1]),
        "heard_nothing": extra.get("heard_nothing") or speech.get("unknown", ""),
        "heard": extra.get("heard") or "{text}",
        "executed": extra.get("executed") or "{names}",
        "asked_risky": extra.get("asked_risky") or speech.get("confirm", ""),
        "unclear_device": extra.get("unclear_device") or speech.get("need_which", ""),
        "stopped": extra.get("stopped") or "{reason}",
        "no_match": extra.get("no_match") or speech.get("unknown", ""),
        "was_chat": extra.get("was_chat") or speech.get("done", ""),
        "decision": extra.get("decision") or "{decision}",
        "in_area": extra.get("in_area") or "{area}",
        "nothing_undo": extra.get("nothing_undo") or speech.get("unknown", ""),
        "teach_which": extra.get("teach_which") or speech.get("need_which", ""),
        "teach_invalid": extra.get("teach_invalid") or speech.get("unknown", ""),
        "teach_ok": extra.get("teach_ok") or speech.get("done", ""),
        "clock_ok": extra.get("clock_ok") or "{time}",
        "clock_missing": extra.get("clock_missing") or speech.get("unknown", ""),
        "no_weather": extra.get("no_weather") or speech.get("unknown", ""),
    }


def scene_lexemes(core: dict) -> list[str]:
    words: list[str] = []
    for src, dst in core.get("scene_synonyms", []):
        words.append(src)
        words.append(dst)
    return words


def smoke_rows(core: dict, on: list[str]) -> list[tuple[str, str]]:
    del on
    return list(core.get("smoke", []))


def calendar_smokes(cal: dict) -> list[tuple[str, str]]:
    rows = []
    if cal.get("list_smoke"):
        rows.append((cal["list_smoke"], "KlarGetCalendarEvents"))
    if cal.get("create_smoke"):
        rows.append((cal["create_smoke"], "KlarCreateCalendarEvent"))
    if cal.get("delete_smoke"):
        rows.append((cal["delete_smoke"], "KlarDeleteCalendarEvent"))
    if cal.get("move_smoke"):
        rows.append((cal["move_smoke"], "KlarMoveCalendarEvent"))
    return rows


def music_smokes(w: dict) -> list[tuple[str, str]]:
    play = (w.get("play") or ["play"])[0]
    search = "depeche"
    return [(f"{play} {search}", "MassPlayMedia")]


DEVICE_STEMS = {
    "island": ["island", "insel"],
    "ceiling": ["ceiling", "decke", "deckenlampe"],
    "globe": ["globe", "kugel"],
    "bedside": ["bedside", "nacht", "nachttisch"],
    "pendant": ["pendant", "pendel"],
    "dishwasher": ["dishwasher", "spuelmaschine", "spulmaschine"],
    "washer": ["washer", "waschmaschine", "washing"],
    "tv": ["tv", "fernseher"],
    "dryer": ["dryer", "trockner"],
    "left": ["left", "links"],
    "right": ["right", "rechts"],
}
GERMAN_HOME = {"aufgabenliste", "klimaanlage", "insel", "filmabend", "verlassen"}
GERMAN_OK = {"de", "en", "de-CH", "de-AT"}


def device_stems(code: str, stems: list[str]) -> list[str]:
    if code in GERMAN_OK:
        return stems
    return [stem for stem in stems if stem not in GERMAN_HOME]


def fixture_alias_rows(w: dict, code: str) -> list[tuple[str, list[str]]]:
    rows: list[tuple[str, list[str]]] = []
    for key, stems in DEVICE_STEMS.items():
        for native in w.get(key, []):
            rows.append((native, unique([native, *device_stems(code, stems)])))
    return rows


def device_synonym_rows(w: dict, code: str) -> list[tuple[str, str]]:
    rows: list[tuple[str, str]] = []
    for key, stems in DEVICE_STEMS.items():
        for native in unique(w.get(key, [])):
            for stem in device_stems(code, stems):
                if native and native != stem:
                    rows.append((native, stem))
    return rows


def scene_synonym_rows(core: dict, w: dict) -> list[tuple[str, str]]:
    rows = [(src, dst) for src, dst in core.get("scene_synonyms", []) if src and dst]
    for native in w.get("scenes", []) + w.get("good_night", []) + w.get("leaving", []):
        if native:
            rows.append((native, native))
    rows.extend(
        [
            ("good_morning", "good_morning"),
            ("leaving_home", "leaving_home"),
            ("movie_night", "movie_night"),
            ("dinner_time", "dinner_time"),
        ]
    )
    return rows


def rooms_named(rooms: list[tuple[str, str]], canon: str) -> list[str]:
    return [native for native, name in rooms if name == canon]


def default_colors() -> list[tuple[str, str]]:
    return [
        ("red", "red"),
        ("blue", "blue"),
        ("green", "green"),
        ("yellow", "yellow"),
        ("orange", "orange"),
        ("pink", "pink"),
        ("black", "black"),
        ("white", "white"),
        ("purple", "purple"),
    ]


def distinct_from(words: list[str], forbidden: list[str], field: str) -> list[str]:
    blocked = set(forbidden)
    kept = [word for word in words if word and word not in blocked]
    if not kept:
        raise ValueError(f"{field} collides with light nouns: {words}")
    return kept


def unique(values: list[str]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for item in values:
        if item and item not in seen:
            seen.add(item)
            out.append(item)
    return out


def lexemes(values: list[str] | None) -> list[str]:
    return unique(part for word in (values or []) for part in word.split())
