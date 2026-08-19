"""Shared lexicon builders. Language data lives in the lexicons_*.py modules."""

from __future__ import annotations

from lang_packs.speech_tmpl import chat, speech
from lang_packs.voices import empty_personality

NUMBER_VALUES = list(range(0, 21)) + [30, 40, 50, 60, 70, 80, 90, 100]


def nums(words: list[str]) -> list[tuple[str, int]]:
    padded = list(words)
    while len(padded) < len(NUMBER_VALUES):
        padded.append(str(NUMBER_VALUES[len(padded)]))
    return list(zip(padded[: len(NUMBER_VALUES)], NUMBER_VALUES))


def C(code, native, variants, w, speech_d, chat_d, rooms, colors, numbers, smoke, **extra):
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
        "personality": extra.get("personality", empty_personality()),
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
):
    spoken = spoken or {}
    heat = heat or on
    cool = cool or off
    lamp = light or on
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
        body("media_pause", on),
        body("media_play", on),
        body("media_next", on),
        body("media_previous", off),
        body("media_mute", off),
        body("media_unmute", on),
        body("media_volume", "{n}"),
        body("media_search", on),
        body("media_transfer", on),
        body("media_favorite", on),
        body("fan_set", "{n}"),
        body("vacuum_start", "{target}"),
        body("vacuum_dock", "{target}"),
        body("vacuum_default", on),
        body("timer_start", on),
        body("timer_cancel", off),
        body("timer_pause", off),
        body("list_add", on),
        body("done", "{name}"),
        f" {lamp}" if lamp else "",
        f"{lamp} {{loc}}" if lamp else "{loc}",
        "{room}",
        "{room}",
        on,
        on,
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
            heat=climate[0],
            cool=climate[-1],
            light=light0,
            correction=correction,
            spoken=spoken,
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
