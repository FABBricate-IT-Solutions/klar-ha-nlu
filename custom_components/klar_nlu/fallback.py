"""HA fallback glue. Product prompts live in the engine."""

from __future__ import annotations

from uuid import uuid4

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
    """Key for one conversation's LLM history. No key means no shared history."""
    key = (session_key or "").strip() or f"once-{uuid4().hex}"
    return f"klar-llm-{key}"[:128]


def forget_old_llm_sessions(store: dict[str, list[tuple[str, str]]], keep: int = 64) -> None:
    """Drop the oldest conversations so per-conversation history cannot grow without end."""
    while len(store) > keep:
        store.pop(next(iter(store)))


_MAX_TURN_CHARS = 600
_MAX_HISTORY_CHARS = 2400


def append_llm_turn(
    turns: list[tuple[str, str]] | None, user: str, assistant: str, keep: int = 8
) -> list[tuple[str, str]]:
    """Keep recent turns, but cap size so article text cannot blow later prompts (A-15)."""
    out = list(turns or [])
    user, assistant = user.strip()[:_MAX_TURN_CHARS], assistant.strip()[:_MAX_TURN_CHARS]
    if user or assistant:
        out.append((user, assistant))
    out = out[-keep:]
    while len(out) > 1 and sum(len(pair[0]) + len(pair[1]) for pair in out) > _MAX_HISTORY_CHARS:
        out.pop(0)
    return out


def can_use_fallback_agent(
    controls_home: bool, chat: bool = False, allow_tools: bool = False
) -> bool:
    del chat
    return allow_tools or not controls_home
