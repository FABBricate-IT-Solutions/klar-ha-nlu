"""HA fallback glue. Product prompts live in the engine."""

from __future__ import annotations

import re

# homeassistant.components.conversation.ConversationEntityFeature.CONTROL
_CONTROL = 1


def agent_has_home_control(features: object) -> bool:
    try:
        flag = int(features)  # type: ignore[arg-type]
    except (TypeError, ValueError):
        return True
    return bool(flag & _CONTROL)


def calendar_query_only(intents: list | None) -> bool:
    if not intents:
        return False
    return all(item.get("name") == "KlarGetCalendarEvents" for item in intents)


def calendar_readback(pack: str, facts: str) -> str:
    events = (facts or "").strip() or (
        "Keine Termine." if pack == "de" or pack.startswith("de-") else "No events."
    )
    if pack == "de" or pack.startswith("de-"):
        return f"Lies nur diese Kalendertermine vor:\n{events}"
    return f"Read back only these calendar events:\n{events}"


def llm_conversation_id(session_key: str) -> str:
    key = (session_key or "klar-followup").strip() or "klar-followup"
    return f"klar-llm-{key}"[:128]


def append_llm_turn(
    turns: list[tuple[str, str]] | None, user: str, assistant: str, keep: int = 8
) -> list[tuple[str, str]]:
    out = list(turns or [])
    user, assistant = user.strip(), assistant.strip()
    if user or assistant:
        out.append((user, assistant))
    return out[-keep:]


def can_use_fallback_agent(
    controls_home: bool, chat: bool = False, allow_tools: bool = False
) -> bool:
    del chat
    return allow_tools or not controls_home
