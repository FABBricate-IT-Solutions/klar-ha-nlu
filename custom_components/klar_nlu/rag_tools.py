"""Klar-owned tools for opt-in NLU-RAG. Never Assist/HA control tools."""

from __future__ import annotations

from typing import Any

_PARSE = "klar.parse"
_ACT = "klar.act"

_INSTRUCT = {
    "de": (
        "Wenn der Satz ein Hausbefehl ist, antworte mit genau einer Zeile und sonst nichts: "
        "KLAR_PARSE: <klarer Befehl>. "
        "Sonst antworte kurz im Gespräch. "
        "Nenne niemals Werkzeuge, Intents oder Präfixe."
    ),
    "en": (
        "If the sentence is a home command, reply with exactly one line and nothing else: "
        "KLAR_PARSE: <clear command>. "
        "Otherwise reply briefly in conversation. "
        "Never name tools, intents, or prefixes."
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
    instruct = _INSTRUCT.get(pack, _INSTRUCT["en"])
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


def holds_klar_tool_prefix(speech: str) -> bool:
    stripped = (speech or "").lstrip()
    marker = "KLAR_"
    return not stripped or stripped.startswith(marker) or marker.startswith(stripped)


def leaks_klar_tools(speech: str) -> bool:
    """True when the model named Klar tools instead of using the protocol line."""
    if parse_tool_reply(speech) is not None:
        return False
    text = (speech or "").lower().replace("`", "")
    return "klar.parse" in text or "klar.act" in text or "klar_parse" in text or "klar_act" in text


def act_payload(intent_name: str, slots: dict[str, str]) -> dict[str, Any]:
    return {
        "name": intent_name,
        "slots": [{"name": key, "value": value} for key, value in slots.items()],
    }
