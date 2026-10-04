"""Fetch Klar ASR boost prompt for wyoming-faster-whisper --initial-prompt."""

from __future__ import annotations

import logging
from pathlib import Path
from typing import Any

from aiohttp import ClientError, ClientTimeout
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import HomeAssistant
from homeassistant.helpers.aiohttp_client import async_get_clientsession

from .const import CONF_TOKEN, CONF_URL, DEFAULT_URL, DOMAIN, engine_headers, engine_url_candidates

_LOGGER = logging.getLogger(__name__)

DEFAULT_RELATIVE = "klar_nlu_asr_boost.txt"
_TIMEOUT = ClientTimeout(total=8)


def boost_path(hass: HomeAssistant, relative: str | None = None) -> Path:
    name = (relative or DEFAULT_RELATIVE).strip() or DEFAULT_RELATIVE
    path = Path(name)
    if path.is_absolute():
        return path
    return Path(hass.config.config_dir) / path


async def fetch_asr_boost(
    hass: HomeAssistant,
    entry: ConfigEntry,
    *,
    language: str | None = None,
    max_tokens: int | None = None,
) -> dict[str, Any] | None:
    stored = (hass.data.get(DOMAIN) or {}).get(entry.entry_id) or {}
    url = str(entry.options.get(CONF_URL) or entry.data.get(CONF_URL) or stored.get("url") or DEFAULT_URL).rstrip("/")
    token = stored.get("token") or entry.options.get(CONF_TOKEN) or entry.data.get(CONF_TOKEN)
    session = async_get_clientsession(hass)
    headers = engine_headers(str(token) if token else None, extra={"Accept": "application/json"})
    params: dict[str, str] = {}
    if language:
        params["language"] = language
    if max_tokens is not None:
        params["max_tokens"] = str(max_tokens)
    last_err: Exception | None = None
    for host in engine_url_candidates(url):
        try:
            async with session.get(
                f"{host}/api/v2/speech/asr_boost",
                params=params or None,
                headers=headers,
                timeout=_TIMEOUT,
            ) as resp:
                if resp.status == 404:
                    _LOGGER.debug("Klar asr_boost route missing on %s", host)
                    return None
                resp.raise_for_status()
                payload = await resp.json()
        except (ClientError, TimeoutError, OSError, ValueError) as err:
            last_err = err
            continue
        if isinstance(payload, dict) and isinstance(payload.get("prompt"), str):
            return payload
        return None
    if last_err is not None:
        _LOGGER.debug("Klar asr_boost fetch failed: %s", last_err)
    return None


async def export_asr_boost(
    hass: HomeAssistant,
    entry: ConfigEntry,
    *,
    language: str | None = None,
    max_tokens: int | None = None,
    path: str | None = None,
) -> dict[str, Any] | None:
    payload = await fetch_asr_boost(hass, entry, language=language, max_tokens=max_tokens)
    if payload is None:
        return None
    target = boost_path(hass, path)
    prompt = str(payload.get("prompt") or "")
    try:
        await hass.async_add_executor_job(_write_prompt, target, prompt)
    except OSError as err:
        _LOGGER.warning("Klar asr_boost write failed (%s): %s", target, err)
        return None
    payload = {**payload, "path": str(target)}
    stored = hass.data.setdefault(DOMAIN, {}).setdefault(entry.entry_id, {})
    stored["asr_boost_path"] = str(target)
    stored["asr_boost_updated_at"] = payload.get("updated_at")
    _LOGGER.info("Klar ASR boost written to %s (%s terms)", target, len(payload.get("terms") or []))
    return payload


def _write_prompt(path: Path, prompt: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(prompt.strip() + ("\n" if prompt.strip() else ""), encoding="utf-8")
