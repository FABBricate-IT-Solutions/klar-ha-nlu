"""Room labels for light acks when Home Assistant speaks a generic name."""

from __future__ import annotations

from typing import Any

_ROOM_SPOKEN = {
    "kuche": ("Küche", "kitchen"),
    "kueche": ("Küche", "kitchen"),
    "kitchen": ("Küche", "kitchen"),
}


def generic_light(name: str) -> bool:
    return name.strip().casefold() in {"licht", "light", "lampe", "lamp", "leuchte"}


def light_word(name: str) -> bool:
    folded = name.casefold().replace(" ", "")
    return any(token in folded for token in ("licht", "light", "lampe", "lamp", "leuchte"))


def spoken_room(raw: str, pack: str, humanize) -> str:
    key = raw.strip().casefold().replace("ü", "u").replace("ä", "a").replace("ö", "o")
    pair = _ROOM_SPOKEN.get(key)
    if pair:
        return pair[1] if pack != "de" else pair[0]
    pretty = humanize(raw)
    return pretty[:1].upper() + pretty[1:] if pretty else ""


def spoken_room_from_item(item: dict, pack: str, slots: dict[str, str], humanize) -> str:
    for key in ("area_name", "area"):
        if slots.get(key):
            return spoken_room(slots[key], pack, humanize)
    entity_id = str(slots.get("entity_id") or "")
    tail = entity_id.split(".", 1)[-1] if "." in entity_id else entity_id
    for part in tail.replace("-", "_").split("_"):
        if part in _ROOM_SPOKEN:
            return spoken_room(part, pack, humanize)
    return ""


def area_light_phrase(room: str, pack: str) -> str:
    if pack != "de":
        return f"the {room} light"
    folded = room.casefold().replace("ü", "u").replace("ä", "a").replace("ö", "o")
    if folded.endswith("e") or folded in {"wohnung"}:
        return f"Licht in der {room}"
    return f"Licht im {room}"


def pretty_where(
    handled: Any,
    item: dict,
    pack: str,
    *,
    slots: dict[str, str],
    spoken_device,
    drop_prefixes,
    locale: dict,
    humanize,
) -> str:
    entity_id = str(slots.get("entity_id") or "")
    names = [
        spoken_device(
            str(getattr(target, "name", None) or ""),
            str(getattr(target, "id", None) or entity_id),
            pack,
        )
        for target in getattr(handled, "success_results", None) or []
    ]
    names = drop_prefixes([name for name in names if name])
    room = spoken_room_from_item(item, pack, slots, humanize)
    if names:
        joined = (locale.get("and_join") or " and ").join(names)
        if room and all(generic_light(name) for name in names):
            return area_light_phrase(room, pack)
        return joined
    raw = str(slots.get("name") or slots.get("area") or "")
    if "." in raw:
        raw = ""
    if raw:
        spoken = spoken_device(raw, entity_id, pack)
        if room and generic_light(spoken):
            return area_light_phrase(room, pack)
        return spoken
    if entity_id:
        spoken = spoken_device("", entity_id, pack)
        if room and (generic_light(spoken) or (light_word(spoken) and room.casefold() not in spoken.casefold())):
            return area_light_phrase(room, pack)
        return spoken
    if room:
        return area_light_phrase(room, pack)
    return "Zuhause" if pack == "de" else "home"
