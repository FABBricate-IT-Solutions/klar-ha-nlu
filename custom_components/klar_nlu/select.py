from __future__ import annotations

from homeassistant.components.select import SelectEntity
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers.device_registry import DeviceInfo
from homeassistant.helpers.entity import EntityCategory
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .const import (
    CONF_PERSONALITY,
    CONF_REFINE_PROMPT,
    DOMAIN,
    PERSONALITIES,
    resolve_personality,
)
from .refine_voices import editable_prompt, prompt_pack


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

    async def async_added_to_hass(self) -> None:
        self.async_on_remove(self._entry.add_update_listener(self._async_entry_updated))

    async def _async_entry_updated(self, hass: HomeAssistant, entry: ConfigEntry) -> None:
        self.async_write_ha_state()

    @property
    def current_option(self) -> str:
        return resolve_personality(self._entry.options.get(CONF_PERSONALITY))

    async def async_select_option(self, option: str) -> None:
        personality = resolve_personality(option)
        if personality not in PERSONALITIES or personality == self.current_option:
            return
        pack = prompt_pack(getattr(getattr(self.hass, "config", None), "language", None))
        self.hass.config_entries.async_update_entry(
            self._entry,
            options={
                **self._entry.options,
                CONF_PERSONALITY: personality,
                CONF_REFINE_PROMPT: editable_prompt(personality, pack),
            },
        )
        self.async_write_ha_state()
