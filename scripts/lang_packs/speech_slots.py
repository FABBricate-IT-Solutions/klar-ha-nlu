"""Spoken slot overlays for generated Assist packs.

pack()/sp() used lexicon nouns as whole replies (timer., tv?). These tables
replace those stubs with target-language sentences. de/en are handwritten
and never loaded here.
"""

from __future__ import annotations

from lang_packs.core import climate_nouns
from lang_packs.voices import PERSONALITY_KEYS, VOICES, normalize_personality

EN_CLIMATE = frozenset({"heat", "cool", "ac", "air", "eakon", "kongtiao", "nanbang", "temperature", "temperatur"})
EN_JARVIS = tuple(VOICES["jarvis"]["en"])
JOINS = {
    "hi": (" और ", " या "),
    "bn": (" এবং ", " বা "),
    "mr": (" आणि ", " किंवा "),
    "ne": (" र ", " वा "),
    "gu": (" અને ", " અથવા "),
    "pa": (" ਅਤੇ ", " ਜਾਂ "),
    "ta": (" மற்றும் ", " அல்லது "),
    "kn": (" ಮತ್ತು ", " ಅಥವಾ "),
    "ml": (" ഉം ", " അല്ലെങ്കിൽ "),
    "te": (" మరియు ", " లేదా "),
    "hy": (" և ", " կամ "),
    "ka": (" და ", " ან "),
    "mn": (" ба ", " эсвэл "),
}

GENERATED_CODES = (
    "fr", "nl", "es", "it", "pt", "ca", "ro",
    "da", "nb", "sv", "fi", "de-CH", "de-AT", "en-GB", "pt-BR", "af",
    "cs", "sk", "pl", "hu", "hr", "sl", "bg", "el", "sr", "sr-Latn", "uk",
    "zh-CN", "zh-TW", "zh-HK", "ar", "he", "fa", "ur", "tr", "th", "ko", "ja",
    "cy", "et", "eu", "ga", "gl", "is", "lb", "kw", "lt", "lv",
    "id", "ms", "sw", "vi",
    "hi", "bn", "gu", "kn", "ml", "mr", "ta", "te", "pa", "ne",
    "hy", "ka", "mn",
)


def S(
    ts,
    tc,
    tp,
    ms,
    mt,
    mf,
    vs,
    vd,
    v0,
    heat,
    cool,
    home,
    fan,
    **more,
) -> dict:
    row = {
        "timer_start": ts,
        "timer_cancel": tc,
        "timer_pause": tp,
        "media_search": ms,
        "media_transfer": mt,
        "media_favorite": mf,
        "vacuum_start": vs,
        "vacuum_dock": vd,
        "vacuum_default": v0,
        "heat_noun": heat,
        "cool_noun": cool,
        "loc_home": home,
        "fan_set": fan,
    }
    row.update({key: value for key, value in more.items() if value})
    return row


def P(*pairs: str) -> list[list[str]]:
    rows: list[list[str]] = []
    items = list(pairs)
    for index in range(0, len(items), 2):
        first = items[index] if index < len(items) else ""
        second = items[index + 1] if index + 1 < len(items) else ""
        rows.append([first, second, ""])
    while len(rows) < len(PERSONALITY_KEYS):
        rows.append([])
    return rows[: len(PERSONALITY_KEYS)]


def _ascii_letters(text: str) -> bool:
    letters = [ch for ch in text if ch.isalpha()]
    return bool(letters) and all(ord(ch) < 128 for ch in letters)


def _scriptier(candidate: str, current: str) -> bool:
    if not current:
        return True
    if _ascii_letters(current) and not _ascii_letters(candidate):
        return True
    return False


def pick_room_names(rooms: list[tuple[str, str]]) -> list[tuple[str, str]]:
    """Canon key (wohnzimmer) → native display. Prefer script over romanization."""
    best: dict[str, str] = {}
    order: list[str] = []
    for native, canon in rooms:
        if not native or not canon:
            continue
        if canon not in best:
            order.append(canon)
            best[canon] = native
        elif _scriptier(native, best[canon]):
            best[canon] = native
    return [(canon, best[canon]) for canon in order[:4]]


def native_room(core: dict, canon: str) -> str:
    best = ""
    for native, name in core.get("rooms") or []:
        if name != canon or not native:
            continue
        if _scriptier(native, best):
            best = native
    return best


def _tables() -> dict[str, dict]:
    from lang_packs.speech_slots_east import EAST
    from lang_packs.speech_slots_indic import INDIC
    from lang_packs.speech_slots_rest import REST
    from lang_packs.speech_slots_script import SCRIPT
    from lang_packs.speech_slots_west import WEST

    merged = {}
    for part in (WEST, EAST, SCRIPT, REST, INDIC):
        merged.update(part)
    return merged


def slot_gaps() -> list[str]:
    have = set(_tables())
    return [code for code in GENERATED_CODES if code not in have]


def apply_speech_slots(code: str, speech: dict, core: dict, personality: list) -> tuple[dict, list]:
    row = dict(_tables().get(code) or {})
    pers = row.pop("personality", None)
    jarvis = row.pop("jarvis", None)
    for key, value in row.items():
        if value:
            speech[key] = value
    pair = JOINS.get(code)
    if pair:
        speech["and_join"], speech["clarify_or"] = pair
    which, on = speech.get("need_which") or "", speech.get("need_on") or ""
    if which and on and _ascii_letters(which) and not _ascii_letters(on):
        speech["need_which"] = on
    if not row.get("loc_home"):
        living = native_room(core, "wohnzimmer")
        if living and not _ascii_letters(living):
            speech["loc_home"] = living
    heat, cool = climate_nouns((core.get("w") or {}).get("climate"))
    if (speech.get("heat_noun") or "").casefold() in EN_CLIMATE and heat and heat.casefold() not in EN_CLIMATE:
        speech["heat_noun"] = heat
    if (speech.get("cool_noun") or "").casefold() in EN_CLIMATE and cool and cool.casefold() not in EN_CLIMATE:
        speech["cool_noun"] = cool
    if pers:
        personality = normalize_personality(pers)
    elif jarvis:
        mapped = {key: list(vals) for key, vals in personality}
        mapped["jarvis"] = list(jarvis)
        personality = normalize_personality(mapped)
    else:
        mapped = {key: list(vals) for key, vals in personality}
        if mapped.get("jarvis") == list(EN_JARVIS):
            sark = mapped.get("sarkastisch") or []
            fuer = mapped.get("fuersorglich") or []
            mapped["jarvis"] = [sark[0] if sark else "", fuer[0] if fuer else "", ""]
            personality = normalize_personality(mapped)
    return speech, personality
