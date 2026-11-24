"""Small helpers shared by the real-Home-Assistant tests."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any

from homeassistant.components.conversation import ConversationInput
from homeassistant.core import Context


@dataclass
class ServiceLog:
    """Every service Klar asked Home Assistant for, registered or not."""

    asked: list[tuple[str, dict[str, Any], bool]] = field(default_factory=list)
    contexts: list[Context | None] = field(default_factory=list)

    def ran(self, entity_id: str) -> list[str]:
        """Registered services called for one entity, in order."""
        return [name for name, data, known in self.asked if known and data.get("entity_id") == entity_id]

    def unregistered(self) -> list[str]:
        """Services Klar asked for that Home Assistant does not have."""
        return [name for name, _data, known in self.asked if not known]


def spoken_input(
    text: str,
    conversation_id: str | None = None,
    device_id: str | None = None,
    satellite_id: str | None = None,
) -> ConversationInput:
    """A conversation turn as Home Assistant hands it to the agent."""
    return ConversationInput(
        text=text,
        context=Context(),
        conversation_id=conversation_id,
        device_id=device_id,
        satellite_id=satellite_id,
        language="en",
        agent_id="conversation.klar_nlu",
    )
