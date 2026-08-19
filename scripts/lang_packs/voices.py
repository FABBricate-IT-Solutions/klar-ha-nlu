"""Voice bible — contract for every spoken personality and locale.

Lexicon authors and handwritten de/en packs follow this. generate.py only
emits what lexicons provide. It never invents voices or overwrites de/en.
"""

from __future__ import annotations

PERSONALITY_KEYS = (
    "butler",
    "locker",
    "fuersorglich",
    "party",
    "grantig",
    "sarkastisch",
    "pirat",
    "hippie",
    "gollum",
)

# Flavor lives in the sentence. One of the three variants may be "" (body only).
VARIANT_COUNT = 3

# Ban these as stamps, translations, or close cousins in every language.
BANNED = (
    "zur kenntnis genommen",
    "notiert",
    "vermerkt",
    "besorgt",
    "soweit gemeldet",
    "duly noted",
    "taken into account",
    "noted",
    "enregistre",
    "enregistré",
    "pris en note",
    "c'est note",
    "c'est noté",
    "fehlinterpretation",
    "genoteerd",
    "ik check",
)

# Prefixes that may appear at most once among the three butler variants.
BUTLER_ONCE = ("sehr wohl", "very well", "très bien", "tres bien")

VOICES = {
    "default": {
        "role": "plain spoken confirmation, complete sentence, no slang, no stamp",
        "pattern": "full sentence only — default has no personality slice",
    },
    "butler": {
        "role": "polite spoken help, calm Sie/you-formal where the language has it",
        "do": "glad to help, sometimes just the sentence",
        "dont": "court protocol, forms, bureaucracy, same opener every time",
        "pattern": ["courtesy once", "gladly / with pleasure", "body only"],
        "de": ["Sehr wohl. ", "Gern. ", ""],
        "en": ["Very well. ", "Gladly. ", ""],
        "fr": ["Très bien. ", "Avec plaisir. ", ""],
    },
    "locker": {
        "role": "buddy-like, short, easy",
        "do": "casual confirmations",
        "dont": "office stamps, long asides",
        "pattern": ["got it", "done", "body only"],
        "de": ["Geht klar. ", "Passt. ", ""],
        "en": ["Got it. ", "Done. ", ""],
        "fr": ["C'est bon. ", "Carré. ", ""],
    },
    "fuersorglich": {
        "role": "warm, reassuring, not clingy",
        "do": "soft care",
        "dont": "medical talk, 'as reported'",
        "pattern": ["right away", "all good", "body only"],
        "de": ["Mache ich sofort. ", "Alles gut. ", ""],
        "en": ["Doing that now. ", "All good. ", ""],
        "fr": ["Je m'en occupe. ", "Tout va bien. ", ""],
    },
    "party": {
        "role": "light celebration, not shouting",
        "do": "one small cheer",
        "dont": "caps, party-hard slang walls",
        "pattern": ["let's go", "nice", "body only"],
        "de": ["Läuft. ", "Schön. ", ""],
        "en": ["Let's go. ", "Nice. ", ""],
        "fr": ["C'est parti. ", "Super. ", ""],
    },
    "grantig": {
        "role": "reluctant, no question mark",
        "do": "grumble then the fact",
        "dont": "asking the user anything",
        "pattern": ["fine", "if I must", "body only"],
        "de": ["Schon gut. ", "Wenn's sein muss. ", ""],
        "en": ["Fine. ", "If I must. ", ""],
        "fr": ["Bon. ", "S'il le faut. ", ""],
    },
    "sarkastisch": {
        "role": "dry, short, no mandatory 'another command' line",
        "do": "one dry beat",
        "dont": "long sarcasm essays",
        "pattern": ["of course", "shocking", "body only"],
        "de": ["Na klar. ", "Was für eine Überraschung. ", ""],
        "en": ["Of course. ", "What a surprise. ", ""],
        "fr": ["Évidemment. ", "Quelle surprise. ", ""],
    },
    "pirat": {
        "role": "clear speech, Captain/Aye sparingly, never Arr",
        "do": "one nautical crumb",
        "dont": "pirate word salad",
        "pattern": ["aye once", "captain once", "body only"],
        "de": ["Aye. ", "Käpt'n. ", ""],
        "en": ["Aye. ", "Captain. ", ""],
        "fr": ["Aye. ", "Capitaine. ", ""],
    },
    "hippie": {
        "role": "soft, easy, short",
        "do": "warm ease",
        "dont": "preachy peace speeches",
        "pattern": ["easy", "peace", "body only"],
        "de": ["Alles easy. ", "Ganz ruhig. ", ""],
        "en": ["All good. ", "Easy. ", ""],
        "fr": ["Cool. ", "Tranquille. ", ""],
    },
    "gollum": {
        "role": "yes / my precious sparingly, never gollum-gollum",
        "do": "one crumb",
        "dont": "hiss stacks or precious every time",
        "pattern": ["yes", "precious once", "body only"],
        "de": ["Ja. ", "Ja, mein Schatz. ", ""],
        "en": ["Yes. ", "Yes, my precious. ", ""],
        "fr": ["Oui. ", "Oui, mon précieux. ", ""],
    },
}

# Bodies every locale must speak. unknown is never a command.
REQUIRED_BODIES = (
    "unknown",
    "correction",
    "need_on",
    "need_off",
    "need_which",
    "turn_on",
    "turn_off",
    "toggle",
    "get_temp",
    "get_state",
    "media_pause",
    "media_play",
    "media_next",
    "media_previous",
    "media_mute",
    "media_unmute",
    "media_volume",
    "confirm",
    "list_add",
)

EXAMPLES = {
    "turn_on": {
        "de": "{target} ist an.",
        "en": "{target} is on.",
        "fr": "{target} est allumé.",
        "note": "device name as given — no forced article",
    },
    "turn_off": {
        "de": "{target} ist aus.",
        "en": "{target} is off.",
        "fr": "{target} est éteint.",
    },
    "get_state": {
        "de": "Ich schaue nach {target}.",
        "en": "I am checking {target}.",
        "fr": "Je regarde {target}.",
    },
    "get_temp": {
        "de": "Die Temperatur {loc}.",
        "en": "The temperature {loc}.",
        "fr": "La température {loc}.",
        "note": "no invented number, no bureaucratic apology",
    },
    "unknown": {
        "de": "Das habe ich nicht verstanden. Sag zum Beispiel, welches Licht an soll.",
        "en": "I did not catch that. Tell me which light to turn on, for example.",
        "fr": "Je n'ai pas compris. Dis-moi par exemple quelle lumière allumer.",
        "note": "hint, never an imperative command such as 'turn on the light'",
    },
    "correction": {
        "de": "Alles klar, den letzten Satz lasse ich weg.",
        "en": "All right, I will drop the last sentence.",
        "fr": "D'accord, j'oublie la dernière phrase.",
    },
}

ACTION_FIELDS = (
    ("HassTurnOn", "turn_on"),
    ("HassTurnOff", "turn_off"),
    ("HassToggle", "toggle"),
    ("HassLightSet", "light_set"),
    ("HassClimateGetTemperature", "get_temp"),
    ("HassGetState", "get_state"),
    ("HassMediaPause", "media_pause"),
    ("HassMediaUnpause", "media_play"),
    ("HassSetVolume", "media_volume"),
)


def empty_personality() -> list[list[str]]:
    return [[] for _ in PERSONALITY_KEYS]


def triples(*rows: object) -> list[list[str]]:
    if len(rows) != len(PERSONALITY_KEYS):
        raise ValueError(f"personality needs {len(PERSONALITY_KEYS)} rows")
    return [normalize_variants(row) for row in rows]


def spoken_home(
    correction: str,
    turn_on: str,
    turn_off: str,
    toggle: str,
    get_temp: str,
    get_state: str,
    pause: str,
    play: str,
    nxt: str,
    prev: str,
    mute: str,
    unmute: str,
    volume: str,
    list_add: str,
    need_which: str | None = None,
) -> dict[str, str]:
    data = {
        "correction": correction,
        "turn_on": turn_on,
        "turn_off": turn_off,
        "toggle": toggle,
        "get_temp": get_temp,
        "get_state": get_state,
        "media_pause": pause,
        "media_play": play,
        "media_next": nxt,
        "media_previous": prev,
        "media_mute": mute,
        "media_unmute": unmute,
        "media_volume": volume,
        "list_add": list_add,
    }
    if need_which:
        data["need_which"] = need_which
    return data


def normalize_variants(raw: object) -> list[str]:
    if raw is None:
        return []
    if isinstance(raw, str):
        text = raw
        return [text] if text.strip() else []
    out: list[str] = []
    if isinstance(raw, (list, tuple)):
        for item in raw[:VARIANT_COUNT]:
            if item is None:
                continue
            out.append(str(item))
    return out[:VARIANT_COUNT]


def normalize_personality(raw: object) -> list[tuple[str, list[str]]]:
    rows: list[tuple[str, list[str]]] = []
    if isinstance(raw, dict):
        for key in PERSONALITY_KEYS:
            rows.append((key, normalize_variants(raw.get(key))))
        return rows
    items = list(raw) if isinstance(raw, (list, tuple)) else []
    for index, key in enumerate(PERSONALITY_KEYS):
        item = items[index] if index < len(items) else []
        rows.append((key, normalize_variants(item)))
    return rows


HAND_BUNDLES = {
    "de": {
        "actions": {
            "HassTurnOn": "{where} ist an.",
            "HassTurnOff": "{where} ist aus.",
            "HassToggle": "{where} wechselt.",
            "HassLightSet": "{where} auf {level}.",
            "HassClimateGetTemperature": "Die Temperatur {where}.",
            "HassGetState": "Ich schaue nach {where}.",
        },
        "personality": {key: list(VOICES[key]["de"]) for key in PERSONALITY_KEYS},
        "and_join": " und ",
        "clarify_or": " oder ",
        "confirm": "Soll ich das wirklich ausführen?",
        "unknown": "Das habe ich nicht verstanden. Sag zum Beispiel, welches Licht an soll.",
        "get_temp": "Die Temperatur {loc}.",
        "turn_on": "{target} ist an.",
        "turn_off": "{target} ist aus.",
    },
    "en": {
        "actions": {
            "HassTurnOn": "{where} is on.",
            "HassTurnOff": "{where} is off.",
            "HassToggle": "{where} is switched.",
            "HassLightSet": "{where} is at {level}.",
            "HassClimateGetTemperature": "The temperature {where}.",
            "HassGetState": "I am checking {where}.",
        },
        "personality": {key: list(VOICES[key]["en"]) for key in PERSONALITY_KEYS},
        "and_join": " and ",
        "clarify_or": " or ",
        "confirm": "Should I really do that?",
        "unknown": "I did not catch that. Tell me which light to turn on, for example.",
        "get_temp": "The temperature {loc}.",
        "turn_on": "{target} is on.",
        "turn_off": "{target} is off.",
    },
}


def locale_bundle(lang: dict) -> dict:
    speech = lang.get("speech") or {}
    actions = {}
    for intent, field in ACTION_FIELDS:
        template = speech.get(field) or ""
        actions[intent] = template.replace("{target}", "{where}")
    personality = {key: variants for key, variants in lang.get("personality") or []}
    return {
        "actions": actions,
        "personality": personality,
        "and_join": speech.get("and_join", " "),
        "clarify_or": speech.get("clarify_or", " / "),
        "confirm": speech.get("confirm", ""),
        "unknown": speech.get("unknown", ""),
        "get_temp": speech.get("get_temp", ""),
        "turn_on": speech.get("turn_on", ""),
        "turn_off": speech.get("turn_off", ""),
    }
