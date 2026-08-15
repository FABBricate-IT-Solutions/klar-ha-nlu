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


_NEWS = {
    "de": (
        "Der Nutzer will aktuelle Nachrichten. "
        "Fasse die folgenden Schlagzeilen in drei bis fünf kurzen Sätzen zusammen. "
        "Erfinde keine Meldungen. "
        "Wenn du Websuche oder Nachrichten-Werkzeuge hast, nutze sie. "
        "Frage am Ende, ob der Nutzer zu einer Meldung mehr erfahren möchte."
    ),
    "en": (
        "The user wants current news. "
        "Summarize the following headlines in three to five short sentences. "
        "Do not invent stories. "
        "If you have web search or news tools, use them. "
        "End by asking whether they want more detail on any story."
    ),
}

_NEWS_FOLLOW = {
    "de": "Bleib beim Nachrichtenthema. Antworte knapp. Steuere keine Geräte.",
    "en": "Stay on the news topic. Keep it short. Do not control devices.",
}


def chat_only_prompt(pack: str, extra: str | None) -> str:
    only = _CHAT_ONLY.get(pack, _CHAT_ONLY["de"])
    extra = (extra or "").strip()
    return f"{extra}\n{only}" if extra else only


def news_prompt(pack: str, headlines: list[str], extra: str | None) -> str:
    body = _NEWS.get(pack, _NEWS["de"])
    if headlines:
        lines = "\n".join(f"- {item}" for item in headlines)
        label = "Schlagzeilen" if pack == "de" else "Headlines"
        body = f"{body}\n\n{label}:\n{lines}"
    return chat_only_prompt(pack, _join_extra(extra, body))


def news_followup_prompt(pack: str, extra: str | None) -> str:
    stay = _NEWS_FOLLOW.get(pack, _NEWS_FOLLOW["de"])
    return chat_only_prompt(pack, _join_extra(extra, stay))


def _join_extra(extra: str | None, body: str) -> str:
    extra = (extra or "").strip()
    return f"{extra}\n{body}" if extra else body


def can_use_fallback_agent(controls_home: bool, chat: bool = False) -> bool:
    del chat
    return not controls_home
