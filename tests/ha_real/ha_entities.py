"""Real Home Assistant entities, one per domain Klar may be asked to switch.

Each entity is added to its real entity component, so the services are the ones Home
Assistant registers for that domain and feature checks apply. The entity only records
which of its methods ran.
"""

from __future__ import annotations

from typing import Any

from homeassistant.components.button import ButtonEntity
from homeassistant.components.climate import ClimateEntity, ClimateEntityFeature, HVACMode
from homeassistant.components.cover import CoverEntity, CoverEntityFeature
from homeassistant.components.fan import FanEntity, FanEntityFeature
from homeassistant.components.lawn_mower import LawnMowerActivity, LawnMowerEntity, LawnMowerEntityFeature
from homeassistant.components.light import ColorMode, LightEntity
from homeassistant.components.lock import LockEntity
from homeassistant.components.media_player import MediaPlayerEntity, MediaPlayerEntityFeature, MediaPlayerState
from homeassistant.components.scene import Scene
from homeassistant.components.switch import SwitchEntity
from homeassistant.components.vacuum import StateVacuumEntity, VacuumActivity, VacuumEntityFeature
from homeassistant.components.valve import ValveEntity, ValveEntityFeature
from homeassistant.const import UnitOfTemperature
from homeassistant.core import HomeAssistant
from homeassistant.helpers.entity import Entity
from homeassistant.setup import async_setup_component


class Recorder(Entity):
    """Base for the fake entities: remember entity method calls in order."""

    def __init__(self, name: str, unique_id: str) -> None:
        self._attr_name = name
        self._attr_unique_id = unique_id
        self.calls: list[str] = []
        self.call_data: list[dict[str, Any]] = []

    def note(self, call: str, **data: Any) -> None:
        self.calls.append(call)
        self.call_data.append(data)
        self.async_write_ha_state()


class FakeLight(Recorder, LightEntity):
    _attr_color_mode = ColorMode.BRIGHTNESS
    _attr_supported_color_modes = {ColorMode.BRIGHTNESS}
    _attr_is_on = False

    async def async_turn_on(self, **kwargs: Any) -> None:
        self._attr_is_on = True
        self.note("turn_on", **kwargs)

    async def async_turn_off(self, **kwargs: Any) -> None:
        self._attr_is_on = False
        self.note("turn_off")


class FakeSwitch(Recorder, SwitchEntity):
    _attr_is_on = False

    async def async_turn_on(self, **kwargs: Any) -> None:
        self._attr_is_on = True
        self.note("turn_on")

    async def async_turn_off(self, **kwargs: Any) -> None:
        self._attr_is_on = False
        self.note("turn_off")


class FakeFan(Recorder, FanEntity):
    _attr_supported_features = (
        FanEntityFeature.SET_SPEED | FanEntityFeature.TURN_ON | FanEntityFeature.TURN_OFF
    )
    _attr_percentage = 0

    async def async_turn_on(
        self, percentage: int | None = None, preset_mode: str | None = None, **kwargs: Any
    ) -> None:
        self._attr_percentage = percentage or 100
        self.note("turn_on", percentage=percentage)

    async def async_turn_off(self, **kwargs: Any) -> None:
        self._attr_percentage = 0
        self.note("turn_off")

    async def async_set_percentage(self, percentage: int) -> None:
        self._attr_percentage = percentage
        self.note("set_percentage", percentage=percentage)


class FakeCover(Recorder, CoverEntity):
    _attr_supported_features = (
        CoverEntityFeature.OPEN | CoverEntityFeature.CLOSE | CoverEntityFeature.SET_POSITION
    )
    _attr_is_closed = True

    async def async_open_cover(self, **kwargs: Any) -> None:
        self._attr_is_closed = False
        self.note("open_cover")

    async def async_close_cover(self, **kwargs: Any) -> None:
        self._attr_is_closed = True
        self.note("close_cover")

    async def async_set_cover_position(self, **kwargs: Any) -> None:
        self.note("set_cover_position", **kwargs)


class FakeLock(Recorder, LockEntity):
    _attr_is_locked = False

    async def async_lock(self, **kwargs: Any) -> None:
        self._attr_is_locked = True
        self.note("lock")

    async def async_unlock(self, **kwargs: Any) -> None:
        self._attr_is_locked = False
        self.note("unlock")


class FakeValve(Recorder, ValveEntity):
    _attr_supported_features = ValveEntityFeature.OPEN | ValveEntityFeature.CLOSE
    _attr_reports_position = False
    _attr_is_closed = True

    async def async_open_valve(self) -> None:
        self._attr_is_closed = False
        self.note("open_valve")

    async def async_close_valve(self) -> None:
        self._attr_is_closed = True
        self.note("close_valve")


class FakeMower(Recorder, LawnMowerEntity):
    _attr_supported_features = LawnMowerEntityFeature.START_MOWING | LawnMowerEntityFeature.DOCK
    _attr_activity = LawnMowerActivity.DOCKED

    async def async_start_mowing(self) -> None:
        self._attr_activity = LawnMowerActivity.MOWING
        self.note("start_mowing")

    async def async_dock(self) -> None:
        self._attr_activity = LawnMowerActivity.DOCKED
        self.note("dock")


class FakeVacuum(Recorder, StateVacuumEntity):
    _attr_supported_features = (
        VacuumEntityFeature.START | VacuumEntityFeature.RETURN_HOME | VacuumEntityFeature.STATE
    )
    _attr_activity = VacuumActivity.DOCKED

    async def async_start(self) -> None:
        self._attr_activity = VacuumActivity.CLEANING
        self.note("start")

    async def async_return_to_base(self, **kwargs: Any) -> None:
        self._attr_activity = VacuumActivity.RETURNING
        self.note("return_to_base")


class FakeButton(Recorder, ButtonEntity):
    async def async_press(self) -> None:
        self.note("press")


class FakeScene(Recorder, Scene):
    async def async_activate(self, **kwargs: Any) -> None:
        self.note("turn_on")


class FakeClimate(Recorder, ClimateEntity):
    _attr_temperature_unit = UnitOfTemperature.CELSIUS
    _attr_supported_features = (
        ClimateEntityFeature.TARGET_TEMPERATURE | ClimateEntityFeature.TURN_ON | ClimateEntityFeature.TURN_OFF
    )
    _attr_current_temperature = 20
    _attr_target_temperature = 19

    def __init__(self, name: str, unique_id: str, modes: list[HVACMode], mode: HVACMode) -> None:
        super().__init__(name, unique_id)
        self._attr_hvac_modes = modes
        self._attr_hvac_mode = mode

    async def async_set_temperature(self, **kwargs: Any) -> None:
        self._attr_target_temperature = kwargs["temperature"]
        self.note("set_temperature", **kwargs)

    async def async_set_hvac_mode(self, hvac_mode: HVACMode) -> None:
        self._attr_hvac_mode = hvac_mode
        self.note("set_hvac_mode", hvac_mode=hvac_mode)

    async def async_turn_on(self) -> None:
        self.note("turn_on")

    async def async_turn_off(self) -> None:
        self._attr_hvac_mode = HVACMode.OFF
        self.note("turn_off")


class FakePlayer(Recorder, MediaPlayerEntity):
    _attr_supported_features = MediaPlayerEntityFeature.TURN_ON | MediaPlayerEntityFeature.TURN_OFF
    _attr_state = MediaPlayerState.OFF

    async def async_turn_on(self) -> None:
        self._attr_state = MediaPlayerState.ON
        self.note("turn_on")

    async def async_turn_off(self) -> None:
        self._attr_state = MediaPlayerState.OFF
        self.note("turn_off")


def one_of_each() -> dict[str, Recorder]:
    """One entity per domain, keyed by domain."""
    return {
        "light": FakeLight("Desk Lamp", "light-1"),
        "switch": FakeSwitch("Coffee Maker", "switch-1"),
        "fan": FakeFan("Ceiling Fan", "fan-1"),
        "cover": FakeCover("Garage Door", "cover-1"),
        "lock": FakeLock("Front Door", "lock-1"),
        "valve": FakeValve("Garden Valve", "valve-1"),
        "lawn_mower": FakeMower("Mowy", "mower-1"),
        "vacuum": FakeVacuum("Robo", "vacuum-1"),
        "button": FakeButton("Doorbell Chime", "button-1"),
        "scene": FakeScene("Movie Night", "scene-1"),
        "climate": FakeClimate("Thermostat", "climate-1", [HVACMode.OFF, HVACMode.HEAT], HVACMode.HEAT),
        "media_player": FakePlayer("Kitchen Speaker", "player-1"),
    }


async def add_entities(hass: HomeAssistant, domain: str, *entities: Entity) -> None:
    """Add entities to the real entity component of ``domain``."""
    if domain not in hass.config.components:
        assert await async_setup_component(hass, domain, {})
        await hass.async_block_till_done()
    await hass.data[domain].async_add_entities(list(entities))
    await hass.async_block_till_done()
