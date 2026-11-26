"""Hand a turn to Home Assistant's default conversation agent.

H-24: after Klar rejects, Custom Sentences and the stock HA agent get a chance
before the LLM. Reminder scripts and media intents live there.
"""

from __future__ import annotations

from dataclasses import replace
from typing import Any

# Home Assistant is imported inside the delegate so stdlib tests can load this
# module without a Home Assistant install.


def delegation_accepted(result: Any) -> bool:
    response = getattr(result, "response", None)
    if response is None:
        return False
    kind = getattr(response, "response_type", None)
    name = str(getattr(kind, "value", kind) or "")
    if name in {"error", "action_incomplete"}:
        return False
    speech = getattr(response, "speech", None) or {}
    plain = speech.get("plain") if isinstance(speech, dict) else {}
    return bool(str((plain or {}).get("speech") or "").strip())


async def async_delegate_default_agent(hass: Any, user_input: Any) -> Any | None:
    """Hand the turn to ``conversation.home_assistant`` when it accepts."""
    try:
        from homeassistant.components import conversation as ha_conversation
    except ImportError:
        return None
    agent = ha_conversation.async_get_agent(hass, "conversation.home_assistant")
    if agent is None or type(agent).__name__.startswith("Klar"):
        return None
    try:
        forwarded = replace(user_input, agent_id="conversation.home_assistant")
        result = await agent.async_process(forwarded)
    except Exception:  # noqa: BLE001 — default agent is a system boundary
        return None
    if not delegation_accepted(result):
        return None
    return result
