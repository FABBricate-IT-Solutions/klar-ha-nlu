"""Spoken replies after Home Assistant ran an intent."""

from __future__ import annotations

from typing import Any

_ACTION = {
    "HassTurnOn": {"de": "Schalte {where} ein.", "en": "Turn on {where}."},
    "HassTurnOff": {"de": "Schalte {where} aus.", "en": "Turn off {where}."},
    "HassToggle": {"de": "Schalte {where} um.", "en": "Toggle {where}."},
    "HassLightSet": {"de": "Setze {where}.", "en": "Set {where}."},
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
        query = query_speech(handled, pack)
        if query:
            return query
    template = (_ACTION.get(name) or {}).get(pack)
    if template:
        where = _pretty_where(handled, item, pack)
        return template.format(where=where)
    text = _plain_speech(handled)
    if text:
        if pack == "de":
            for eng, de in _DE_STATE.items():
                text = text.replace(f": {eng}", f" ist {de}")
                text = text.replace(f" {eng}.", f" {de}.")
        return text
    return query_speech(handled, pack) or None


def query_speech(handled: Any, pack: str) -> str:
    states = list(getattr(handled, "matched_states", None) or [])
    if not states:
        states = list(getattr(handled, "unmatched_states", None) or [])
    rows = [_state_value(state, pack) for state in states[:12]]
    rows = [row for row in rows if row[0] and row[1]]
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
    names = [
        str(getattr(target, "name", None) or getattr(target, "id", "") or "")
        for target in getattr(handled, "success_results", None) or []
    ]
    names = _drop_prefixes([_compound_light(name) for name in names if name])
    if names:
        return (" und " if pack == "de" else " and ").join(names)
    raw = _where(handled, item)
    if raw:
        return _compound_light(raw.replace("_", " "))
    return "home" if pack == "en" else "Zuhause"


def _where(handled: Any, item: dict) -> str:
    names = [
        str(getattr(target, "name", None) or getattr(target, "id", "") or "")
        for target in getattr(handled, "success_results", None) or []
    ]
    names = [name for name in names if name]
    if names:
        return ", ".join(dict.fromkeys(names))
    slots = {
        slot["name"]: slot["value"]
        for slot in item.get("slots") or []
        if isinstance(slot, dict) and slot.get("name")
    }
    return str(slots.get("area") or slots.get("name") or slots.get("entity_id") or "")
