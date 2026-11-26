"""User policy actions that run in Home Assistant after Klar decides."""

from __future__ import annotations

import logging
from typing import Any

_LOGGER = logging.getLogger(__name__)

_MAX_RENDER = 500

# H-24: these rejects stay with Klar; everything else continues to HA then LLM.
_SKIP_HANDOFF = frozenset({"empty_input", "invalid_input", "unsafe"})


def hit_and_payload(payload: dict[str, Any]) -> tuple[str, str]:
    trace = payload.get("policy_trace") if isinstance(payload.get("policy_trace"), dict) else {}
    hit = str(trace.get("hit") or "").strip()
    action = str(trace.get("payload") or "").strip()
    return hit, action


def skips_llm_fallback(hit: str) -> bool:
    return hit in {"reply", "template", "script"}


def continues_after_reject(decision_type: str, reason: str) -> bool:
    """True when a Klar rejection should leave Klar (default agent, then LLM)."""
    if decision_type != "reject":
        return False
    return reason not in _SKIP_HANDOFF


def routes_general_chat(decision_type: str, reason: str) -> bool:
    """Send real chat, and Klar rejections, toward the LLM after HA handoff.

    H-24: ``no_action`` and other rejections continue (default agent first in
    conversation.py, then the LLM). Empty, invalid, and unsafe input stays put.
    """
    if decision_type == "chat":
        return True
    return continues_after_reject(decision_type, reason)


def keeps_engine_chat(hit: str, chat: bool, speech: str) -> bool:
    """Clock and other household replies already have speech — do not ask the LLM."""
    return bool(chat and speech.strip() and hit not in {"llm", "template"})


async def render_user_template(hass: Any, raw: str, text: str, exposed: Any | None = None) -> str | None:
    from homeassistant.exceptions import TemplateError
    from homeassistant.helpers.template import Template

    source = raw.strip()
    if not source:
        return None
    rooms: list[dict[str, Any]] = []
    try:
        from .floor_query import area_temperature_context

        visible = exposed if callable(exposed) else (lambda _id: True)
        rooms = area_temperature_context(hass, visible)
    except ImportError:
        rooms = []
    try:
        rendered = Template(source, hass).async_render({"text": text, "rooms": rooms}, parse_result=False)
    except (TemplateError, ValueError, TypeError) as err:
        _LOGGER.warning("Policy-Template fehlgeschlagen: %s", err)
        return None
    out = str(rendered).strip()
    return out[:_MAX_RENDER] if out else None
