from __future__ import annotations

import logging

import aiohttp
from homeassistant.components import conversation
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant, ServiceCall
from homeassistant.helpers import entity_registry as er
from homeassistant.helpers.aiohttp_client import async_get_clientsession

from .const import CONF_TOKEN, CONF_URL, DEFAULT_URL, DOMAIN

_LOGGER = logging.getLogger(__name__)

SERVICE_UNDO = "undo"
SERVICE_TEACH = "teach_alias"


def _entry(hass: HomeAssistant) -> ConfigEntry | None:
    entries = hass.config_entries.async_entries(DOMAIN)
    return entries[0] if entries else None


def _agent_id(hass: HomeAssistant, entry: ConfigEntry) -> str | None:
    registry = er.async_get(hass)
    for item in er.async_entries_for_config_entry(registry, entry.entry_id):
        if item.domain == "conversation":
            return item.entity_id
    return None


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
        agent_id = _agent_id(hass, entry)
        language = str(hass.config.language or "en")
        text = "rückgängig" if language.startswith("de") else "undo that"
        await conversation.async_converse(
            hass, text, None, call.context, language=language, agent_id=agent_id
        )

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
            await conversation.async_converse(
                hass, text, None, call.context, language=language, agent_id=_agent_id(hass, entry)
            )
            return
        url, token = _url_token(hass, entry)
        headers = {"X-Klar-Token": token} if token else {}
        session = async_get_clientsession(hass)
        try:
            async with session.post(
                f"{url}/api/entities",
                json={"entity_id": entity_id, "aliases": [alias]},
                headers=headers,
                timeout=aiohttp.ClientTimeout(total=5),
            ) as resp:
                resp.raise_for_status()
        except (aiohttp.ClientError, TimeoutError, OSError) as err:
            _LOGGER.warning("Klar teach_alias failed: %s", err)

    hass.services.async_register(DOMAIN, SERVICE_UNDO, undo)
    hass.services.async_register(DOMAIN, SERVICE_TEACH, teach)
