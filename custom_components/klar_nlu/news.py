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
    "de-CH": "https://www.tagesschau.de/infoservices/alle-meldungen-100~rss2.xml",
    "de-AT": "https://www.tagesschau.de/infoservices/alle-meldungen-100~rss2.xml",
    "en": "https://feeds.bbci.co.uk/news/world/rss.xml",
    "en-GB": "https://feeds.bbci.co.uk/news/world/rss.xml",
    "fr": "https://feeds.bbci.co.uk/afrique/rss.xml",
    "es": "https://feeds.bbci.co.uk/mundo/rss.xml",
    "ca": "https://feeds.bbci.co.uk/mundo/rss.xml",
    "gl": "https://feeds.bbci.co.uk/mundo/rss.xml",
    "pt": "https://feeds.bbci.co.uk/portuguese/rss.xml",
    "pt-BR": "https://feeds.bbci.co.uk/portuguese/rss.xml",
    "ar": "https://feeds.bbci.co.uk/arabic/rss.xml",
    "fa": "https://feeds.bbci.co.uk/arabic/rss.xml",
    "ur": "https://feeds.bbci.co.uk/urdu/rss.xml",
    "hi": "https://feeds.bbci.co.uk/hindi/rss.xml",
    "bn": "https://feeds.bbci.co.uk/bengali/rss.xml",
    "mr": "https://feeds.bbci.co.uk/hindi/rss.xml",
    "ne": "https://feeds.bbci.co.uk/hindi/rss.xml",
    "gu": "https://feeds.bbci.co.uk/hindi/rss.xml",
    "pa": "https://feeds.bbci.co.uk/hindi/rss.xml",
    "ta": "https://feeds.bbci.co.uk/tamil/rss.xml",
    "te": "https://feeds.bbci.co.uk/telugu/rss.xml",
    "ja": "https://www.nhk.or.jp/rss/news/cat0.xml",
    "zh-CN": "https://feeds.bbci.co.uk/zhongwen/simp/rss.xml",
    "zh-TW": "https://feeds.bbci.co.uk/zhongwen/trad/rss.xml",
    "zh-HK": "https://feeds.bbci.co.uk/zhongwen/trad/rss.xml",
    "ko": "https://feeds.bbci.co.uk/news/world/rss.xml",
    "th": "https://feeds.bbci.co.uk/thai/rss.xml",
}
_DEFAULT_FEED = _FEEDS["en"]
_UA = "KlarNLU/2026.8.9 (Home Assistant news briefing)"

_NUDGE = {
    "de": "Möchtest du zu einer der Meldungen mehr erfahren?",
    "de-CH": "Wotsch zu einere vo de Meldige meh ghöre?",
    "de-AT": "Möchtest du zu einer der Meldungen mehr erfahren?",
    "en": "Would you like to hear more about any of those stories?",
    "en-GB": "Would you like to hear more about any of those stories?",
    "fr": "Tu veux en savoir plus sur l'une de ces infos ?",
    "es": "¿Quieres saber más de alguna de estas noticias?",
    "ja": "どれかについてもっと聞きたいですか？",
    "zh-CN": "想了解其中哪一条的详情吗？",
    "zh-TW": "想了解其中哪一則的詳情嗎？",
    "zh-HK": "想知多啲邊一則？",
    "ko": "이 중 더 듣고 싶은 소식이 있나요?",
    "hi": "किसी खबर के बारे में और सुनना है?",
    "ar": "هل تريد معرفة المزيد عن أحد هذه الأخبار؟",
    "th": "อยากฟังรายละเอียดข่าวไหนเพิ่มไหม",
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
    "もっと",
    "詳しく",
    "了解",
    "详情",
    "詳情",
    "더 듣",
    "더 알",
    "और सुन",
    "और जान",
    "المزيد",
    "เพิ่ม",
    "en savoir",
    "saber más",
    "saber mas",
)


def feed_url(pack: str) -> str:
    if pack in _FEEDS:
        return _FEEDS[pack]
    base = pack.split("-", 1)[0]
    return _FEEDS.get(base, _DEFAULT_FEED)


def nudge(pack: str) -> str:
    if pack in _NUDGE:
        return _NUDGE[pack]
    base = pack.split("-", 1)[0]
    return _NUDGE.get(base, _NUDGE["en"])


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
    url = feed_url(pack)
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
