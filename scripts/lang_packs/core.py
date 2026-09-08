"""Shared lexicon builders. Language data lives in the lexicons_*.py modules."""

from __future__ import annotations

from lang_packs.speech_tmpl import chat, speech
from lang_packs.voices import empty_personality, localize_jarvis

NUMBER_VALUES = list(range(0, 21)) + [30, 40, 50, 60, 70, 80, 90, 100]


def nums(words: list[str]) -> list[tuple[str, int]]:
    padded = list(words)
    while len(padded) < len(NUMBER_VALUES):
        padded.append(str(NUMBER_VALUES[len(padded)]))
    return list(zip(padded[: len(NUMBER_VALUES)], NUMBER_VALUES))


def C(code, native, variants, w, speech_d, chat_d, rooms, colors, numbers, smoke, **extra):
    personality = localize_jarvis(
        extra.get("personality", empty_personality()),
        (w or {}).get("yes") or [],
        (speech_d or {}).get("confirm") or "",
    )
    core = {
        "code": code,
        "mod": extra.get("mod", code.replace("-", "_").lower()),
        "native": native,
        "script": extra.get("script", "Latn"),
        "variants": variants,
        "w": w,
        "speech": speech_d,
        "chat": chat_d,
        "rooms": rooms,
        "colors": colors,
        "numbers": numbers,
        "smoke": smoke,
        "personality": personality,
    }
    core.update({k: v for k, v in extra.items() if k not in ("mod", "script", "personality")})
    return core


def w(
    on,
    off,
    open_,
    close,
    query,
    set_,
    light,
    cover,
    climate,
    media,
    lock,
    door,
    timer,
    list_,
    fan,
    vacuum,
    scene,
    fillers,
    and_,
    or_,
    yes,
    all_,
    **more,
):
    data = {
        "on": on,
        "off": off,
        "open": open_,
        "close": close,
        "query": query,
        "set": set_,
        "light": light,
        "cover": cover,
        "climate": climate,
        "media": media,
        "lock": lock,
        "door": door,
        "timer": timer,
        "list": list_,
        "fan": fan,
        "vacuum": vacuum,
        "scene": scene,
        "fillers": fillers,
        "and": and_,
        "or": or_,
        "yes": yes,
        "all": all_,
    }
    data.update(more)
    return data


def rooms(*pairs: tuple[str, str]) -> list[tuple[str, str]]:
    return list(pairs)


def euro_chat(hello, thanks, who, story, news, intro, nudge, done):
    return chat(
        hello,
        thanks,
        ["mood"],
        who,
        ["tell"],
        story,
        ["weather"],
        ["idea"],
        who + ["why"],
        news,
        ["stop"],
        intro,
        nudge,
        done,
    )


def _spoken(spoken: dict, key: str, default: str) -> str:
    value = spoken.get(key)
    return default if value is None else value


def _first(words, fallback=""):
    if isinstance(words, str):
        return words or fallback
    return words[0] if words else fallback


def _dot(text: str) -> str:
    text = (text or "").strip()
    if not text:
        return "."
    if text[-1] in ".?!。？！…":
        return text
    return text + "."


# Climate lists sometimes put AC/cold first (ar/fa) or have a single word (hi/ja).
_COOL_LEADING = frozenset(
    {
        "تكييف",
        "سرما",
        "klima",
        "eakon",
        "kongtiao",
        "air",
        "ac",
        "clim",
        "airco",
        "pendingin",
        "dieuhoa",
        "aire",
        "エアコン",
        "空调",
        "空調",
        "에어컨",
        "แอร์",
        "تكييف",
        "מיזוג",
        "سرما",
    }
)


def climate_nouns(climate) -> tuple[str, str]:
    seen: set[str] = set()
    words: list[str] = []
    for item in climate or []:
        if item and item not in seen:
            seen.add(item)
            words.append(item)
    if not words:
        return "heat", "cool"
    if len(words) == 1:
        word = words[0]
        if word in _COOL_LEADING:
            return "heat", word
        return word, "ac" if word.casefold() != "ac" else "cool"
    first, last = words[0], words[-1]
    if first in _COOL_LEADING and last != first:
        return last, first
    return first, last


def sp(
    unknown,
    on,
    off,
    which,
    confirm,
    and_join=" ",
    or_join=" / ",
    heat="",
    cool="",
    light="",
    correction=None,
    spoken=None,
    slots=None,
):
    spoken = spoken or {}
    slots = slots or {}
    timer = slots.get("timer") or "timer"
    media = slots.get("media") or "media"
    vacuum = slots.get("vacuum") or "vacuum"
    lst = slots.get("list") or "list"
    fan = slots.get("fan") or "fan"
    living = slots.get("living") or ""
    play = slots.get("play") or ""
    pause = slots.get("pause") or ""
    on_verb = slots.get("on_verb") or ""
    off_verb = slots.get("off_verb") or ""
    door = slots.get("door") or light or ""
    if not heat or heat == on:
        heat = "heat"
    if not cool or cool == off or cool == heat:
        cool = "cool" if heat != "cool" else "ac"
    lamp = light
    body = lambda key, default: _spoken(spoken, key, default)
    return speech(
        body("unknown", unknown),
        body("need_on", on),
        body("need_off", off),
        body("need_which", which),
        body("correction", correction if correction is not None else confirm),
        body("clarify", "{names}?"),
        or_join,
        and_join,
        body("group_on", "{names}."),
        body("group_off", "{names}."),
        body("turn_on", "{target}."),
        body("turn_on_scene", "{target}."),
        body("turn_off", "{target}."),
        body("toggle", "{target}."),
        body("light_set", "{target} {n}%"),
        body("light_color", "{target} {color}"),
        body("climate_set", "{noun} {target} {n}"),
        heat,
        cool,
        body("get_temp", "{loc}"),
        body("get_state", "{target}"),
        body("media_pause", _dot(pause) if pause else _dot(media)),
        body("media_play", _dot(play) if play else _dot(f"{on_verb} {media}".strip())),
        body("media_next", f"{media}+"),
        body("media_previous", f"{media}-"),
        body("media_mute", _dot(off_verb) if off_verb else _dot(media)),
        body("media_unmute", _dot(on_verb) if on_verb else _dot(media)),
        body("media_volume", "{n}"),
        body("media_search", f"{media}?"),
        body("media_transfer", "{target}"),
        body("media_favorite", f"{media}!"),
        body("fan_set", f"{fan} {{n}}"),
        body("vacuum_start", "{target}"),
        body("vacuum_dock", "{target}"),
        body("vacuum_default", vacuum),
        body("timer_start", _dot(timer)),
        body("timer_cancel", _dot(f"{off_verb} {timer}".strip())),
        body("timer_pause", _dot(f"{pause} {timer}".strip()) if pause else f"{timer} /"),
        body("list_add", _dot(lst)),
        body("done", "{name}"),
        f" {lamp}" if lamp else "",
        f"{lamp} {{loc}}" if lamp else "{loc}",
        "{room}",
        "{room}",
        living or "{room}",
        door or lamp or vacuum,
        body("confirm", confirm),
    )


def living_kitchen(living: str, kitchen: str) -> list[tuple[str, str]]:
    return rooms((living, "wohnzimmer"), (kitchen, "kuche"))


def pack(
    code,
    native,
    variants,
    *,
    script="Latn",
    on,
    off,
    open_,
    close,
    query,
    set_,
    light,
    cover,
    climate,
    media,
    lock,
    door,
    timer,
    list_,
    fan,
    vacuum,
    scene,
    fillers,
    and_,
    or_,
    yes,
    all_,
    living,
    kitchen,
    unknown,
    need_on,
    need_off,
    confirm,
    numbers,
    colors=None,
    extra_verbs=None,
    extra_w=None,
    smoke=None,
    and_join=" ",
    or_join=" / ",
    personality=None,
    correction=None,
    need_which=None,
    spoken=None,
):
    words = w(
        on=on,
        off=off,
        open_=open_,
        close=close,
        query=query,
        set_=set_,
        light=light,
        cover=cover,
        climate=climate,
        media=media,
        lock=lock,
        door=door,
        timer=timer,
        list_=list_,
        fan=fan,
        vacuum=vacuum,
        scene=scene,
        fillers=fillers,
        and_=and_,
        or_=or_,
        yes=yes,
        all_=all_,
        kitchen=[kitchen],
        **(extra_w or {}),
    )
    on0, off0, light0 = on[0], off[0], light[0]
    heat_noun, cool_noun = climate_nouns(climate)
    return C(
        code,
        native,
        variants,
        words,
        sp(
            unknown,
            need_on,
            need_off,
            need_which or need_on,
            confirm,
            and_join=and_join,
            or_join=or_join,
            heat=heat_noun,
            cool=cool_noun,
            light=light0,
            correction=correction,
            spoken=spoken,
            slots={
                "timer": _first(timer, "timer"),
                "media": _first(media, "media"),
                "vacuum": _first(vacuum, "vacuum"),
                "list": _first(list_, "list"),
                "fan": _first(fan, "fan"),
                "living": living,
                "play": _first(words.get("play"), ""),
                "pause": _first(words.get("pause"), ""),
                "on_verb": on0,
                "off_verb": off0,
                "door": _first(door, light0),
            },
        ),
        euro_chat([on0], [confirm], [on0], [on0], [off0], unknown, confirm, confirm),
        living_kitchen(living, kitchen),
        colors or [],
        nums(numbers),
        smoke or [(f"{on0} {light0} {living}", "HassTurnOn"), (f"{off0} {light0} {kitchen}", "HassTurnOff")],
        script=script,
        extra_verbs=extra_verbs or [],
        personality=personality if personality is not None else empty_personality(),
    )
