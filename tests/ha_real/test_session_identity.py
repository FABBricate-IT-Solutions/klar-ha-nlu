"""H-01: the engine session comes from Home Assistant, never from a constant.

Before package 1.10 every turn without a conversation id reached the engine as
``klar-followup``. One person's pending confirmation could then be answered by anyone
else who also sent no id.
"""

from __future__ import annotations

from engine_fake import FakeEngine, intent, reject
from ha_entities import FakeSwitch, add_entities
from homeassistant.components import conversation
from homeassistant.components.conversation import ConversationResult
from homeassistant.components.conversation.chat_log import DATA_CHAT_LOGS
from homeassistant.core import Context, HomeAssistant
from homeassistant.util import ulid as ulid_util
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


async def _pump_that_asks_first(hass: HomeAssistant, engine: FakeEngine) -> FakeSwitch:
    pump = FakeSwitch("Garden Pump", "switch-pump")
    await add_entities(hass, "switch", pump)
    engine.asks_before(
        "start the garden pump", "Start Garden Pump?", [intent("HassTurnOn", entity_id=pump.entity_id)]
    )
    return pump


async def _coffee_maker(hass: HomeAssistant, engine: FakeEngine) -> FakeSwitch:
    switch = FakeSwitch("Coffee Maker", "switch-1")
    await add_entities(hass, "switch", switch)
    engine.executes("turn on the coffee maker", [intent("HassTurnOn", entity_id=switch.entity_id)])
    return switch


async def test_second_user_without_an_id_cannot_answer_the_first_users_question(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    pump = await _pump_that_asks_first(hass, engine)
    asked = await _say(hass, "start the garden pump")
    answered = await _say(hass, "yes")

    first, second = engine.session_ids()
    assert first and second
    assert first != second, "two clients reached the engine with the same session id"
    assert pump.calls == [], "a stranger's yes started the pump"
    assert asked.continue_conversation is True
    assert answered.conversation_id != asked.conversation_id


async def test_first_user_can_still_answer_with_the_id_klar_returned(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    pump = await _pump_that_asks_first(hass, engine)
    asked = await _say(hass, "start the garden pump")
    await _say(hass, "yes")
    assert pump.calls == []

    await _say(hass, "yes", conversation_id=asked.conversation_id)

    assert pump.calls == ["turn_on"]
    assert engine.session_ids()[0] == engine.session_ids()[2] == asked.conversation_id


async def test_reply_names_the_chat_session_home_assistant_opened(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    await _coffee_maker(hass, engine)
    result = await _say(hass, "turn on the coffee maker")

    assert result.conversation_id in hass.data[DATA_CHAT_LOGS]
    assert engine.session_ids() == [result.conversation_id]


async def test_idle_conversation_id_follows_the_new_chat_session(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    """Home Assistant replaces an id it no longer knows; the engine session follows."""
    await _coffee_maker(hass, engine)
    stale = ulid_util.ulid_now()
    result = await _say(hass, "turn on the coffee maker", conversation_id=stale)

    assert result.conversation_id != stale
    assert engine.session_ids() == [result.conversation_id]


async def test_satellite_without_an_id_keeps_one_session(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    pump = await _pump_that_asks_first(hass, engine)
    await _say(hass, "start the garden pump", satellite_id="assist_satellite.kitchen")
    await _say(hass, "yes", satellite_id="assist_satellite.bedroom")
    assert pump.calls == []

    await _say(hass, "yes", satellite_id="assist_satellite.kitchen")

    assert pump.calls == ["turn_on"]
    kitchen, bedroom, kitchen_again = engine.session_ids()
    assert kitchen == kitchen_again != bedroom


async def test_undo_service_reaches_the_conversation_that_acted_last(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    """The service used to rely on the shared session; now it names the one that acted."""
    await _coffee_maker(hass, engine)
    engine.on("undo that", lambda _engine, body: reject(body, "nothing_to_undo", "Nothing to undo."))
    acted = await _say(hass, "turn on the coffee maker")
    await _say(hass, "undo that")

    await hass.services.async_call("klar_nlu", "undo", {}, blocking=True)

    acting, other, service = engine.session_ids()
    assert acting == acted.conversation_id
    assert other != acting
    assert service == acting
    assert hass.data["klar_nlu"][klar.entry_id]["service_turns"] == {}


async def test_undo_service_before_any_turn_gets_a_session_of_its_own(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    engine.on("undo that", lambda _engine, body: reject(body, "nothing_to_undo", "Nothing to undo."))

    await hass.services.async_call("klar_nlu", "undo", {}, blocking=True)
    await hass.services.async_call("klar_nlu", "undo", {}, blocking=True)

    first, second = engine.session_ids()
    assert first and second and first != second
