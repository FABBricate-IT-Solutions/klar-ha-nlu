"""Hand reminder sentences to Home Assistant when Klar has no reminder intent.

The voice-reminder package registers ``SetReminder*`` intent scripts on the
default conversation agent. Klar used to read "in 20 minutes" as a brightness
and never reached those scripts.
"""

from __future__ import annotations

import re
from typing import Any

from .passthrough import async_delegate_default_agent, delegation_accepted

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
    # H-12: SetReminder* is not a Home Assistant intent — always hand the turn over.
    if any(_is_reminder_intent(item) for item in intents):
        return True
    return reminder_shaped(text)


async def async_delegate_reminder(hass: Any, user_input: Any) -> Any | None:
    return await async_delegate_default_agent(hass, user_input)


def _is_reminder_intent(item: Any) -> bool:
    if not isinstance(item, dict):
        return False
    return str(item.get("name") or "") in _REMINDER_INTENTS


__all__ = [
    "async_delegate_reminder",
    "delegation_accepted",
    "reminder_shaped",
    "should_delegate",
]
