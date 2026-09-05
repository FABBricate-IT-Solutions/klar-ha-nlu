"""Klar-owned tools for opt-in NLU-RAG. Never Assist/HA control tools."""

from __future__ import annotations

from typing import Any

_PARSE = "klar.parse"
_ACT = "klar.act"


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
