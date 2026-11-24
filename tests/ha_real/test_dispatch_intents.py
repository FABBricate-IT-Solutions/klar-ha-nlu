"""Intents with their own slots (brightness, position, speed, temperature) on real services."""

from __future__ import annotations

import pytest
from engine_fake import FakeEngine, intent
from ha_entities import FakeClimate, FakeCover, FakeFan, FakeLight, FakeSwitch, FakeVacuum, add_entities
from homeassistant.components.climate import HVACMode
from homeassistant.components.homeassistant.exposed_entities import async_expose_entity
from homeassistant.core import HomeAssistant
from pytest_homeassistant_custom_component.common import MockConfigEntry
from support import ServiceLog, spoken_input

from custom_components.klar_nlu import dispatch


async def _run(hass: HomeAssistant, item: dict, exposed: bool = True) -> dispatch.IntentStepResult:
    turn = spoken_input("intent")
    result = await dispatch.handle_intent(hass, turn, item, "en", "conversation", lambda _entity_id: exposed)
    await hass.async_block_till_done()
    return result


async def test_light_brightness_reaches_the_light(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, service_log: ServiceLog
) -> None:
    light = FakeLight("Desk Lamp", "light-1")
    await add_entities(hass, "light", light)
    result = await _run(hass, intent("HassLightSet", entity_id=light.entity_id, brightness=40))
    assert result.ok, result.error
    assert service_log.ran(light.entity_id) == ["light.turn_on"]
    assert light.call_data == [{"brightness": 102}]


async def test_vacuum_start_and_return(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, service_log: ServiceLog
) -> None:
    vacuum = FakeVacuum("Robo", "vacuum-1")
    await add_entities(hass, "vacuum", vacuum)
    async_expose_entity(hass, "conversation", vacuum.entity_id, True)
    assert (await _run(hass, intent("HassVacuumStart", entity_id=vacuum.entity_id))).ok
    assert (await _run(hass, intent("HassVacuumReturnToBase", entity_id=vacuum.entity_id))).ok
    assert vacuum.calls == ["start", "return_to_base"]
    assert service_log.unregistered() == []


async def test_cover_position(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, service_log: ServiceLog
) -> None:
    cover = FakeCover("Living Room Blinds", "cover-1")
    await add_entities(hass, "cover", cover)
    async_expose_entity(hass, "conversation", cover.entity_id, True)
    result = await _run(hass, intent("HassSetPosition", domain="cover", entity_id=cover.entity_id, position=30))
    assert result.ok, result.error
    assert cover.calls == ["set_cover_position"]
    assert cover.call_data == [{"position": 30}]


async def test_fan_speed(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, service_log: ServiceLog
) -> None:
    fan = FakeFan("Ceiling Fan", "fan-1")
    await add_entities(hass, "fan", fan)
    async_expose_entity(hass, "conversation", fan.entity_id, True)
    result = await _run(hass, intent("HassFanSetSpeed", entity_id=fan.entity_id, percentage=50))
    assert result.ok, result.error
    assert fan.calls == ["turn_on"]
    assert fan.call_data == [{"percentage": 50}]


@pytest.mark.xfail(
    strict=True,
    reason="H-10: the intent is resolved again by name and fails on duplicate names; fixed by package 2.8",
)
async def test_two_fans_with_one_name_reach_the_chosen_one(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, service_log: ServiceLog
) -> None:
    first, second = FakeFan("Fan", "fan-1"), FakeFan("Fan", "fan-2")
    await add_entities(hass, "fan", first, second)
    for fan in (first, second):
        async_expose_entity(hass, "conversation", fan.entity_id, True)
    assert first.entity_id != second.entity_id
    result = await _run(hass, intent("HassFanSetSpeed", entity_id=second.entity_id, percentage=50))
    assert result.ok, result.error
    assert second.calls == ["turn_on"]
    assert first.calls == []


async def test_heating_thermostat_takes_a_target(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, service_log: ServiceLog
) -> None:
    thermostat = FakeClimate("Thermostat", "climate-1", [HVACMode.OFF, HVACMode.HEAT], HVACMode.HEAT)
    await add_entities(hass, "climate", thermostat)
    result = await _run(hass, intent("HassClimateSetTemperature", entity_id=thermostat.entity_id, temperature=21))
    assert result.ok, result.error
    assert thermostat.calls == ["set_temperature"]
    assert thermostat.call_data[0]["temperature"] == 21.0


@pytest.mark.xfail(
    strict=True,
    reason="H-11: set temperature forces hvac_mode heat, also on cool-only units; fixed by package 2.8",
)
async def test_cool_only_unit_is_not_forced_to_heat(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, service_log: ServiceLog
) -> None:
    unit = FakeClimate("Air Conditioner", "climate-2", [HVACMode.OFF, HVACMode.COOL], HVACMode.OFF)
    await add_entities(hass, "climate", unit)
    result = await _run(hass, intent("HassClimateSetTemperature", entity_id=unit.entity_id, temperature=24))
    sent = [data for name, data, _known in service_log.asked if name == "climate.set_temperature"]
    assert [data.get("hvac_mode") for data in sent] == [None]
    assert result.ok, result.error


async def test_unexposed_entity_is_refused_before_any_service(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, service_log: ServiceLog
) -> None:
    switch = FakeSwitch("Coffee Maker", "switch-1")
    await add_entities(hass, "switch", switch)
    result = await _run(hass, intent("HassTurnOn", entity_id=switch.entity_id), exposed=False)
    assert not result.ok
    assert result.error == "entity_not_exposed"
    assert service_log.asked == []
    assert switch.calls == []


@pytest.mark.xfail(
    strict=True,
    reason="A-17: direct service calls carry no user context; fixed by package 2.8",
)
async def test_service_calls_carry_the_callers_context(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, service_log: ServiceLog
) -> None:
    switch = FakeSwitch("Coffee Maker", "switch-1")
    await add_entities(hass, "switch", switch)
    turn = spoken_input("turn on the coffee maker")
    result = await dispatch.handle_intent(
        hass, turn, intent("HassTurnOn", entity_id=switch.entity_id), "en", "conversation", lambda _entity_id: True
    )
    assert result.ok, result.error
    assert service_log.contexts == [turn.context]
