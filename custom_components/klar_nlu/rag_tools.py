"""Klar-owned tools for opt-in NLU-RAG. Never Assist/HA control tools."""

from __future__ import annotations

from typing import Any

_PARSE = "klar.parse"
_ACT = "klar.act"

_INSTRUCT = {
    "de": (
        "Du darfst das Haus nur über Klar-Werkzeuge steuern: "
        f"{_PARSE} mit einem Satz oder {_ACT} mit Intent und Slots. "
        "Rufe keine Home-Assistant-Werkzeuge auf."
    ),
    "en": (
        "You may control the home only through Klar tools: "
        f"{_PARSE} with a sentence or {_ACT} with intent and slots. "
        "Do not call Home Assistant tools."
    ),
}


def retrieval_lines(retrieval: dict[str, Any] | None, pack: str) -> str:
    if not isinstance(retrieval, dict):
        return ""
    entities = retrieval.get("entities") or []
    names = []
    for item in entities[:8]:
        if isinstance(item, dict) and item.get("name"):
            names.append(str(item["name"]))
    areas = [str(item) for item in (retrieval.get("areas") or [])[:8]]
    last = [str(item) for item in (retrieval.get("last") or [])[:8]]
    label = "Kontext" if pack == "de" else "Context"
    bits = []
    if names:
        bits.append(", ".join(names))
    if areas:
        bits.append("/".join(areas))
    if last:
        bits.append(" · ".join(last))
    if not bits:
        return ""
    return f"{label}: {'; '.join(bits)}"


def rag_prompt(pack: str, retrieval: dict[str, Any] | None, extra: str | None) -> str:
    instruct = _INSTRUCT.get(pack, _INSTRUCT["de"])
    context = retrieval_lines(retrieval, pack)
    parts = [part for part in (extra, context, instruct) if part]
    return "\n".join(parts)


def parse_tool_reply(speech: str) -> dict[str, Any] | None:
    text = (speech or "").strip()
    if text.startswith("KLAR_PARSE:"):
        return {"tool": _PARSE, "text": text.split(":", 1)[1].strip()}
    if text.startswith("KLAR_ACT:"):
        body = text.split(":", 1)[1].strip()
        name, _, rest = body.partition(" ")
        slots: dict[str, str] = {}
        for part in rest.split():
            key, _, value = part.partition("=")
            if key and value:
                slots[key] = value
        return {"tool": _ACT, "intent": name.strip(), "slots": slots}
    return None


def act_payload(intent_name: str, slots: dict[str, str]) -> dict[str, Any]:
    return {
        "name": intent_name,
        "slots": [{"name": key, "value": value} for key, value in slots.items()],
    }
