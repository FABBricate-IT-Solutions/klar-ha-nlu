from __future__ import annotations

from homeassistant.components.switch import SwitchEntity
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers.device_registry import DeviceInfo
from homeassistant.helpers.entity import EntityCategory
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .const import CONF_QUIET_ACK, DEFAULT_QUIET_ACK, DOMAIN


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    async_add_entities([KlarQuietAckSwitch(entry)])


class KlarQuietAckSwitch(SwitchEntity):
    _attr_has_entity_name = True
    _attr_translation_key = "quiet_ack"
    _attr_entity_category = EntityCategory.CONFIG
    _attr_icon = "mdi:bell-check"

    def __init__(self, entry: ConfigEntry) -> None:
        self._entry = entry
        self._attr_unique_id = f"{entry.entry_id}_quiet_ack"
        self._attr_device_info = DeviceInfo(
            identifiers={(DOMAIN, entry.entry_id)},
            name="Klar NLU",
            manufacturer="FABBricate IT Solutions",
        )

    async def async_added_to_hass(self) -> None:
        self.async_on_remove(self._entry.add_update_listener(self._async_entry_updated))

    async def _async_entry_updated(self, hass: HomeAssistant, entry: ConfigEntry) -> None:
        self.async_write_ha_state()

    @property
    def is_on(self) -> bool:
        return bool(self._entry.options.get(CONF_QUIET_ACK, DEFAULT_QUIET_ACK))

    async def async_turn_on(self, **kwargs: object) -> None:
        del kwargs
        await self._async_set(True)

    async def async_turn_off(self, **kwargs: object) -> None:
        del kwargs
        await self._async_set(False)

    async def _async_set(self, enabled: bool) -> None:
        if enabled == self.is_on:
            return
        self.hass.config_entries.async_update_entry(
            self._entry,
            options={**self._entry.options, CONF_QUIET_ACK: enabled},
        )
        self.async_write_ha_state()
