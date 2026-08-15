from __future__ import annotations

from homeassistant.config_entries import ConfigEntry
from homeassistant.const import EVENT_HOMEASSISTANT_STOP, Platform
from homeassistant.core import HomeAssistant
from homeassistant.exceptions import ConfigEntryNotReady

from .const import (
    CONF_MODE,
    CONF_PERSONALITY,
    CONF_URL,
    DEFAULT_PERSONALITY,
    DEFAULT_URL,
    DOMAIN,
    MODE_LOCAL,
)
from .engine import KlarEngine, async_push_personality

PLATFORMS = [Platform.CONVERSATION, Platform.SELECT]


async def async_setup_entry(hass: HomeAssistant, entry: ConfigEntry) -> bool:
    hass.data.setdefault(DOMAIN, {})
    engine: KlarEngine | None = None
    if entry.data.get(CONF_MODE) == MODE_LOCAL:
        engine = KlarEngine(hass)
        try:
            await engine.async_start()
        except Exception as err:
            raise ConfigEntryNotReady(str(err)) from err
        async def _stop_on_shutdown(_event) -> None:
            await engine.async_stop()

        entry.async_on_unload(
            hass.bus.async_listen_once(EVENT_HOMEASSISTANT_STOP, _stop_on_shutdown)
        )
    hass.data[DOMAIN][entry.entry_id] = {"engine": engine}
    await hass.config_entries.async_forward_entry_setups(entry, PLATFORMS)
    url = (
        entry.options.get(CONF_URL)
        or entry.data.get(CONF_URL)
        or DEFAULT_URL
    )
    await async_push_personality(
        hass,
        url,
        str(entry.options.get(CONF_PERSONALITY, DEFAULT_PERSONALITY)),
    )
    entry.async_on_unload(entry.add_update_listener(_async_reload))
    return True


async def _async_reload(hass: HomeAssistant, entry: ConfigEntry) -> None:
    await hass.config_entries.async_reload(entry.entry_id)


async def async_unload_entry(hass: HomeAssistant, entry: ConfigEntry) -> bool:
    unload_ok = await hass.config_entries.async_unload_platforms(entry, PLATFORMS)
    stored = hass.data[DOMAIN].pop(entry.entry_id, None) or {}
    engine = stored.get("engine")
    if engine is not None:
        await engine.async_stop()
    return unload_ok
