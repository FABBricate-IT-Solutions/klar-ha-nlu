"""Hand reminder sentences to Home Assistant when Klar has no reminder intent.

The voice-reminder package registers ``SetReminder*`` intent scripts on the
default conversation agent. Klar used to read "in 20 minutes" as a brightness
and never reached those scripts.
"""

from __future__ import annotations

import re
from dataclasses import replace
from typing import Any

# Home Assistant is imported inside the delegate function so stdlib tests can
# load this module without a Home Assistant install.

_REMINDER = re.compile(r"\b(remind(?:er|ers)?|erinner\w*)\b", re.IGNORECASE)
_REMINDER_INTENTS = {
    "SetReminderRelative",
    "SetReminderAbsolute",
    "SetReminderDaypart",
    "OpenReminderList",
    "CancelReminder",
    "CancelNextReminder",
    "CancelAllReminders",
}


def reminder_shaped(text: str) -> bool:
    return bool(_REMINDER.search(text or ""))


def should_delegate(text: str, intents: list[Any]) -> bool:
    if not reminder_shaped(text):
        return False
    return not any(_is_reminder_intent(item) for item in intents)


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


async def async_delegate_reminder(hass: Any, user_input: Any) -> Any | None:
    try:
        # Unit tests load this file without Home Assistant installed.
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


def _is_reminder_intent(item: Any) -> bool:
    if not isinstance(item, dict):
        return False
    return str(item.get("name") or "") in _REMINDER_INTENTS
