"""The ChatLog path: turns enter through Home Assistant's conversation API.

``conversation.async_converse`` opens the chat session and the chat log the same way
Assist does, then calls the Klar agent. The stub tests hand the agent a mock chat log.
"""

from __future__ import annotations

import pytest
from engine_fake import FakeEngine, intent
from ha_entities import FakeLock, FakeSwitch, add_entities
from homeassistant.components import conversation
from homeassistant.components.conversation import AssistantContent, ConversationResult, UserContent
from homeassistant.components.conversation.chat_log import DATA_CHAT_LOGS, ChatLog
from homeassistant.core import Context, HomeAssistant
from homeassistant.helpers import intent as ha_intent
from pytest_homeassistant_custom_component.common import MockConfigEntry

AGENT = "conversation.klar_nlu"


async def _say(
    hass: HomeAssistant, text: str, conversation_id: str | None = None, **source: str
) -> ConversationResult:
    result = await conversation.async_converse(
        hass, text, conversation_id, Context(), language="en", agent_id=AGENT, **source
    )
    await hass.async_block_till_done()
    return result


def _speech(result: ConversationResult) -> str:
    return str(result.response.speech["plain"]["speech"])


def _logs(hass: HomeAssistant) -> dict[str, ChatLog]:
    return hass.data.get(DATA_CHAT_LOGS, {})


async def _coffee_maker(hass: HomeAssistant, engine: FakeEngine) -> FakeSwitch:
    switch = FakeSwitch("Coffee Maker", "switch-1")
    await add_entities(hass, "switch", switch)
    engine.executes("turn on the coffee maker", [intent("HassTurnOn", entity_id=switch.entity_id)])
    return switch


async def test_executed_turn_is_written_to_the_chat_log(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    switch = await _coffee_maker(hass, engine)
    result = await _say(hass, "turn on the coffee maker", conversation_id="kitchen-tablet")

    assert switch.calls == ["turn_on"]
    assert _speech(result) == engine.spoken
    assert result.response.response_type is ha_intent.IntentResponseType.ACTION_DONE
    assert result.continue_conversation is False
    log = _logs(hass)["kitchen-tablet"]
    turn = log.content[-2:]
    assert isinstance(turn[0], UserContent)
    assert turn[0].content == "turn on the coffee maker"
    assert isinstance(turn[1], AssistantContent)
    assert turn[1].content == engine.spoken
    assert turn[1].agent_id == AGENT
    assert log.as_dict()["conversation_id"] == "kitchen-tablet"


async def test_second_turn_continues_the_same_log(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    await _coffee_maker(hass, engine)
    await _say(hass, "turn on the coffee maker", conversation_id="kitchen-tablet")
    await _say(hass, "turn on the coffee maker", conversation_id="kitchen-tablet")

    roles = [item.role for item in _logs(hass)["kitchen-tablet"].content]
    assert roles == ["system", "user", "assistant", "user", "assistant"]


async def test_confirmation_keeps_the_conversation_open(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    lock = FakeLock("Front Door", "lock-1")
    await add_entities(hass, "lock", lock)
    engine.asks_before(
        "unlock the front door",
        "Unlock Front Door?",
        [intent("HassTurnOff", domain="lock", entity_id=lock.entity_id)],
    )
    result = await _say(hass, "unlock the front door", conversation_id="hall-panel")

    assert _speech(result) == "Unlock Front Door?"
    assert result.continue_conversation is True
    assert lock.calls == []
    assert result.conversation_id == "hall-panel"


async def test_unreachable_engine_answers_in_the_chat_log(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    engine.reachable = False
    result = await _say(hass, "turn on the coffee maker", conversation_id="kitchen-tablet")

    assert _speech(result) == "Klar is not responding right now."
    last = _logs(hass)["kitchen-tablet"].content[-1]
    assert isinstance(last, AssistantContent)
    assert last.content == "Klar is not responding right now."


async def test_satellite_turn_uses_a_session_of_its_own(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    await _coffee_maker(hass, engine)
    await _say(hass, "turn on the coffee maker", satellite_id="assist_satellite.kitchen")
    await _say(hass, "turn on the coffee maker", satellite_id="assist_satellite.bedroom")

    first, second = engine.session_ids()
    assert first != second


@pytest.mark.xfail(
    strict=True,
    reason="H-01: typed Assist clients without a conversation id share one engine session; fixed by package 1.10",
)
async def test_typed_clients_without_an_id_do_not_share_a_session(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    lock = FakeLock("Front Door", "lock-1")
    await add_entities(hass, "lock", lock)
    engine.asks_before(
        "unlock the front door",
        "Unlock Front Door?",
        [intent("HassTurnOff", domain="lock", entity_id=lock.entity_id)],
    )
    await _say(hass, "unlock the front door")
    await _say(hass, "yes")

    first, second = engine.session_ids()
    assert first != second, "two clients reached the engine with the same session id"
    assert lock.calls == [], "a stranger's yes unlocked the door"


@pytest.mark.xfail(
    strict=True,
    reason="H-01: the reply names the shared engine session, not Home Assistant's chat session; fixed by package 1.10",
)
async def test_reply_names_the_chat_session_home_assistant_opened(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    await _coffee_maker(hass, engine)
    result = await _say(hass, "turn on the coffee maker")

    assert result.conversation_id in _logs(hass)
