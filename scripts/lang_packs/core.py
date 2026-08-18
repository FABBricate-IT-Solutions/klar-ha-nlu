"""Shared lexicon builders. Language data lives in the lexicons_*.py modules."""

from __future__ import annotations

from lang_packs.speech_tmpl import chat, speech

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
        "personality": extra.get("personality", ["", "", "", "", "", "", "", "", ""]),
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


def sp(unknown, on, off, which, confirm, and_join=" ", or_join=" / ", heat="", cool="", light=""):
    heat = heat or on
    cool = cool or off
    lamp = light or on
    return speech(
        unknown,
        on,
        off,
        which,
        confirm,
        "{names}?",
        or_join,
        and_join,
        "{names}.",
        "{names}.",
        "{target}.",
        "{target}.",
        "{target}.",
        "{target}.",
        "{target} {n}%",
        "{target} {color}",
        "{noun} {target} {n}",
        heat,
        cool,
        "{loc}",
        "{target}",
        on,
        on,
        on,
        off,
        off,
        on,
        "{n}",
        on,
        on,
        on,
        "{n}",
        "{target}",
        "{target}",
        on,
        on,
        off,
        off,
        on,
        "{name}",
        f" {lamp}" if lamp else "",
        f"{lamp} {{loc}}" if lamp else "{loc}",
        "{room}",
        "{room}",
        on,
        on,
        confirm,
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
            need_on,
            confirm,
            and_join=and_join,
            or_join=or_join,
            heat=climate[0],
            cool=climate[-1],
            light=light0,
        ),
        euro_chat([on0], [confirm], [on0], [on0], [off0], unknown, confirm, confirm),
        living_kitchen(living, kitchen),
        colors or [],
        nums(numbers),
        smoke or [(f"{on0} {light0} {living}", "HassTurnOn"), (f"{off0} {light0} {kitchen}", "HassTurnOff")],
        script=script,
        extra_verbs=extra_verbs or [],
    )
