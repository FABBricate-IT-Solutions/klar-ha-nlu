"""Claim the App's auto-minted write token (package 6.6)."""

from __future__ import annotations

import logging

from aiohttp import ClientError, ClientTimeout
from homeassistant.core import HomeAssistant
from homeassistant.helpers.aiohttp_client import async_get_clientsession

from .const import engine_url_candidates

_LOGGER = logging.getLogger(__name__)


async def async_claim_setup_token(hass: HomeAssistant, url: str) -> str | None:
    """Claim the App's auto-minted write token. Supervisor/loopback only on the engine."""
    session = async_get_clientsession(hass)
    for base in engine_url_candidates(url):
        try:
            async with session.get(
                f"{base}/api/v2/setup/token",
                timeout=ClientTimeout(total=3),
            ) as resp:
                if resp.status >= 400:
                    continue
                payload = await resp.json()
        except (ClientError, TimeoutError, OSError, ValueError) as err:
            _LOGGER.debug("Klar setup token not claimed from %s: %s", base, err)
            continue
        if not isinstance(payload, dict):
            continue
        token = str(payload.get("token") or "").strip()
        if token:
            return token
    return None
