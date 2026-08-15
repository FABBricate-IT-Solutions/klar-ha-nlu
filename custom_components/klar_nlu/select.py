from __future__ import annotations

from homeassistant.components.select import SelectEntity
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers.device_registry import DeviceInfo
from homeassistant.helpers.entity import EntityCategory
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .const import (
    CONF_PERSONALITY,
    DEFAULT_PERSONALITY,
    DOMAIN,
    PERSONALITIES,
)


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    async_add_entities([KlarPersonalitySelect(entry)])


class KlarPersonalitySelect(SelectEntity):
    _attr_has_entity_name = True
    _attr_translation_key = "personality"
    _attr_entity_category = EntityCategory.CONFIG
    _attr_options = list(PERSONALITIES)

    def __init__(self, entry: ConfigEntry) -> None:
        self._entry = entry
        self._attr_unique_id = f"{entry.entry_id}_personality"
        self._attr_device_info = DeviceInfo(
            identifiers={(DOMAIN, entry.entry_id)},
            name="Klar NLU",
            manufacturer="FABBricate IT Solutions",
        )

    @property
    def current_option(self) -> str:
        value = str(self._entry.options.get(CONF_PERSONALITY, DEFAULT_PERSONALITY))
        return value if value in PERSONALITIES else DEFAULT_PERSONALITY

    async def async_select_option(self, option: str) -> None:
        if option not in PERSONALITIES:
            return
        self.hass.config_entries.async_update_entry(
            self._entry, options={**self._entry.options, CONF_PERSONALITY: option}
        )
