"""Look up a factual question before the LLM may speak.

Wikipedia is the built-in source. The query is the question itself. The model
may only repeat what that text contains.
"""

from __future__ import annotations

import json
import logging
import re
from typing import Any
from urllib.parse import urlencode

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
_UA = "KlarNLU/2026.10.2 (Home Assistant factual lookup)"
_EXTRACT = 1200
_SPEECH = 700
_GLUE = frozenset(
    """
    der die das den dem des ein eine einer einem einen und oder ist sind war hat haben
    wie wer was welche welcher welches wo wann warum wieso von zu zum zur für fuer mit
    the a an is are was of who what which where when why to for in on
    """.split()
)
_MISS = {
    "de": "Dazu habe ich keine aktuelle Quelle.",
    "en": "I don't have a current source for that.",
}
_INSTRUCTION = {
    "de": (
        "Beantworte nur aus dem folgenden Text. Erfinde nichts und nutze kein eigenes Wissen. "
        "Wenn der Text die Antwort nicht enthält, sage genau: Das steht in der Quelle nicht. "
        "Wiederhole die Frage nicht."
    ),
    "en": (
        "Answer only from the following text. Do not invent anything and do not use your own knowledge. "
        "If the text does not contain the answer, say exactly: The source does not say. "
        "Do not repeat the question."
    ),
}


def lookup_query(text: str) -> str:
    query = " ".join((text or "").split())
    return query[:180]


def wiki_lang(pack: str) -> str:
    tag = (pack or "en").split("-", 1)[0].casefold()
    if re.fullmatch(r"[a-z]{2,3}", tag):
        return tag
    return "en"


def search_url(pack: str, query: str) -> str:
    params = urlencode(
        {
            "action": "query",
            "list": "search",
            "srsearch": query,
            "srlimit": "5",
            "format": "json",
            "utf8": "1",
        }
    )
    return f"https://{wiki_lang(pack)}.wikipedia.org/w/api.php?{params}"


def extract_url(pack: str, title: str) -> str:
    params = urlencode(
        {
            "action": "query",
            "prop": "extracts",
            "exintro": "1",
            "explaintext": "1",
            "redirects": "1",
            "titles": title,
            "format": "json",
            "utf8": "1",
        }
    )
    return f"https://{wiki_lang(pack)}.wikipedia.org/w/api.php?{params}"


def search_title(payload: object) -> str:
    if not isinstance(payload, dict):
        return ""
    rows = (payload.get("query") or {}).get("search") if isinstance(payload.get("query"), dict) else None
    if not isinstance(rows, list):
        return ""
    for row in rows[:5]:
        if not isinstance(row, dict):
            continue
        title = str(row.get("title") or "").strip()
        folded = title.casefold()
        if not title or folded.startswith(("liste ", "list of ", "begriffsklärung", "disambiguation")):
            continue
        return title
    return ""


def intro_text(payload: object) -> str:
    if not isinstance(payload, dict):
        return ""
    query = payload.get("query")
    pages = query.get("pages") if isinstance(query, dict) else None
    if not isinstance(pages, dict):
        return ""
    for page in pages.values():
        if not isinstance(page, dict) or "missing" in page:
            continue
        extract = " ".join(str(page.get("extract") or "").split())
        if extract:
            return extract[:_EXTRACT]
    return ""


def lookup_miss(pack: str) -> str:
    if pack in _MISS:
        return _MISS[pack]
    base = pack.split("-", 1)[0]
    return _MISS.get(base, _MISS["en"])


def lookup_instruction(pack: str, facts: list[str]) -> str:
    body = _INSTRUCTION["de"] if pack == "de" or pack.startswith("de-") else _INSTRUCTION["en"]
    source = "\n".join(fact for fact in facts if fact.strip())
    return f"{body}\n\nQuelle:\n{source}" if pack == "de" or pack.startswith("de-") else f"{body}\n\nSource:\n{source}"


def source_speech(facts: list[str]) -> str:
    text = " ".join(fact.strip() for fact in facts if fact.strip())
    if len(text) <= _SPEECH:
        return text
    cut = text[:_SPEECH]
    dot = max(cut.rfind(". "), cut.rfind("。"))
    if dot >= 80:
        return cut[: dot + 1]
    return cut.rsplit(" ", 1)[0].rstrip(" ,;") + "."


def echoes_question(asked: str, reply: str) -> bool:
    left = _fold(asked)
    right = _fold(reply)
    return bool(left) and left == right


def grounded(asked: str, reply: str, source: str) -> bool:
    if not reply.strip() or echoes_question(asked, reply):
        return False
    known = set(_words(source))
    extra = [word for word in _words(reply) if word not in _GLUE and word not in set(_words(asked))]
    return bool(extra) and all(word in known for word in extra)


async def lookup_facts(hass: HomeAssistant, pack: str, text: str, session: Any = None) -> list[str]:
    query = lookup_query(text)
    if not query:
        return []
    raw_search = await _get(hass, search_url(pack, query), session)
    title = search_title(_json(raw_search))
    if not title:
        return []
    raw_page = await _get(hass, extract_url(pack, title), session)
    intro = intro_text(_json(raw_page))
    if not intro:
        return []
    return [f"{title}. {intro}"]


def _json(raw: str) -> object:
    if not raw.strip():
        return {}
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        return {}


def _words(text: str) -> list[str]:
    return re.findall(r"[0-9]+|[^\W\d_]{2,}", text.casefold(), flags=re.UNICODE)


def _fold(text: str) -> str:
    return "".join(ch for ch in text.casefold() if ch.isalnum())


async def _get(hass: HomeAssistant, url: str, session: Any) -> str:
    if aiohttp is None and session is None:
        return ""
    try:
        client = session or async_get_clientsession(hass)
        timeout = aiohttp.ClientTimeout(total=4) if aiohttp is not None else None
        async with client.get(url, timeout=timeout, headers={"User-Agent": _UA}) as resp:
            resp.raise_for_status()
            return await resp.text()
    except Exception as err:  # noqa: BLE001 — outbound reference source is a boundary
        _LOGGER.warning("Quelle nicht erreichbar: %s", err)
        return ""
