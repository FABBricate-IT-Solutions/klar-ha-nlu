from __future__ import annotations

import logging

import aiohttp
import voluptuous as vol
from homeassistant.components import conversation
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant, ServiceCall
from homeassistant.helpers import config_validation as cv
from homeassistant.helpers import entity_registry as er
from homeassistant.helpers.aiohttp_client import async_get_clientsession
from homeassistant.helpers.service import async_register_admin_service

from .asr_boost import export_asr_boost
from .const import (
    CONF_TOKEN,
    CONF_URL,
    DEFAULT_URL,
    DOMAIN,
    LAST_EXECUTE_SESSION,
    LAST_SESSION,
    SERVICE_TURNS,
    engine_headers,
    engine_url_candidates,
)

_LOGGER = logging.getLogger(__name__)

SERVICE_UNDO = "undo"
SERVICE_TEACH = "teach_alias"
SERVICE_EXPORT_ASR_BOOST = "export_asr_boost"

_UNDO_SCHEMA = vol.Schema({})
_TEACH_SCHEMA = vol.Schema(
    {
        vol.Required("alias"): vol.All(cv.string, vol.Length(min=2, max=40)),
        vol.Optional("entity_id"): cv.entity_id,
    }
)
_EXPORT_SCHEMA = vol.Schema(
    {
        vol.Optional("language"): cv.string,
        vol.Optional("max_tokens"): vol.All(vol.Coerce(int), vol.Range(min=1, max=223)),
        vol.Optional("path"): cv.string,
    }
)


def _entry(hass: HomeAssistant) -> ConfigEntry | None:
    entries = hass.config_entries.async_entries(DOMAIN)
    return entries[0] if entries else None


def _agent_id(hass: HomeAssistant, entry: ConfigEntry) -> str | None:
    registry = er.async_get(hass)
    for item in er.async_entries_for_config_entry(registry, entry.entry_id):
        if item.domain == "conversation":
            return item.entity_id
    return None


async def _say_in_session(
    hass: HomeAssistant, entry: ConfigEntry, key: str, text: str, call: ServiceCall, language: str
) -> None:
    """Send a service's sentence into the conversation it is about.

    Klar has no session shared by everyone, so "undo" needs the conversation that
    acted last. The agent picks that session up by the context of this call, because
    Home Assistant would replace an idle conversation id with a new one.
    """
    stored = (hass.data.get(DOMAIN) or {}).get(entry.entry_id) or {}
    session = stored.get(key)
    turns = stored.setdefault(SERVICE_TURNS, {}) if isinstance(stored, dict) else {}
    if isinstance(session, str) and session:
        turns[call.context.id] = session
    try:
        await conversation.async_converse(
            hass, text, None, call.context, language=language, agent_id=_agent_id(hass, entry)
        )
    finally:
        turns.pop(call.context.id, None)


def _url_token(hass: HomeAssistant, entry: ConfigEntry) -> tuple[str, str | None]:
    stored = (hass.data.get(DOMAIN) or {}).get(entry.entry_id) or {}
    url = str(entry.options.get(CONF_URL) or entry.data.get(CONF_URL) or DEFAULT_URL).rstrip("/")
    token = stored.get("token") or entry.options.get(CONF_TOKEN) or entry.data.get(CONF_TOKEN)
    return url, str(token) if token else None


async def async_setup_services(hass: HomeAssistant) -> None:
    if hass.data.get(DOMAIN, {}).get("services"):
        return
    hass.data.setdefault(DOMAIN, {})["services"] = True

    async def undo(call: ServiceCall) -> None:
        entry = _entry(hass)
        if entry is None:
            return
        language = str(hass.config.language or "en")
        text = "rückgängig" if language.startswith("de") else "undo that"
        await _say_in_session(hass, entry, LAST_EXECUTE_SESSION, text, call, language)

    async def teach(call: ServiceCall) -> None:
        entry = _entry(hass)
        if entry is None:
            return
        alias = str(call.data.get("alias") or "").strip()
        entity_id = str(call.data.get("entity_id") or "").strip()
        if len(alias) < 2 or len(alias) > 40 or any(ch.isascii() and ord(ch) < 32 for ch in alias):
            return
        if entity_id.count(".") != 1:
            language = str(hass.config.language or "en")
            text = f"nenn das {alias}" if language.startswith("de") else f"call this {alias}"
            await _say_in_session(hass, entry, LAST_SESSION, text, call, language)
            return
        url, token = _url_token(hass, entry)
        headers = engine_headers(token)
        session = async_get_clientsession(hass)
        last_err: Exception | None = None
        for base in engine_url_candidates(url):
            try:
                async with session.post(
                    f"{base}/api/entities",
                    json={"entity_id": entity_id, "aliases": [alias]},
                    headers=headers,
                    timeout=aiohttp.ClientTimeout(total=5),
                ) as resp:
                    resp.raise_for_status()
                return
            except (aiohttp.ClientError, TimeoutError, OSError) as err:
                last_err = err
        if last_err is not None:
            _LOGGER.warning("Klar teach_alias failed: %s", last_err)

    async def export_boost(call: ServiceCall) -> None:
        entry = _entry(hass)
        if entry is None:
            return
        language = str(call.data.get("language") or "").strip() or None
        path = str(call.data.get("path") or "").strip() or None
        raw_tokens = call.data.get("max_tokens")
        max_tokens = int(raw_tokens) if isinstance(raw_tokens, (int, float, str)) and str(raw_tokens).isdigit() else None
        result = await export_asr_boost(
            hass,
            entry,
            language=language,
            max_tokens=max_tokens,
            path=path,
        )
        if result is None:
            _LOGGER.warning("Klar export_asr_boost failed (engine unreachable or route missing)")

    # A-05 / H-21: admin-only services with schemas; removed again on unload.
    async_register_admin_service(hass, DOMAIN, SERVICE_UNDO, undo, schema=_UNDO_SCHEMA)
    async_register_admin_service(hass, DOMAIN, SERVICE_TEACH, teach, schema=_TEACH_SCHEMA)
    async_register_admin_service(
        hass, DOMAIN, SERVICE_EXPORT_ASR_BOOST, export_boost, schema=_EXPORT_SCHEMA
    )


async def async_unload_services(hass: HomeAssistant) -> None:
    if not hass.data.get(DOMAIN, {}).get("services"):
        return
    for service in (SERVICE_UNDO, SERVICE_TEACH, SERVICE_EXPORT_ASR_BOOST):
        if hass.services.has_service(DOMAIN, service):
            hass.services.async_remove(DOMAIN, service)
    hass.data.get(DOMAIN, {}).pop("services", None)
