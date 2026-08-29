"""Spoken replies after Home Assistant ran an intent."""

from __future__ import annotations

from pathlib import Path
from typing import Any

try:
    from .clock_speech import finish_clock_speech, strip_clock_seconds
    from .speech_place import pretty_where as _pretty_where_place
except ImportError:
    from clock_speech import finish_clock_speech, strip_clock_seconds
    from speech_place import pretty_where as _pretty_where_place
try:
    from .speech_locale import SPEECH_PACKS
except ImportError:
    try:
        from speech_locale import SPEECH_PACKS
    except ImportError:
        SPEECH_PACKS = {}

_FALLBACK_ACTION = {
    "HassTurnOn": "{where} is on.",
    "HassTurnOff": "{where} is off.",
    "HassToggle": "{where} is switched.",
    "HassLightSet": "{where} is at {level}.",
}
_WRAP = 0

_MEDIA_ACTION = {
    "HassMediaPause": ("{where} ist pausiert.", "{where} is paused."),
    "HassMediaUnpause": ("{where} spielt weiter.", "{where} resumed playback."),
    "HassMediaNext": ("Auf {where} läuft der nächste Titel.", "The next track is playing on {where}."),
    "HassMediaPrevious": ("Auf {where} läuft der vorherige Titel.", "The previous track is playing on {where}."),
    "HassMediaPlayerMute": ("{where} ist stumm.", "{where} is muted."),
    "HassMediaPlayerUnmute": ("Der Ton von {where} ist an.", "{where} is unmuted."),
    "MassFavorite": ("Als Favorit markiert.", "Marked as a favorite."),
    "HassMediaSearchAndPlay": ("Die Wiedergabe wurde gestartet.", "Playback started."),
    "MassPlayMedia": ("Die Wiedergabe wurde gestartet.", "Playback started."),
    "MassTransferQueue": ("Die Warteschlange wurde übertragen.", "The queue was transferred."),
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

def _locale(pack: str) -> dict:
    return SPEECH_PACKS.get(pack) or SPEECH_PACKS.get("en") or {}


def _en(pack: str) -> bool:
    return pack != "de"


def style(speech: str, personality: str, pack: str) -> str:
    global _WRAP
    if personality in {"", "default"}:
        return speech
    variants = list((_locale(pack).get("personality") or {}).get(personality) or [])
    if not variants:
        variants = list((_locale("en").get("personality") or {}).get(personality) or [])
    if not variants:
        return speech
    _WRAP += 1
    prefix = variants[(hash(speech) + _WRAP) % len(variants)]
    if not prefix or speech.startswith(prefix.strip()):
        return speech
    return f"{prefix}{speech}"

def from_handled(handled: Any, pack: str, item: dict) -> str | None:
    name = str(item.get("name") or "")
    if _is_query(handled, name):
        return query_speech(handled, pack, item) or None
    media_action = _media_action_speech(name, pack, item, handled)
    if media_action:
        return media_action
    template = (_locale(pack).get("actions") or {}).get(name) or _FALLBACK_ACTION.get(name)
    if template:
        where = _pretty_where(handled, item, pack)
        return template.format(where=where, level=_level(item, pack), loc=where)
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
    states = [state for state in states if not _infra_state(state)]
    status = _slots(item).get("media_status") if item else ""
    if status:
        for state in states:
            if str(getattr(state, "entity_id", "")).startswith("media_player."):
                return media_state_speech(state, status, pack)
        return ""
    rows = [_state_value(state, pack) for state in states[:12]]
    rows = [row for row in rows if row[0] and row[1]]
    area = _room_label(item)
    if area:
        return _room_status(rows, area, pack)
    lights = [row for row in rows if row[2] == "light"]
    if len(lights) >= 2:
        on_word, off_word = ("on", "off") if _en(pack) else ("an", "aus")
        on = sum(1 for row in lights if row[1] == on_word)
        off = sum(1 for row in lights if row[1] == off_word)
        if _en(pack):
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
        if _en(pack):
            parts.append(f"{name} is {spoken.replace(',', '.')}.")
        else:
            parts.append(f"{name} ist {spoken}.")
    return " ".join(parts)

def queue_speech(response: Any, state: Any, pack: str) -> str:
    current = _media_title(state)
    upcoming = [title for title in _queue_titles(response) if title and title != current][:3]
    if _en(pack):
        bits = [f"Now playing {current}."] if current else []
        if not upcoming:
            empty = "The queue is empty." if not current else "There is nothing else in the queue."
            return " ".join([*bits, empty])
        bits.append(f"Next is {upcoming[0]}.")
        if len(upcoming) > 1:
            bits.append("Then " + ", ".join(upcoming[1:]) + ".")
        return " ".join(bits)
    bits = [f"Gerade läuft {current}."] if current else []
    if not upcoming:
        empty = "Die Warteschlange ist leer." if not current else "Danach ist die Warteschlange leer."
        return " ".join([*bits, empty])
    bits.append(f"Als Nächstes kommt {upcoming[0]}.")
    if len(upcoming) > 1:
        bits.append("Danach " + ", ".join(upcoming[1:]) + ".")
    return " ".join(bits)

def media_state_speech(state: Any, status: str, pack: str) -> str:
    attrs = getattr(state, "attributes", None) or {}
    if not isinstance(attrs, dict):
        attrs = {}
    title = _media_title(state)
    raw_state = str(getattr(state, "state", "") or "")
    spoken_state = _speak_state(raw_state, pack)
    if status == "volume":
        volume = attrs.get("volume_level")
        muted = bool(attrs.get("is_volume_muted"))
        pct = _volume_percent(volume)
        if _en(pack):
            body = f"Volume is {pct} percent." if pct else "I cannot read the volume."
            return f"{body} It is muted." if muted else body
        body = f"Lautstärke ist {pct} Prozent." if pct else "Ich kann die Lautstärke nicht lesen."
        return f"{body} Der Ton ist stumm." if muted else body
    if status == "mute":
        muted = bool(attrs.get("is_volume_muted"))
        if _en(pack):
            return "It is muted." if muted else "It is not muted."
        return "Der Ton ist stumm." if muted else "Der Ton ist an."
    if status in {"now_playing", "player"}:
        if title:
            if _en(pack):
                prefix = "Now playing" if raw_state == "playing" else "Selected"
                return f"{prefix} {title}."
            prefix = "Gerade läuft" if raw_state == "playing" else "Ausgewählt ist"
            return f"{prefix} {title}."
        if _en(pack):
            return f"The player is {spoken_state}."
        return f"Der Player ist {spoken_state}."
    return ""

def _media_action_speech(name: str, pack: str, item: dict, handled: Any) -> str:
    where = _pretty_where(handled, item, pack)
    if name == "HassSetVolume":
        if _en(pack):
            return f"{where} volume is set to {_level(item, pack)}."
        return f"Die Lautstärke von {where} ist auf {_level(item, pack)}."
    if name == "HassSetVolumeRelative":
        down = _slots(item).get("volume_step") == "down"
        if _en(pack):
            return f"{where} volume was {'lowered' if down else 'raised'}."
        action = "verringert" if down else "erhöht"
        return f"Die Lautstärke von {where} wurde {action}."
    templates = _MEDIA_ACTION.get(name)
    if not templates:
        return ""
    return templates[1 if _en(pack) else 0].format(where=where)

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
        if _en(pack):
            return f"The light {where} is {spoken}."
        return f"{where[:1].upper()}{where[1:]} ist das Licht {spoken}."
    if len(lights) >= 2:
        on_word, off_word = ("on", "off") if _en(pack) else ("an", "aus")
        on = sum(1 for row in lights if row[1] == on_word)
        off = sum(1 for row in lights if row[1] == off_word)
        if _en(pack):
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
        if _en(pack):
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

def _media_title(state: Any) -> str:
    attrs = getattr(state, "attributes", None) or {}
    if not isinstance(attrs, dict):
        attrs = {}
    title = str(attrs.get("media_title") or attrs.get("title") or "").strip()
    artist = str(attrs.get("media_artist") or attrs.get("artist") or "").strip()
    album = str(attrs.get("media_album_name") or attrs.get("album") or "").strip()
    if title and artist:
        return f"{title} by {artist}"
    if title and album:
        return f"{title} ({album})"
    return title

def _volume_percent(value: Any) -> str:
    try:
        raw = float(value)
    except (TypeError, ValueError):
        return ""
    if raw <= 1.0:
        raw *= 100
    return str(round(raw))

def _queue_titles(response: Any) -> list[str]:
    items = _queue_items(response)
    return [_title_from_item(item) for item in items]

def _queue_items(value: Any) -> list[Any]:
    if isinstance(value, list):
        return value
    if isinstance(value, dict):
        for key in ("items", "queue", "queue_items", "media_items"):
            nested = value.get(key)
            if isinstance(nested, list):
                return nested
            if isinstance(nested, dict):
                found = _queue_items(nested)
                if found:
                    return found
        for nested in value.values():
            found = _queue_items(nested)
            if found:
                return found
    return []

def _title_from_item(item: Any) -> str:
    if isinstance(item, str):
        return item
    if not isinstance(item, dict):
        return ""
    media = item.get("media_item") if isinstance(item.get("media_item"), dict) else item
    title = str(media.get("name") or media.get("title") or media.get("media_title") or "").strip()
    artist = str(media.get("artist") or media.get("media_artist") or "").strip()
    if title and artist:
        return f"{title} by {artist}"
    return title

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

def _infra_needles() -> tuple[str, ...]:
    path = Path(__file__).with_name("infra_needles.txt")
    return tuple(
        line.strip()
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip() and not line.startswith("#")
    )

_INFRA = _infra_needles()

def _infra_state(state: Any) -> bool:
    entity_id = str(getattr(state, "entity_id", "") or "").lower()
    name = str(getattr(state, "name", "") or "").lower()
    attrs = getattr(state, "attributes", None) or {}
    if isinstance(attrs, dict):
        name = str(attrs.get("friendly_name") or name).lower()
        tags = attrs.get("tags") or []
        if isinstance(tags, list) and any(str(tag).lower() == "infra" for tag in tags):
            return True
    blob = f"{entity_id} {name}"
    return any(needle in blob for needle in _INFRA)

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
    return _pretty_where_place(
        handled,
        item,
        pack,
        slots=_slots(item),
        spoken_device=_spoken_device,
        drop_prefixes=_drop_prefixes,
        locale=_locale(pack),
        humanize=_humanize,
    )

def _slots(item: dict) -> dict[str, str]:
    return {
        str(slot["name"]): str(slot.get("value") or "")
        for slot in item.get("slots") or []
        if isinstance(slot, dict) and slot.get("name")
    }

_COLORS = {
    "red": {"de": "rot", "en": "red"},
    "blue": {"de": "blau", "en": "blue"},
    "green": {"de": "grün", "en": "green"},
    "yellow": {"de": "gelb", "en": "yellow"},
    "orange": {"de": "orange", "en": "orange"},
    "pink": {"de": "pink", "en": "pink"},
    "black": {"de": "schwarz", "en": "black"},
    "white": {"de": "weiß", "en": "white"},
    "purple": {"de": "lila", "en": "purple"},
}

def _level(item: dict, pack: str) -> str:
    slots = _slots(item)
    color = slots.get("color") or ""
    spoken = ""
    if color:
        spoken = (_COLORS.get(color) or {}).get(pack) or color
    bri = slots.get("brightness") or slots.get("percentage") or slots.get("volume_level") or ""
    if spoken and bri:
        unit = "Prozent" if pack == "de" else "percent"
        return f"{spoken}, {bri} {unit}"
    if spoken:
        return spoken
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
        pretty = f"{pretty} light" if _en(pack) else f"{pretty}licht"
    if pretty:
        return pretty[:1].upper() + pretty[1:]
    return pretty
