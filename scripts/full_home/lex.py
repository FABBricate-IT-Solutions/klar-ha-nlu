"""Lexemes for realizing the full-home catalog, including de/en."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from lang_packs.extras import FAMILY, floors, pack_extras
from lang_packs.lexicons import ALL_CORES

_spec = importlib.util.spec_from_file_location("parity_lex_mod", ROOT / "scripts" / "parity" / "lex.py")
_parity = importlib.util.module_from_spec(_spec)
assert _spec.loader is not None
_spec.loader.exec_module(_parity)
parity_lex = _parity.lex_of
unique = _parity.unique

DE_ROOMS = {
    "living": ["wohnzimmer", "wohnraum"],
    "kitchen": ["kuche", "kueche"],
    "dining": ["esszimmer"],
    "master_bedroom": ["schlafzimmer"],
    "bedroom_2": ["schlafzimmer2"],
    "bedroom_3": ["schlafzimmer3"],
    "bedroom_4": ["schlafzimmer4"],
    "main_bath": ["badezimmer", "bad"],
    "master_bath": ["masterbad"],
    "hallway": ["flur", "diele"],
    "entryway": ["eingang", "diele"],
    "family_room": ["familienzimmer"],
    "laundry": ["waschkueche"],
    "powder_room": ["gaestewc"],
    "garage": ["garage"],
    "office": ["arbeitszimmer", "buero"],
    "garden": ["garten", "balkon"],
    "basement": ["keller"],
    "home": ["wohnung", "haus"],
}

EN_ROOMS = {
    "living": ["living room", "living"],
    "kitchen": ["kitchen"],
    "dining": ["dining"],
    "master_bedroom": ["bedroom", "master"],
    "bedroom_2": ["bedroom2"],
    "bedroom_3": ["bedroom3"],
    "bedroom_4": ["bedroom4"],
    "main_bath": ["bathroom", "bath"],
    "master_bath": ["ensuite"],
    "hallway": ["hallway", "hall"],
    "entryway": ["entryway", "foyer"],
    "family_room": ["family"],
    "laundry": ["laundry"],
    "powder_room": ["powder"],
    "garage": ["garage"],
    "office": ["office", "study"],
    "garden": ["garden", "yard"],
    "basement": ["basement"],
    "home": ["home", "house"],
}


def _compact(on, off, query, set_, **more) -> dict:
    lex = {
        "on": on[0],
        "on2": on[1] if len(on) > 1 else on[0],
        "off": off[0],
        "query": query[0],
        "set": set_[0],
        "open": more.get("open", ["open"])[0],
        "close": more.get("close", ["close"])[0],
        "light": more.get("light", ["light"])[0],
        "cover": more.get("cover", ["cover"])[0],
        "climate": more.get("climate", ["climate"])[0],
        "ac": more.get("ac", ["ac"])[0],
        "media": more.get("media", ["tv"])[0],
        "lock": more.get("lock", ["lock"])[0],
        "lock_v": more.get("lock_v", more.get("lock", ["lock"]))[0],
        "unlock": more.get("unlock", ["unlock"])[0],
        "fan": more.get("fan", ["fan"])[0],
        "vacuum": more.get("vacuum", ["vacuum"])[0],
        "scene": more.get("scene", ["scene"])[0],
        "timer": more.get("timer", ["timer"])[0],
        "list": more.get("list", ["list"])[0],
        "switch": more.get("switch", ["switch"])[0],
        "and": more.get("and", ["and"])[0],
        "then": (more.get("and") or ["and"])[-1],
        "all": more.get("all", ["all"])[0],
        "except": more.get("except", ["except"])[0],
        "add": more.get("add", ["add"])[0],
        "done": more.get("done", ["done"])[0],
        "pause": more.get("pause", ["pause"])[0],
        "play": more.get("play", ["play"])[0],
        "minutes": more.get("minutes", ["minutes"])[0],
        "island": more.get("island", ["island"])[0],
        "ceiling": more.get("ceiling", ["ceiling"])[0],
        "globe": more.get("globe", ["globe"])[0],
        "bedside": more.get("bedside", ["bedside"])[0],
        "device": more.get("device", ["device"])[0],
        "dishwasher": more.get("dishwasher", ["dishwasher"])[0],
        "washer": more.get("washer", ["washer"])[0],
        "dryer": more.get("dryer", ["dryer"])[0],
        "tv": more.get("tv", ["tv"])[0],
        "lamp": (more.get("lamp") or ["lamp"])[0],
        "good_night": more.get("good_night", ["night"])[0],
        "leaving": more.get("leaving", ["leaving"])[0],
        "film": more.get("film", ["film"])[0],
        "by": more.get("by", ["by"])[0],
        "radio": more.get("radio", ["radio"])[0],
        "album": more.get("album", ["album"])[0],
        "music": more.get("music", ["music"])[0],
        "next": more.get("next", ["next"])[0],
        "prev": more.get("prev", ["previous"])[0],
        "track": more.get("track", ["track"])[0],
        "clock": more.get("clock", ["time"])[0],
        "weather": more.get("weather", ["weather"])[0],
        "door": more.get("door", ["door"])[0],
        "window": more.get("window", ["window"])[0],
        "motion": more.get("motion", ["motion"])[0],
        "humidity": more.get("humidity", ["humidity"])[0],
        "doorsensor": more.get("doorsensor", ["doorsensor"])[0],
        "windowsensor": more.get("windowsensor", ["windowsensor"])[0],
        "tempsensor": more.get("tempsensor", ["tempsensor"])[0],
        "rangehood": more.get("rangehood", more.get("switch", ["switch"]))[0],
        "bathfan": more.get("bathfan", more.get("fan", ["fan"]))[0],
        "ensuite": more.get("ensuite", more.get("light", ["light"]))[0],
        "dinner": more.get("dinner", more.get("scene", ["scene"]))[0],
        "morning": more.get("morning", more.get("scene", ["scene"]))[0],
        "kids": more.get("kids", more.get("good_night", ["night"]))[0],
        "colors": more.get("colors", {}),
        "rooms": more.get("rooms", {}),
        "floors": more.get("floors", {"upper": ["upper"], "ground": ["ground"], "basement": ["basement"]}),
        "order": more.get("order", "vnr"),
    }
    lex.update({key: val for key, val in more.items() if key not in lex})
    return lex


def lex_de() -> dict:
    lex = _compact(
        on=["mach", "an", "schalte"],
        off=["aus", "machaus"],
        query=["wie", "was", "status"],
        set_=["stelle", "setz"],
        open=["oeffne", "auf"],
        close=["schliesse", "zu"],
        light=["licht", "lampe"],
        cover=["rollo", "jalousie"],
        climate=["heizung", "temperatur"],
        ac=["klima"],
        media=["tv", "fernseher"],
        lock=["tuer", "schloss"],
        lock_v=["schliess"],
        unlock=["oeffne"],
        fan=["luefter"],
        vacuum=["staubsauger"],
        scene=["szene"],
        timer=["timer"],
        list=["liste", "einkauf"],
        switch=["schalter"],
        and_=["und", "dann"],
        all=["alle"],
        except_=["ausser", "ohne"],
        add=["setz"],
        done=["erledigt"],
        pause=["pause"],
        play=["spiel", "spiele"],
        minutes=["minuten"],
        island=["insel"],
        ceiling=["decke", "deckenlampe"],
        globe=["kugel"],
        bedside=["nachttisch"],
        device=["geraet"],
        dishwasher=["spuelmaschine"],
        washer=["waschmaschine"],
        dryer=["trockner"],
        tv=["tv"],
        lamp=["lampe"],
        good_night=["nacht"],
        leaving=["verlassen"],
        film=["filmabend"],
        by=["von"],
        radio=["radio"],
        album=["album"],
        music=["musik"],
        clock=["uhrzeit"],
        weather=["wetter"],
        door=["tuer"],
        window=["fenster"],
        motion=["bewegung"],
        humidity=["luftfeuchtigkeit"],
        doorsensor=["tursensor"],
        windowsensor=["fenstersensor"],
        tempsensor=["raumfuehler"],
        rangehood=["dunstabzug"],
        bathfan=["badluefter"],
        ensuite=["anschlusslicht"],
        dinner=["essen"],
        morning=["morgen"],
        kids=["kinder"],
        colors={"red": "rot", "blue": "blau", "green": "gruen", "yellow": "gelb", "white": "weiss", "black": "schwarz"},
        rooms={key: list(vals) for key, vals in DE_ROOMS.items()},
        floors={"upper": ["oben", "obergeschoss"], "ground": ["unten", "erdgeschoss"], "basement": ["keller"]},
        order="nvr",
    )
    lex["code"] = "de"
    lex["and"] = "und"
    lex["except"] = "ausser"
    lex["front_door"] = "Haustür"
    lex["garage_door"] = "Garagentor"
    return lex


def lex_en() -> dict:
    lex = _compact(
        on=["turn on", "switch on"],
        off=["turn off", "switch off"],
        query=["what", "is"],
        set_=["set", "change"],
        open=["open"],
        close=["close"],
        light=["light", "lights"],
        cover=["blinds"],
        climate=["thermostat", "temperature"],
        ac=["ac"],
        media=["tv"],
        lock=["lock", "door"],
        lock_v=["lock"],
        unlock=["unlock"],
        fan=["fan"],
        vacuum=["vacuum"],
        scene=["scene"],
        timer=["timer"],
        list=["list"],
        switch=["switch"],
        and_=["and", "then"],
        all=["all"],
        except_=["except"],
        add=["add"],
        done=["done"],
        pause=["pause"],
        play=["play"],
        minutes=["minutes"],
        island=["island"],
        ceiling=["ceiling"],
        globe=["globe"],
        bedside=["bedside"],
        device=["device"],
        dishwasher=["dishwasher"],
        washer=["washer"],
        dryer=["dryer"],
        tv=["tv"],
        lamp=["lamp"],
        good_night=["night"],
        leaving=["leaving"],
        film=["movie"],
        by=["by"],
        radio=["radio"],
        album=["album"],
        music=["music"],
        clock=["time"],
        weather=["weather"],
        door=["door"],
        window=["window"],
        motion=["motion"],
        humidity=["humidity"],
        doorsensor=["doorsensor"],
        windowsensor=["windowsensor"],
        tempsensor=["roomsensor"],
        rangehood=["rangehood"],
        bathfan=["bathfan"],
        ensuite=["ensuite"],
        dinner=["dinner"],
        morning=["morning"],
        kids=["kids"],
        colors={c: c for c in ("red", "blue", "green", "yellow", "white", "black")},
        rooms={key: list(vals) for key, vals in EN_ROOMS.items()},
        floors={"upper": ["upstairs", "upper"], "ground": ["downstairs", "ground"], "basement": ["basement"]},
        order="en",
    )
    lex["code"] = "en"
    lex["and"] = "and"
    lex["except"] = "except"
    lex["front_door"] = "front door"
    lex["garage_door"] = "garage door"
    return lex


EXTRA_ROOMS = {
    "office": "office",
    "garden": "garden",
    "basement": "basement",
}


def _extend_rooms(lex: dict) -> dict:
    rooms = lex.setdefault("rooms", {})
    living = (rooms.get("living") or rooms.get("wohnzimmer") or ["living"])[0]
    kitchen = (rooms.get("kitchen") or rooms.get("kuche") or [living])[0]
    bed = (rooms.get("master_bedroom") or rooms.get("schlafzimmer") or [living])[0]
    code = lex.get("code", "")
    german = code == "de" or code.startswith("de-")
    rooms.setdefault("office", rooms.get("arbeitszimmer") or (["arbeitszimmer"] if german else ["office", "study"]))
    rooms.setdefault("garden", rooms.get("balkon") or rooms.get("balcony") or (["garten"] if german else ["garden"]))
    rooms.setdefault("basement", ["keller"] if german else {
        "fr": ["cave"],
        "cs": ["sklep"],
        "ja": ["chika"],
    }.get(code, ["basement"]))
    rooms.setdefault("home", rooms.get("wohnung") or [living])
    extra = FAMILY.get(lex.get("code"), {})
    for key, names in extra.items():
        rooms.setdefault(key, []).extend(names)
    if rooms.get("office"):
        distinct = [name for name in rooms["office"] if name not in {living, "lounge", "wohnzimmer"}]
        rooms["office"] = distinct or (["arbeitszimmer"] if german else ["office"])
    if rooms.get("dining"):
        distinct = [name for name in rooms["dining"] if name not in {living, kitchen, "lounge", "wohnzimmer"}]
        rooms["dining"] = distinct or [f"{living}ess"]
    rooms.setdefault("bedroom_2", [f"{bed.replace(' ', '')}2"])
    rooms.setdefault("bedroom_3", [f"{bed.replace(' ', '')}3"])
    rooms.setdefault("bedroom_4", [f"{bed.replace(' ', '')}4"])
    entry0 = (rooms.get("entryway") or extra.get("entryway") or [living])[0]
    hall_native = {
        "fr": "couloir", "cs": "chodba", "sk": "chodba", "pl": "korytarz", "ja": "roka", "nl": "overloop",
        "es": "pasillo", "it": "corridoio", "pt": "corredor", "fi": "kaytava", "ko": "bokdo",
        "zh-CN": "zoulang", "zh-TW": "zoulang", "zh-HK": "zoulang", "tr": "koridor", "ar": "ممر",
    }.get(code)
    hall = [name for name in (rooms.get("hallway") or []) if name and name != entry0 and entry0 not in name]
    if hall_native:
        hall = [hall_native] + [name for name in hall if name != hall_native]
    rooms["hallway"] = unique(hall or ([hall_native] if hall_native else ["hallway"]))
    lex["rooms"] = {key: unique(vals) for key, vals in rooms.items()}
    floors_map = dict(lex.get("floors") or floors(lex.get("code", "en")))
    floors_map.setdefault("basement", ["basement", "keller"])
    lex["floors"] = floors_map
    extras = pack_extras(lex.get("code", ""))
    for key in ("play", "pause", "radio", "album", "music", "clock", "weather", "door", "dock"):
        if key in extras and key not in lex:
            lex[key] = extras[key][0]
    play = extras.get("play") or []
    on = lex.get("on")
    if play and play[0] != on:
        lex["play"] = play[0]
    elif lex.get("play") in {None, "", on}:
        lex["play"] = "play"
    dock = extras.get("dock") or []
    if dock:
        lex["dock"] = dock[0]
        lex["vac_home"] = dock[-1]
    lex.setdefault("by", "by")
    lex.setdefault("radio", "radio")
    lex.setdefault("album", "album")
    lex.setdefault("music", lex.get("media"))
    lex.setdefault("clock", "clock")
    lex.setdefault("weather", "weather")
    lex.setdefault("ac", "ac")
    lex.setdefault("window", lex.get("cover"))
    _fill_house(lex)
    lex.setdefault("ensuite", "ensuite")
    lex.setdefault("order", "vnr")
    return lex


NIGHTISH = {
    "nuit", "yoru", "night", "vecer", "nacht", "noche", "notte", "noite", "yo", "wanshang",
    "bam", "noc", "nat", "natt", "nit", "noapte", "ejjel", "gece", "malam", "dem", "raat", "nos",
}

HOUSE = {
    "fr": ("mouvement", "humidite", "sonde", "capteurporte", "capteurfenetre", "extracteur", "hotte", "diner", "matin", "enfants", "cinema"),
    "cs": ("pohyb", "vlhkost", "cidlo", "dveresenzor", "oknosenzor", "koupelventilator", "digestor", "vecere", "rano", "deti", "kino"),
    "ja": ("kando", "shitsudo", "ondo", "doasensa", "madosensa", "yokusovent", "renjifud", "bangohan", "asa", "kodomo", "eiga"),
    "nl": ("beweging", "vocht", "kamersensor", "deursensor", "raamsensor", "badventilator", "afzuigkap", "diner", "ochtend", "kinderen", "filmavond"),
    "es": ("movimiento", "humedad", "sonda", "sensorpuerta", "sensorventana", "extractor", "campana", "cena", "manana", "ninos", "cine"),
    "zh-CN": ("yundong", "shidu", "wenshidu", "mensuoqi", "chuangsuoqi", "yushifengshan", "youyanji", "wanfan", "zaoshang", "haizi", "dianyingye"),
}


def _fill_house(lex: dict) -> None:
    query = str(lex.get("query") or "").lower()
    scene = lex.get("scene") or "scene"
    night = lex.get("good_night") or "night"
    row = HOUSE.get(lex.get("code", ""), ("motion", "humidity", "tempsensor", "doorsensor", "windowsensor", f"{lex.get('fan', 'fan')}bad", "rangehood", "dinner", "morning", "kids", "cinema"))
    keys = ("motion", "humidity", "tempsensor", "doorsensor", "windowsensor", "bathfan", "rangehood", "dinner", "morning", "kids", "film")
    for key, word in zip(keys, row):
        cur = lex.get(key)
        collide = cur in {None, "", query, scene, night, lex.get("climate"), lex.get("fan"), lex.get("door"), lex.get("window"), lex.get("cover"), lex.get("switch")}
        if key == "film" and cur and cur not in NIGHTISH and cur != night:
            continue
        if collide or (key == "kids" and cur in NIGHTISH) or (key == "film" and (not cur or cur in NIGHTISH)):
            lex[key] = word
    lex.setdefault("front_door", f"front {lex.get('door', 'door')}")
    lex.setdefault("garage_door", f"garage {lex.get('door', lex.get('lock', 'lock'))}")
    lex.setdefault("dock", "dock")
    bedside = str(lex.get("bedside") or "").lower()
    if bedside in {query, night} or bedside in NIGHTISH:
        lex["bedside"] = "bedside"


def all_lexes() -> list[dict]:
    out = [lex_de(), lex_en()]
    for core in ALL_CORES:
        lex = _extend_rooms(parity_lex(core))
        lex.setdefault("code", core["code"])
        words = core.get("w") or {}
        if words.get("play") and words["play"][0] != lex.get("on"):
            lex["play"] = words["play"][0]
        if words.get("pause"):
            lex["pause"] = words["pause"][0]
        # media.rs only strips spiel/play/listen; use play so search_query stays clean
        if core["code"] in {"fr", "cs", "ja"}:
            lex["play"] = "play"
        climate = words.get("climate") or []
        if climate:
            lex["ac"] = climate[-1]
        media_words = words.get("media") or []
        skip_media = {"tv", "tele", "televize", "terebi", "dianshi", "tibi", "thiwi", "televizors", "media", "volume"}
        for cand in media_words:
            if cand not in skip_media:
                lex["music"] = cand
                break
        if words.get("add"):
            lex["add"] = words["add"][0]
        if words.get("done"):
            lex["done"] = words["done"][0]
        if words.get("lock_v"):
            lex["lock_v"] = words["lock_v"][0]
        if words.get("unlock"):
            lex["unlock"] = words["unlock"][0]
        if words.get("minutes"):
            lex["minutes"] = words["minutes"][0]
        dryer = {"fr": "sechelinge", "cs": "susicka", "ja": "kansouki"}.get(core["code"], "dryer")
        if lex.get("dryer") in {None, "", lex.get("washer"), lex.get("light")}:
            lex["dryer"] = dryer
        if lex.get("ensuite") in {None, "", lex.get("light")}:
            lex["ensuite"] = "ensuite"
        lex.setdefault("next", "next")
        lex.setdefault("prev", "previous")
        lex.setdefault("track", "track")
        chat = core.get("chat") or {}
        if chat.get("weather"):
            lex["weather"] = chat["weather"][0]
        house = pack_extras(core["code"])
        if house.get("clock"):
            lex["clock"] = house["clock"][0]
        climate = lex.get("climate") or ""
        house_weather = (house.get("weather") or [""])[0]
        if house_weather and climate and climate not in house_weather:
            lex["weather"] = house_weather
        elif not lex.get("weather"):
            lex["weather"] = "weather"
        out.append(lex)
    return out


def room(lex: dict, area: str) -> str:
    names = lex["rooms"].get(area) or [area.replace("_", " ")]
    return names[0]


def floor_word(lex: dict, floor_id: str) -> str:
    return (lex["floors"].get(floor_id) or [floor_id])[0]


def color_word(lex: dict, color: str) -> str:
    return lex.get("colors", {}).get(color, color)
