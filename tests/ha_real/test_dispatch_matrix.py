"""Dispatch matrix: every switchable domain times every generic intent, on real services.

The stub tests replace ``hass.services`` with a mock, so a call to a service Home
Assistant does not have (``lock.turn_on``) looked like success. Here the entity sits in
its real entity component and the call goes through the real service registry.

A cell names the service that must run for the entity. ``UNSPECIFIED`` cells have no
agreed mapping yet (toggling a lock, switching a scene off); for those the only rule is
that Klar never asks Home Assistant for a service it does not have. That rule holds for
every cell.

``KNOWN_BUGS`` held package-2.8 expected failures (H-02); it is empty now that the
typed dispatch table is in place.
"""

from __future__ import annotations

from typing import Any

import pytest
from engine_fake import FakeEngine, intent
from ha_entities import add_entities, one_of_each
from homeassistant.core import HomeAssistant
from pytest_homeassistant_custom_component.common import MockConfigEntry
from support import ServiceLog, spoken_input

from custom_components.klar_nlu import dispatch

ON, OFF, TOGGLE = "HassTurnOn", "HassTurnOff", "HassToggle"
UNSPECIFIED = None

# Home Assistant's own on/off intent maps cover, lock, valve and button the same way
# (homeassistant/components/intent: on = open / lock / press, off = close / unlock).
EXPECTED: dict[str, dict[str, str | None]] = {
    "light": {ON: "light.turn_on", OFF: "light.turn_off", TOGGLE: "light.toggle"},
    "switch": {ON: "switch.turn_on", OFF: "switch.turn_off", TOGGLE: "switch.toggle"},
    "fan": {ON: "fan.turn_on", OFF: "fan.turn_off", TOGGLE: "fan.toggle"},
    "cover": {ON: "cover.open_cover", OFF: "cover.close_cover", TOGGLE: "cover.toggle"},
    "climate": {ON: "climate.turn_on", OFF: "climate.turn_off", TOGGLE: "climate.toggle"},
    "media_player": {ON: "media_player.turn_on", OFF: "media_player.turn_off", TOGGLE: "media_player.toggle"},
    "lock": {ON: "lock.lock", OFF: "lock.unlock", TOGGLE: UNSPECIFIED},
    "valve": {ON: "valve.open_valve", OFF: "valve.close_valve", TOGGLE: "valve.toggle"},
    "lawn_mower": {ON: "lawn_mower.start_mowing", OFF: "lawn_mower.dock", TOGGLE: UNSPECIFIED},
    "vacuum": {ON: "vacuum.start", OFF: "vacuum.return_to_base", TOGGLE: UNSPECIFIED},
    "button": {ON: "button.press", OFF: UNSPECIFIED, TOGGLE: UNSPECIFIED},
    "scene": {ON: "scene.turn_on", OFF: UNSPECIFIED, TOGGLE: UNSPECIFIED},
}

# Package 2.8 cleared H-02: the typed dispatch table maps each domain to a real service
# (or refuses the step when there is no agreed mapping).
KNOWN_BUGS: dict[tuple[str, str], str] = {}


def _cells() -> list[Any]:
    cells: list[Any] = []
    for domain, row in EXPECTED.items():
        for name, service in row.items():
            finding = KNOWN_BUGS.get((domain, name))
            marks = (
                [pytest.mark.xfail(strict=True, reason=f"{finding}: fixed by package 2.8 (typed dispatch table)")]
                if finding
                else []
            )
            cells.append(pytest.param(domain, name, service, id=f"{domain}-{name}", marks=marks))
    return cells


@pytest.mark.parametrize(("domain", "name", "service"), _cells())
async def test_dispatch_matrix(
    hass: HomeAssistant,
    klar: MockConfigEntry,
    engine: FakeEngine,
    service_log: ServiceLog,
    domain: str,
    name: str,
    service: str | None,
) -> None:
    entity = one_of_each()[domain]
    await add_entities(hass, domain, entity)
    entity_id = entity.entity_id
    assert entity_id.startswith(f"{domain}.")
    item = intent(name, domain=domain, entity_id=entity_id)

    result = await dispatch.handle_intent(
        hass, spoken_input("matrix"), item, "en", "conversation", lambda _entity_id: True
    )
    await hass.async_block_till_done()

    assert service_log.unregistered() == [], (
        f"{name} on {entity_id} asked Home Assistant for a service that does not exist"
    )
    if service is UNSPECIFIED:
        return
    assert service_log.ran(entity_id) == [service]
    assert entity.calls, f"{service} did not reach {entity_id}"
    assert result.ok, result.error
    assert result.speech == engine.spoken
