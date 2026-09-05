from __future__ import annotations

import logging

from homeassistant.config_entries import ConfigEntry
from homeassistant.const import EVENT_HOMEASSISTANT_STOP, Platform
from homeassistant.core import HomeAssistant
from homeassistant.exceptions import ConfigEntryNotReady

from .const import (
    CONF_ALLOW_LLM_TOOLS,
    CONF_ASSIST_FILTER,
    CONF_CALENDAR_LLM,
    CONF_CHANNEL,
    CONF_FALLBACK_AGENT,
    CONF_LANGUAGES,
    CONF_MODE,
    CONF_NLU_RAG,
    CONF_PERSONALITY,
    CONF_QUIET_ACK,
    CONF_REFINE_SPEECH,
    CONF_TOKEN,
    CONF_URL,
    DEFAULT_ALLOW_LLM_TOOLS,
    DEFAULT_CALENDAR_LLM,
    DEFAULT_NLU_RAG,
    DEFAULT_QUIET_ACK,
    DEFAULT_REFINE_SPEECH,
    DEFAULT_URL,
    DOMAIN,
    MODE_LOCAL,
    resolve_channel,
    resolve_personality,
)
from .engine import KlarEngine, async_push_personality
from .lang_select import engine_language_state
from .panel import async_setup_panel
from .quiet import async_setup_chime
from .services import async_setup_services
from .sync import HomeGraphSync, engine_url

PLATFORMS = [Platform.CONVERSATION, Platform.SELECT, Platform.SENSOR, Platform.SWITCH]
_LOGGER = logging.getLogger(__name__)


async def async_setup_entry(hass: HomeAssistant, entry: ConfigEntry) -> bool:
    hass.data.setdefault(DOMAIN, {})
    engine: KlarEngine | None = None
    if entry.options.get(CONF_MODE, entry.data.get(CONF_MODE)) == MODE_LOCAL:
        engine = KlarEngine(
            hass,
            channel=resolve_channel(
                entry.options.get(CONF_CHANNEL, entry.data.get(CONF_CHANNEL))
            ),
        )
        try:
            await engine.async_start()
        except Exception as err:
            raise ConfigEntryNotReady(str(err)) from err
        async def _stop_on_shutdown(_event) -> None:
            await engine.async_stop()

        entry.async_on_unload(
            hass.bus.async_listen_once(EVENT_HOMEASSISTANT_STOP, _stop_on_shutdown)
        )
    token = (engine.token if engine is not None else None) or entry.options.get(
        CONF_TOKEN
    ) or entry.data.get(CONF_TOKEN)
    url = engine_url(entry)
    sync = HomeGraphSync(hass, entry, url, token)
    hass.data[DOMAIN][entry.entry_id] = {
        "engine": engine,
        "token": token,
        "sync": sync,
        "url": url,
        "applied_options": dict(entry.options),
    }
    await hass.config_entries.async_forward_entry_setups(entry, PLATFORMS)
    await sync.async_start()
    await async_setup_services(hass)
    await async_setup_chime(hass)
    try:
        await async_setup_panel(hass)
    except Exception:
        _LOGGER.exception("Klar sidebar panel failed; engine still loads")
    await _async_sync_personality(hass, entry)
    entry.async_on_unload(entry.add_update_listener(_async_on_update))
    return True


def _option(entry: ConfigEntry, key: str) -> object:
    return entry.options.get(key, entry.data.get(key))


def _pipeline_flags(entry: ConfigEntry) -> dict[str, object]:
    return {
        "nlu_rag": bool(entry.options.get(CONF_NLU_RAG, DEFAULT_NLU_RAG)),
        "refine_speech": bool(entry.options.get(CONF_REFINE_SPEECH, DEFAULT_REFINE_SPEECH)),
        "calendar_llm": bool(entry.options.get(CONF_CALENDAR_LLM, DEFAULT_CALENDAR_LLM)),
        "quiet_ack": bool(entry.options.get(CONF_QUIET_ACK, DEFAULT_QUIET_ACK)),
        "allow_llm_tools": bool(
            entry.options.get(CONF_ALLOW_LLM_TOOLS, DEFAULT_ALLOW_LLM_TOOLS)
        ),
        "fallback_llm": bool(entry.options.get(CONF_FALLBACK_AGENT)),
    }


async def _async_sync_personality(hass: HomeAssistant, entry: ConfigEntry) -> None:
    stored = (hass.data.get(DOMAIN) or {}).get(entry.entry_id) or {}
    token = stored.get("token") or _option(entry, CONF_TOKEN)
    url = _option(entry, CONF_URL) or DEFAULT_URL
    languages, _chrome = engine_language_state(
        entry.options.get(CONF_LANGUAGES),
        getattr(hass.config, "language", None),
    )
    await async_push_personality(
        hass,
        str(url),
        resolve_personality(entry.options.get(CONF_PERSONALITY)),
        token=str(token) if token else None,
        languages=languages,
        pipeline=_pipeline_flags(entry),
    )


async def _async_on_update(hass: HomeAssistant, entry: ConfigEntry) -> None:
    stored = (hass.data.get(DOMAIN) or {}).get(entry.entry_id)
    previous = dict(stored.get("applied_options") or {}) if stored else {}
    current = dict(entry.options)
    if stored is not None:
        stored["applied_options"] = current
    await _async_sync_personality(hass, entry)
    reload_keys = (
        CONF_URL,
        CONF_TOKEN,
        CONF_LANGUAGES,
        CONF_ASSIST_FILTER,
        CONF_CHANNEL,
        CONF_MODE,
    )
    if any(previous.get(key) != current.get(key) for key in reload_keys):
        await hass.config_entries.async_reload(entry.entry_id)


async def async_unload_entry(hass: HomeAssistant, entry: ConfigEntry) -> bool:
    unload_ok = await hass.config_entries.async_unload_platforms(entry, PLATFORMS)
    stored = hass.data[DOMAIN].pop(entry.entry_id, None) or {}
    sync = stored.get("sync")
    if sync is not None:
        await sync.async_stop()
    engine = stored.get("engine")
    if engine is not None:
        await engine.async_stop()
    return unload_ok
