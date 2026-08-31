"""User policy actions that run in Home Assistant after Klar decides."""

from __future__ import annotations

import logging
from typing import Any

_LOGGER = logging.getLogger(__name__)

_MAX_RENDER = 500


def hit_and_payload(payload: dict[str, Any]) -> tuple[str, str]:
    trace = payload.get("policy_trace") if isinstance(payload.get("policy_trace"), dict) else {}
    hit = str(trace.get("hit") or "").strip()
    action = str(trace.get("payload") or "").strip()
    return hit, action


def skips_llm_fallback(hit: str) -> bool:
    return hit in {"reply", "template", "script"}


def keeps_engine_chat(hit: str, chat: bool, speech: str) -> bool:
    """Clock and other household replies already have speech — do not ask the LLM."""
    return bool(chat and speech.strip() and hit not in {"llm", "template"})


async def render_user_template(hass: Any, raw: str, text: str) -> str | None:
    from homeassistant.exceptions import TemplateError
    from homeassistant.helpers.template import Template

    source = raw.strip()
    if not source:
        return None
    try:
        rendered = Template(source, hass).async_render({"text": text}, parse_result=False)
    except (TemplateError, ValueError, TypeError) as err:
        _LOGGER.warning("Policy-Template fehlgeschlagen: %s", err)
        return None
    out = str(rendered).strip()
    return out[:_MAX_RENDER] if out else None
