"""Fetch headlines and stitch the news-briefing reply."""

from __future__ import annotations

import logging
import xml.etree.ElementTree as ET
from typing import Any

try:
    import aiohttp
    from homeassistant.core import HomeAssistant
    from homeassistant.helpers.aiohttp_client import async_get_clientsession
except ImportError:  # stdlib tests load this module without Home Assistant
    aiohttp = None  # type: ignore[assignment]
    HomeAssistant = Any

    async def async_get_clientsession(hass: Any) -> Any:
        raise RuntimeError("homeassistant missing")

_LOGGER = logging.getLogger(__name__)

_FEEDS = {
    "de": "https://www.tagesschau.de/infoservices/alle-meldungen-100~rss2.xml",
    "en": "https://feeds.bbci.co.uk/news/world/rss.xml",
}
_UA = "KlarNLU/2026.8.9 (Home Assistant news briefing)"

_NUDGE = {
    "de": "Möchtest du zu einer der Meldungen mehr erfahren?",
    "en": "Would you like to hear more about any of those stories?",
}

_ASKED = (
    "möchtest du",
    "möchten sie",
    "moechten sie",
    "willst du",
    "wollen sie",
    "mehr erfahren",
    "mehr über",
    "mehr ueber",
    "mehr hören",
    "soll ich",
    "would you like",
    "want to hear",
    "hear more",
    "learn more",
    "shall i",
    "more detail",
    "any of those",
    "einer der",
)


def nudge(pack: str) -> str:
    return _NUDGE.get(pack, _NUDGE["de"])


def asked_for_more(text: str) -> bool:
    lowered = (text or "").casefold()
    return any(hint in lowered for hint in _ASKED)


def compose_speech(intro: str, llm: str, extra: str, announced: bool) -> str:
    parts: list[str] = []
    if intro and not announced:
        parts.append(intro.strip())
    if llm:
        parts.append(llm.strip())
    if extra:
        parts.append(extra.strip())
    return " ".join(part for part in parts if part)


def headlines_from_xml(raw: str, limit: int = 5) -> list[str]:
    if not raw.strip():
        return []
    try:
        root = ET.fromstring(raw)
    except ET.ParseError:
        return []
    titles: list[str] = []
    for node in root.iter():
        tag = node.tag.split("}")[-1]
        if tag not in {"item", "entry"}:
            continue
        title = ""
        for child in node:
            if child.tag.split("}")[-1] == "title" and child.text:
                title = child.text.strip()
                break
        if title and title not in titles:
            titles.append(title)
        if len(titles) >= limit:
            break
    return titles


async def fetch_headlines(hass: HomeAssistant, pack: str, limit: int = 5) -> list[str]:
    if aiohttp is None:
        return []
    url = _FEEDS.get(pack, _FEEDS["de"])
    try:
        session = async_get_clientsession(hass)
        async with session.get(
            url,
            timeout=aiohttp.ClientTimeout(total=4),
            headers={"User-Agent": _UA},
        ) as resp:
            resp.raise_for_status()
            raw = await resp.text()
    except Exception as err:  # noqa: BLE001 — outbound news feed is a boundary
        _LOGGER.warning("Nachrichten-Feed nicht erreichbar: %s", err)
        return []
    return headlines_from_xml(raw, limit)


async def announce(hass: HomeAssistant, user_input: Any, text: str) -> bool:
    sat = getattr(user_input, "satellite_id", None)
    if not sat or "." not in str(sat) or not text:
        return False
    try:
        await hass.services.async_call(
            "assist_satellite",
            "announce",
            {"message": text},
            target={"entity_id": str(sat)},
            blocking=False,
        )
        return True
    except Exception as err:  # noqa: BLE001 — satellite announce is optional
        _LOGGER.debug("Intro nicht angesagt: %s", err)
        return False
