"""Post-expand fixups so packs pass package 4.4 lint (L-17/L-18/L-19)."""

from __future__ import annotations

import re

from lang_packs.convo import with_warm_white
from lang_packs.household_lex import household_for
from lang_packs.native_apply import is_script_pack
from lang_packs.native_colors import colors_for

EN_CHAT_STUBS = frozenset({"mood", "tell", "idea", "why", "weather", "stop", "news"})


def _bare(template: str | None) -> bool:
    if template is None:
        return True
    return not any(ch.isalpha() for ch in re.sub(r"\{\w+\}", "", template))


def _first(words: list[str] | None) -> str:
    for word in words or []:
        if word:
            return word
    return ""


def _dot(text: str) -> str:
    text = (text or "").strip()
    if not text:
        return "."
    if text[-1] in ".?!。？！…؟":
        return text
    return text + "."


def _unique(values: list[str]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for item in values:
        if item and item not in seen:
            seen.add(item)
            out.append(item)
    return out


def default_colors() -> list[tuple[str, str]]:
    return [(color, color) for color in ("red", "blue", "green", "yellow", "orange", "pink", "black", "white", "purple")]


def resolve_colors(core: dict) -> list[tuple[str, str]]:
    existing = list(core.get("colors") or [])
    native = colors_for(core["code"], existing)
    if native is not None:
        return with_warm_white(core["code"], native)
    if is_script_pack(core):
        cleaned = colors_for(core["code"], existing or default_colors())
        if cleaned:
            return with_warm_white(core["code"], cleaned)
    return with_warm_white(core["code"], existing or default_colors())


def merge_household_words(code: str, w: dict) -> list[str]:
    """Copy real clock/weekday/… phrases into words; return greets for chat."""
    lex = household_for(code)
    for key in ("clock", "weekday", "weather", "explain", "teach", "undo", "date", "today"):
        if lex.get(key):
            w[key] = _unique(list(lex[key]) + list(w.get(key) or []))
    return list(lex.get("greet") or [])


def ensure_action_speech(speech: dict, w: dict) -> None:
    open_v = _first(w.get("open"))
    close_v = _first(w.get("close"))
    lock_v = _first(w.get("lock_v")) or _first(w.get("lock"))
    unlock_v = _first(w.get("unlock")) or open_v
    yes = _first(w.get("yes")) or _first(w.get("done")) or "OK"

    if _bare(speech.get("cover_open")) or speech.get("cover_open") == speech.get("cover_close"):
        speech["cover_open"] = _dot(f"{open_v} {{target}}".strip() if open_v else "{target} open")
        speech["cover_close"] = _dot(f"{close_v} {{target}}".strip() if close_v else "{target} closed")
    if _bare(speech.get("lock")) or speech.get("lock") == speech.get("unlock"):
        speech["lock"] = _dot(f"{lock_v} {{target}}".strip() if lock_v else "{target} locked")
        speech["unlock"] = _dot(f"{unlock_v} {{target}}".strip() if unlock_v else "{target} unlocked")
    if _bare(speech.get("done")):
        speech["done"] = _dot(f"{yes}: {{name}}")
    for field in ("light_step_up", "light_step_down"):
        if _bare(speech.get(field)):
            speech[field] = speech.get("turn_on") or _dot("{target}")


def scrub_chat(chat: dict, w: dict, greets: list[str]) -> None:
    on_verbs = set(w.get("on") or [])
    if greets:
        chat["greet"] = greets
    elif chat.get("greet") and set(chat["greet"]) <= on_verbs:
        chat["greet"] = []
    for key in ("feeling", "tell", "advice", "world", "open"):
        chat[key] = [word for word in (chat.get(key) or []) if word not in EN_CHAT_STUBS]
    # Parity reject probes need a world cue; reuse a weather phrase head, never EN "weather".
    if not chat.get("world"):
        weather = w.get("weather") or []
        head = (weather[0].split()[0] if weather else "") or (greets[0] if greets else "")
        chat["world"] = [head] if head else []
