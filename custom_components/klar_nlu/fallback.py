"""Chat-only fallback. Prompt text is not a tool lock."""

from __future__ import annotations

# homeassistant.components.conversation.ConversationEntityFeature.CONTROL
_CONTROL = 1

_CHAT_ONLY = {
    "de": (
        "Du antwortest nur im Gespräch. Steuere keine Geräte "
        "und rufe keine Home-Assistant-Werkzeuge auf."
    ),
    "en": (
        "Reply in conversation only. Do not control devices "
        "and do not call Home Assistant tools."
    ),
}


def agent_has_home_control(features: object) -> bool:
    try:
        flag = int(features)  # type: ignore[arg-type]
    except (TypeError, ValueError):
        return True
    return bool(flag & _CONTROL)


def chat_only_prompt(pack: str, extra: str | None) -> str:
    only = _CHAT_ONLY.get(pack, _CHAT_ONLY["de"])
    extra = (extra or "").strip()
    return f"{extra}\n{only}" if extra else only


def can_use_fallback_agent(controls_home: bool, chat: bool) -> bool:
    return (not controls_home) or chat
