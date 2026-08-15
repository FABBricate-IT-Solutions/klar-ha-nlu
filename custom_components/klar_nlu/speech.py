"""Spoken replies after Home Assistant ran an intent."""

from __future__ import annotations

from typing import Any

_ACTION = {
    "HassTurnOn": {"de": "{where} ist an.", "en": "{where} is on."},
    "HassTurnOff": {"de": "{where} ist aus.", "en": "{where} is off."},
    "HassToggle": {"de": "{where} ist umgeschaltet.", "en": "{where} is toggled."},
    "HassLightSet": {"de": "{where} auf {level}.", "en": "{where} is at {level}."},
}

_DE_STATE = {
    "on": "an",
    "off": "aus",
    "unavailable": "nicht da",
    "unknown": "unbekannt",
    "open": "offen",
    "closed": "zu",
    "locked": "zu",
    "unlocked": "offen",
    "home": "zuhause",
    "not_home": "unterwegs",
    "idle": "bereit",
    "paused": "pausiert",
    "playing": "spielt",
    "docked": "an der Station",
    "cleaning": "saugt",
    "returning": "fährt zur Station",
    "heat": "heizt",
    "cool": "kühlt",
    "auto": "automatisch",
    "dry": "entfeuchtet",
    "fan_only": "nur Lüfter",
}

_STYLE = {
    ("butler", "de"): "Sehr wohl. ",
    ("butler", "en"): "Very well. ",
    ("locker", "de"): "Geht klar. ",
    ("locker", "en"): "Got it. ",
    ("fuersorglich", "de"): "Mache ich sofort. ",
    ("fuersorglich", "en"): "Doing that now. ",
    ("party", "de"): "Läuft! ",
    ("party", "en"): "Let's go! ",
    ("grantig", "de"): "Schon gut. ",
    ("grantig", "en"): "Fine. ",
    ("sarkastisch", "de"): "Wie überraschend, wieder ein Befehl. ",
    ("sarkastisch", "en"): "What a surprise, another command. ",
    ("pirat", "de"): "Aye. ",
    ("pirat", "en"): "Aye. ",
    ("hippie", "de"): "Alles easy. ",
    ("hippie", "en"): "All good. ",
    ("gollum", "de"): "Ja, mein Schatz. ",
    ("gollum", "en"): "Yes, my precious. ",
}


def style(speech: str, personality: str, pack: str) -> str:
    prefix = _STYLE.get((personality, pack), "")
    if not prefix or speech.startswith(prefix.strip()):
        return speech
    return f"{prefix}{speech}"


def from_handled(handled: Any, pack: str, item: dict) -> str | None:
    name = str(item.get("name") or "")
    if _is_query(handled, name):
        query = query_speech(handled, pack, item)
        if query:
            return query
    template = (_ACTION.get(name) or {}).get(pack)
    if template:
        where = _pretty_where(handled, item, pack)
        return template.format(where=where, level=_level(item, pack))
    text = _plain_speech(handled)
    if text:
        if pack == "de":
            for eng, de in _DE_STATE.items():
                text = text.replace(f": {eng}", f" ist {de}")
                text = text.replace(f" {eng}.", f" {de}.")
        return text
    return query_speech(handled, pack, item) or None


def query_speech(handled: Any, pack: str, item: dict | None = None) -> str:
    states = list(getattr(handled, "matched_states", None) or [])
    if not states:
        states = list(getattr(handled, "unmatched_states", None) or [])
    rows = [_state_value(state, pack) for state in states[:12]]
    rows = [row for row in rows if row[0] and row[1]]
    area = _room_label(item)
    if area:
        return _room_status(rows, area, pack)
    lights = [row for row in rows if row[2] == "light"]
    if len(lights) >= 2:
        on_word, off_word = ("on", "off") if pack == "en" else ("an", "aus")
        on = sum(1 for row in lights if row[1] == on_word)
        off = sum(1 for row in lights if row[1] == off_word)
        if pack == "en":
            bits = [f"{on} lights on"] if on else []
            if off:
                bits.append(f"{off} lights off")
        else:
            bits = [f"{on} Licht an" if on == 1 else f"{on} Lichter an"] if on else []
            if off:
                bits.append(f"{off} Licht aus" if off == 1 else f"{off} Lichter aus")
        extra = [
            f"{n} ist {v}" if pack == "de" else f"{n} is {v}"
            for n, v, d in rows
            if d != "light"
        ]
        return ". ".join(bits + extra[:3]) + "."
    parts: list[str] = []
    for name, spoken, _domain in rows[:4]:
        if pack == "en":
            parts.append(f"{name} is {spoken.replace(',', '.')}.")
        else:
            parts.append(f"{name} ist {spoken}.")
    return " ".join(parts)


def _room_label(item: dict | None) -> str:
    if not item:
        return ""
    slots = _slots(item)
    if slots.get("entity_id"):
        return ""
    return str(slots.get("area_name") or slots.get("area") or "").strip()


def _in_area(area: str, pack: str) -> str:
    pretty = _humanize(area)
    if pretty:
        pretty = pretty[:1].upper() + pretty[1:]
    if pack != "de":
        return f"in the {pretty}"
    folded = pretty.lower().replace("ü", "u").replace("ä", "a").replace("ö", "o")
    if folded.endswith("e") or folded in {"wohnung"}:
        return f"in der {pretty}"
    return f"im {pretty}"


def _room_status(rows: list[tuple[str, str, str]], area: str, pack: str) -> str:
    where = _in_area(area, pack)
    lights = [row for row in rows if row[2] == "light"]
    others = [(n, v) for n, v, d in rows if d != "light"][:3]
    if len(rows) == 1 and rows[0][2] == "light":
        spoken = rows[0][1]
        if pack == "en":
            return f"The light {where} is {spoken}."
        return f"{where[:1].upper()}{where[1:]} ist das Licht {spoken}."
    if len(lights) >= 2:
        on_word, off_word = ("on", "off") if pack == "en" else ("an", "aus")
        on = sum(1 for row in lights if row[1] == on_word)
        off = sum(1 for row in lights if row[1] == off_word)
        if pack == "en":
            bits = [f"{on} lights on"] if on else []
            if off:
                bits.append(f"{off} lights off")
            extra = [f"{n} is {v}" for n, v in others]
            return f"{where[:1].upper()}{where[1:]}: " + ", ".join(bits + extra) + "."
        bits = [f"{on} Licht an" if on == 1 else f"{on} Lichter an"] if on else []
        if off:
            bits.append(f"{off} Licht aus" if off == 1 else f"{off} Lichter aus")
        extra = [f"{n} {v}" for n, v in others]
        return f"{where[:1].upper()}{where[1:]}: " + ", ".join(bits + extra) + "."
    parts: list[str] = []
    for name, spoken, domain in rows[:4]:
        label = "Licht" if pack == "de" and domain == "light" and name.lower() in _LIGHT_TAIL else name
        if pack == "en":
            parts.append(f"{label} is {spoken.replace(',', '.')}")
        else:
            parts.append(f"{label} ist {spoken}")
    body = ". ".join(parts)
    if not body:
        return f"{where[:1].upper()}{where[1:]} ist in Ordnung." if pack == "de" else f"{where[:1].upper()}{where[1:]} looks fine."
    return f"{where[:1].upper()}{where[1:]}: {body}."


def _speak_state(raw: str, pack: str) -> str:
    key = str(raw).strip().lower()
    if pack == "de":
        return _DE_STATE.get(key, str(raw).replace(".", ","))
    return str(raw)


def _state_value(state: Any, pack: str) -> tuple[str, str, str]:
    attrs = getattr(state, "attributes", None) or {}
    if not isinstance(attrs, dict):
        attrs = {}
    unit = str(attrs.get("unit_of_measurement") or attrs.get("temperature_unit") or "")
    name = str(attrs.get("friendly_name") or "")
    name = name or str(getattr(state, "name", None) or getattr(state, "entity_id", ""))
    entity_id = str(getattr(state, "entity_id", "") or "")
    domain = entity_id.split(".", 1)[0]
    raw = attrs.get("current_temperature")
    if raw is None or raw == "":
        spoken = _speak_state(getattr(state, "state", ""), pack)
    else:
        spoken = f"{str(raw).replace('.', ',')} {unit or '°C'}".strip()
    return name, spoken, domain


def _plain_speech(handled: Any) -> str:
    speech = getattr(handled, "speech", None) or {}
    plain = speech.get("plain") if isinstance(speech, dict) else None
    if isinstance(plain, dict):
        text = str(plain.get("speech") or "").strip()
        if text:
            return text
    as_dict = getattr(handled, "as_dict", None)
    if callable(as_dict):
        data = as_dict()
        nested = (data.get("speech") or {}).get("plain") or {}
        return str(nested.get("speech") or "").strip()
    return ""


def _is_query(handled: Any, name: str) -> bool:
    rtype = getattr(handled, "response_type", None)
    value = getattr(rtype, "value", rtype)
    return str(value) == "query_answer" or name in {
        "HassGetState",
        "HassClimateGetTemperature",
    }


_LIGHT_TAIL = {"licht", "lichter", "lampe", "lampen", "light", "lights"}
_ALL_HEAD = {"alle", "all", "every", "überall", "ueberall"}


def _compound_light(name: str) -> str:
    parts = [part for part in str(name).split() if part]
    if (
        len(parts) >= 2
        and parts[0].lower() not in _ALL_HEAD
        and parts[-1].lower() in _LIGHT_TAIL
    ):
        head = "".join(parts[:-1])
        return f"{head[:1].upper()}{head[1:]}{parts[-1].lower()}"
    return name


def _drop_prefixes(names: list[str]) -> list[str]:
    keys = [(name, name.lower().replace(" ", "")) for name in names]
    out: list[str] = []
    for name, key in keys:
        if any(key != other and other.startswith(key) for _, other in keys):
            continue
        if name not in out:
            out.append(name)
    return out


def _pretty_where(handled: Any, item: dict, pack: str) -> str:
    slots = _slots(item)
    entity_id = str(slots.get("entity_id") or "")
    names = [
        _spoken_device(
            str(getattr(target, "name", None) or ""),
            str(getattr(target, "id", None) or entity_id),
            pack,
        )
        for target in getattr(handled, "success_results", None) or []
    ]
    names = _drop_prefixes([name for name in names if name])
    if names:
        return (" und " if pack == "de" else " and ").join(names)
    raw = str(slots.get("name") or slots.get("area") or "")
    if "." in raw:
        raw = ""
    if raw:
        return _spoken_device(raw, entity_id, pack)
    if entity_id:
        return _spoken_device("", entity_id, pack)
    return "Zuhause" if pack == "de" else "home"


def _slots(item: dict) -> dict[str, str]:
    return {
        str(slot["name"]): str(slot.get("value") or "")
        for slot in item.get("slots") or []
        if isinstance(slot, dict) and slot.get("name")
    }


def _level(item: dict, pack: str) -> str:
    slots = _slots(item)
    bri = slots.get("brightness") or slots.get("percentage") or ""
    if not bri:
        return "die neue Stufe" if pack == "de" else "the new level"
    return f"{bri} Prozent" if pack == "de" else f"{bri} percent"


def _humanize(raw: str) -> str:
    text = str(raw).strip()
    if "." in text and " " not in text:
        text = text.split(".", 1)[-1]
    return text.replace("_", " ")


def _spoken_device(name: str, entity_id: str, pack: str) -> str:
    domain = entity_id.split(".", 1)[0] if "." in entity_id else ""
    pretty = _compound_light((name or _humanize(entity_id)).strip())
    if pretty == entity_id or pretty.startswith(f"{domain}."):
        pretty = _compound_light(_humanize(entity_id))
    folded = pretty.lower().replace(" ", "")
    light_word = any(
        token in folded for token in ("licht", "lampe", "leuchte", "light", "lamp", "kugel")
    )
    if domain == "light" and pretty and not light_word:
        pretty = f"{pretty} light" if pack == "en" else f"{pretty}licht"
    if pretty:
        return pretty[:1].upper() + pretty[1:]
    return pretty
