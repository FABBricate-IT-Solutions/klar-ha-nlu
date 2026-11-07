"""Fetch headlines and stitch the news-briefing reply."""

from __future__ import annotations

import html
import json
import logging
import re
import xml.etree.ElementTree as ET
from typing import Any
from urllib.parse import parse_qs, quote, urlencode, urlparse

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
_BROWSER = "Mozilla/5.0"
_GOOGLE_BATCH = "https://news.google.com/_/DotsSplashUi/data/batchexecute"
_GOOGLE_SG = re.compile(r'data-n-a-sg="([^"]+)"')
_GOOGLE_TS = re.compile(r'data-n-a-ts="([^"]+)"')
_GOOGLE_URL = re.compile(r'garturlres\\",\\"(https://[^\\"]+)')
_LIMIT = 5
_READ_ALOUD = frozenset(
    {
        "read",
        "aloud",
        "lese",
        "lies",
        "lesen",
        "lest",
        "les",
        "vorlesen",
        "vorlese",
        "vorliest",
        "vor",
    }
)
_TOPICS = {
    "sport": "sport",
    "sports": "sport",
    "fussball": "sport",
    "football": "sport",
    "soccer": "sport",
    "politik": "politik",
    "politics": "politik",
    "political": "politik",
    "wirtschaft": "wirtschaft",
    "business": "wirtschaft",
    "economy": "wirtschaft",
    "kultur": "kultur",
    "culture": "kultur",
    "wissenschaft": "wissenschaft",
    "science": "wissenschaft",
    "wissen": "wissenschaft",
    "lokal": "lokal",
    "local": "lokal",
}
_SECTION = {
    ("de", "sport"): "https://www.tagesschau.de/sport/index~rss2.xml",
    ("de", "politik"): "https://www.tagesschau.de/inland/index~rss2.xml",
    ("de", "wirtschaft"): "https://www.tagesschau.de/wirtschaft/index~rss2.xml",
    ("de", "kultur"): "https://www.tagesschau.de/kultur/index~rss2.xml",
    ("de", "wissenschaft"): "https://www.tagesschau.de/wissen/index~rss2.xml",
    ("en", "sport"): "https://feeds.bbci.co.uk/sport/rss.xml",
    ("en", "politik"): "https://feeds.bbci.co.uk/news/politics/rss.xml",
    ("en", "wirtschaft"): "https://feeds.bbci.co.uk/news/business/rss.xml",
    ("en", "kultur"): "https://feeds.bbci.co.uk/news/entertainment_and_arts/rss.xml",
    ("en", "wissenschaft"): "https://feeds.bbci.co.uk/news/science_and_environment/rss.xml",
}
_REGIONS = {"de": "DE", "en": "US", "fr": "FR", "es": "ES", "it": "IT", "pt": "PT", "ja": "JP", "nl": "NL"}

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


_STORY_STOP = _READ_ALOUD | frozenset(
    {
        "die",
        "der",
        "das",
        "den",
        "dem",
        "des",
        "und",
        "oder",
        "mit",
        "von",
        "zu",
        "zur",
        "zum",
        "sie",
        "mir",
        "bitte",
        "eine",
        "einer",
        "einem",
        "eines",
        "ja",
        "ueber",
        "über",
        "darueber",
        "darüber",
        "dazu",
        "news",
        "nachricht",
        "nachrichten",
        "meldung",
        "meldungen",
        "artikel",
        "article",
        "diese",
        "dieser",
        "dieses",
        "diesem",
        "diesen",
        "mehr",
        "ich",
        "du",
        "dich",
        "dir",
        "im",
        "in",
        "fuer",
        "für",
        "fall",
        "aehm",
        "aeh",
        "kannst",
        "koennen",
        "moechte",
        "moechten",
        "erfahren",
        "genauer",
        "genaue",
        "vollstaendig",
        "aktuelle",
        "aktuellen",
        "aktueller",
        "aktuelles",
        "digitale",
        "digitalen",
        "digitaler",
        "digitales",
        "mal",
        "noch",
    }
)
_LAST_STORY = frozenset({"letzte", "letzten", "letzter", "letztes", "last"})
_STORY_INDEX = {
    "erste": 0,
    "ersten": 0,
    "erster": 0,
    "erstes": 0,
    "first": 0,
    "zweite": 1,
    "zweiten": 1,
    "zweiter": 1,
    "zweites": 1,
    "second": 1,
    "dritte": 2,
    "dritten": 2,
    "dritter": 2,
    "drittes": 2,
    "third": 2,
}
_TAG = re.compile(r"<[^>]+>")
_LDJSON = re.compile(r"<script\b[^>]*type=[\"']application/ld\+json[\"'][^>]*>(.*?)</script>", re.I | re.S)
_ARTICLE = re.compile(r"<article\b[^>]*>(.*?)</article>", re.I | re.S)
_MAIN = re.compile(r"<main\b[^>]*>(.*?)</main>", re.I | re.S)
_PARAGRAPH = re.compile(r"<p\b[^>]*>(.*?)</p>", re.I | re.S)
_CHROME = ("newsletter", "cookie", "subscribe", "sign in", "anmelden", "datenschutz")


def _words(text: str) -> list[str]:
    folded = (text or "").casefold()
    for src, dst in (("ä", "ae"), ("ö", "oe"), ("ü", "ue"), ("ß", "ss")):
        folded = folded.replace(src, dst)
    return [word for word in folded.split() if word]


def wants_read(text: str) -> bool:
    return any(word in _READ_ALOUD for word in _words(text))


def stories_from_xml(raw: str, limit: int = 5) -> list[dict[str, str]]:
    if not raw.strip():
        return []
    try:
        root = ET.fromstring(raw)
    except ET.ParseError:
        return []
    stories: list[dict[str, str]] = []
    seen: set[str] = set()
    for node in root.iter():
        tag = node.tag.split("}")[-1]
        if tag not in {"item", "entry"}:
            continue
        title = ""
        body = ""
        link = ""
        for child in node:
            name = child.tag.split("}")[-1]
            text = " ".join(part.strip() for part in child.itertext() if part and part.strip())
            if name == "title" and text and not title:
                title = text
            elif name == "link" and not link:
                href = text if text.startswith("http") else str(child.get("href") or "")
                if href.startswith("http"):
                    link = href
            elif name in {"description", "summary", "content", "encoded"} and text:
                plain = _plain(text)
                if len(plain) > len(body):
                    body = plain
        title = title.strip()
        if not title or title in seen:
            continue
        seen.add(title)
        stories.append({"title": title, "text": body[:12000], "link": link})
        if len(stories) >= limit:
            break
    return stories


def pick_story(text: str, stories: list[dict[str, str]], earlier: str = "") -> dict[str, str] | None:
    if not stories:
        return None
    announced = stories[:_LIMIT]
    words = _words(text)
    if any(word in _LAST_STORY for word in words):
        return announced[-1]
    for word, index in _STORY_INDEX.items():
        if word in words and index < len(announced):
            return announced[index]
    matched = _best_story(words, stories)
    if matched is not None:
        return matched
    if wants_read(text):
        return _best_story(_words(earlier), stories)
    return None


def story_query(text: str) -> str:
    words: list[str] = []
    seen: set[str] = set()
    for word in _needles(_words(text)):
        if word in seen:
            continue
        seen.add(word)
        words.append(word)
    return " ".join(words)[:80]


def spoken_story(story: dict[str, str], article: str = "") -> str:
    title = (story.get("title") or "").strip().rstrip(".")
    body = (article or story.get("text") or "").strip()
    if title and body:
        return f"{title}. {body}"
    return title or body


def article_body_from_html(raw: str) -> str:
    if not raw:
        return ""
    structured = _structured_body(raw)
    written = _paragraph_body(raw)
    if len(structured) >= 400 and len(structured) + 200 >= len(written):
        return structured[:12000]
    if len(written) >= 400:
        return written[:12000]
    return (structured or written)[:12000]


def _structured_body(raw: str) -> str:
    best = ""
    for block in _LDJSON.findall(raw):
        try:
            data = json.loads(block)
        except json.JSONDecodeError:
            continue
        body = _article_body(data)
        if len(body) > len(best):
            best = body
    return best


def _paragraph_body(raw: str) -> str:
    region = _ARTICLE.search(raw) or _MAIN.search(raw)
    chunk = region.group(1) if region else raw
    parts: list[str] = []
    for block in _PARAGRAPH.findall(chunk):
        text = _plain(block)
        if len(text) < 80:
            continue
        folded = text.casefold()
        if any(hint in folded for hint in _CHROME) and len(text) < 300:
            continue
        parts.append(text)
        if sum(len(part) for part in parts) >= 12000:
            break
    return " ".join(parts)


def _article_body(data: object) -> str:
    if isinstance(data, list):
        for item in data:
            found = _article_body(item)
            if found:
                return found
        return ""
    if not isinstance(data, dict):
        return ""
    kind = data.get("@type")
    kinds = kind if isinstance(kind, list) else [kind]
    if any(str(item) in {"NewsArticle", "Article", "ReportageNewsArticle"} for item in kinds):
        text = data.get("articleBody")
        if isinstance(text, str) and text.strip():
            return " ".join(text.split())
    for key in ("@graph", "mainEntity"):
        found = _article_body(data.get(key))
        if found:
            return found
    return ""


def _plain(text: str) -> str:
    return " ".join(_TAG.sub(" ", html.unescape(text or "")).split())


def _needles(words: list[str]) -> list[str]:
    return [word for word in words if len(word) >= 3 and word not in _STORY_STOP and word not in _LAST_STORY and word not in _STORY_INDEX]


def _story_parts(story: dict[str, str]) -> set[str]:
    parts: set[str] = set()
    for token in _words(f"{story.get('title', '')} {story.get('text', '')}"):
        parts.add(token)
        if "-" in token:
            parts.update(piece for piece in token.split("-") if piece)
    return parts


def _best_story(words: list[str], stories: list[dict[str, str]]) -> dict[str, str] | None:
    needles = _needles(words)
    if not needles:
        return None
    best: dict[str, str] | None = None
    best_score = 0
    for story in stories:
        parts = _story_parts(story)
        score = sum(1 for word in needles if word in parts)
        if score > best_score:
            best = story
            best_score = score
    if best_score == 0 or (best_score == 1 and len(needles) >= 2):
        return None
    return best


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


def _http_url(url: str) -> str:
    text = url.strip()
    if len(text) > 200 or not text.startswith(("http://", "https://")):
        return ""
    rest = text.split("://", 1)[1]
    host = rest.split("/", 1)[0].split("?", 1)[0].split("#", 1)[0]
    if not host or host.startswith("@") or "@" in host or not _host_ok(host):
        return ""
    return text


def _host_ok(host: str) -> bool:
    if host.startswith("["):
        return "]" in host
    name = host.split(":", 1)[0]
    return name.casefold() == "localhost" or "." in name


def custom_feed(pack: str, feeds: object) -> str:
    if not isinstance(feeds, list):
        return ""
    for item in feeds:
        if not isinstance(item, dict):
            continue
        if str(item.get("pack") or "") != pack:
            continue
        url = _http_url(str(item.get("url") or ""))
        if url:
            return url
    return ""


def bare_topic(topic: str) -> str:
    words = [word for word in (topic or "").split() if word.casefold() not in _READ_ALOUD]
    return " ".join(words)


def topic_key(topic: str) -> str:
    words = [word for word in (topic or "").casefold().split() if word]
    if not words:
        return ""
    if words[0] in _TOPICS:
        return _TOPICS[words[0]]
    return ""


def section_feed(pack: str, topic: str) -> str:
    key = topic_key(topic)
    if not key:
        return ""
    lang = "de" if pack.lower().startswith("de") else "en" if pack.lower().startswith("en") else pack.split("-", 1)[0]
    return _SECTION.get((lang, key), "")


def search_url(pack: str, topic: str) -> str:
    raw = (pack or "en").strip() or "en"
    lang = raw.split("-", 1)[0].lower() or "en"
    tail = raw.split("-", 1)[1] if "-" in raw else ""
    region = tail.upper() if len(tail) == 2 and tail.isalpha() else _REGIONS.get(lang, "US")
    query = " ".join((topic or "").split())[:80]
    return "https://news.google.com/rss/search?" + urlencode({"q": query, "hl": lang, "gl": region, "ceid": f"{region}:{lang}"})


def choose_feed(pack: str, topic: str, feeds: object) -> tuple[str, str]:
    topic = bare_topic(topic)
    if topic.strip():
        section = section_feed(pack, topic)
        if section:
            return "section", section
        return "search", search_url(pack, topic)
    custom = custom_feed(pack, feeds)
    if custom:
        return "custom", custom
    return "builtin", feed_url(pack)


def spoken_headlines(titles: list[str]) -> str:
    parts = [title.strip().rstrip(".") for title in titles if title and title.strip()]
    return ". ".join(parts)


def feedreader_sources(hass: HomeAssistant) -> tuple[list[str], list[str]]:
    titles: list[str] = []
    urls: list[str] = []
    data = getattr(hass, "data", None)
    blob = data.get("feedreader") if isinstance(data, dict) else None
    titles.extend(_titles_of(blob))
    config = getattr(hass, "config_entries", None)
    entries = []
    if config is not None and hasattr(config, "async_entries"):
        try:
            entries = list(config.async_entries("feedreader"))
        except Exception:  # noqa: BLE001 — feedreader is optional
            entries = []
    for entry in entries:
        payload = getattr(entry, "data", None)
        if not isinstance(payload, dict):
            continue
        url = _http_url(str(payload.get("url") or payload.get("feed_url") or ""))
        if url:
            urls.append(url)
    return titles, urls


def _titles_of(blob: object) -> list[str]:
    found: list[str] = []
    if isinstance(blob, dict):
        for key in ("titles", "headlines"):
            values = blob.get(key)
            if isinstance(values, list):
                found.extend(str(item).strip() for item in values)
        for value in blob.values():
            if isinstance(value, dict):
                title = value.get("title")
                if isinstance(title, str):
                    found.append(title.strip())
    elif isinstance(blob, list):
        for item in blob:
            if isinstance(item, str):
                found.append(item.strip())
            elif isinstance(item, dict) and isinstance(item.get("title"), str):
                found.append(item["title"].strip())
    return [item for item in found if item]


def _take(titles: list[str], seen: set[str], items: list[str]) -> None:
    for item in items:
        text = item.strip()
        if not text or text in seen or len(titles) >= _LIMIT:
            continue
        seen.add(text)
        titles.append(text)


async def _fetch_raw(hass: HomeAssistant, url: str, timeout: float = 4, user_agent: str = "") -> str:
    if aiohttp is None or not _http_url(url):
        return ""
    try:
        session = async_get_clientsession(hass)
        async with session.get(
            url,
            timeout=aiohttp.ClientTimeout(total=timeout),
            headers={"User-Agent": user_agent or _UA},
        ) as resp:
            resp.raise_for_status()
            return await resp.text()
    except Exception as err:  # noqa: BLE001 — outbound news feed is a boundary
        _LOGGER.warning("Nachrichten-Feed nicht erreichbar: %s", err)
        return ""


async def _post_form(hass: HomeAssistant, url: str, body: str, timeout: float = 8) -> str:
    if aiohttp is None or not _http_url(url):
        return ""
    try:
        session = async_get_clientsession(hass)
        async with session.post(
            url,
            data=body,
            timeout=aiohttp.ClientTimeout(total=timeout),
            headers={"User-Agent": _BROWSER, "Content-Type": "application/x-www-form-urlencoded;charset=UTF-8"},
        ) as resp:
            resp.raise_for_status()
            return await resp.text()
    except Exception as err:  # noqa: BLE001 — outbound news feed is a boundary
        _LOGGER.warning("Nachrichten-Feed nicht erreichbar: %s", err)
        return ""


async def fetch_url(hass: HomeAssistant, url: str, limit: int = _LIMIT) -> list[str]:
    return headlines_from_xml(await _fetch_raw(hass, url), limit)


def google_article_id(url: str) -> str:
    parsed = urlparse(url or "")
    if parsed.hostname != "news.google.com":
        return ""
    parts = [part for part in parsed.path.split("/") if part]
    if len(parts) >= 2 and parts[-2] in {"articles", "read"}:
        return parts[-1]
    return ""


def google_params_url(url: str, art_id: str) -> str:
    found = parse_qs(urlparse(url).query)
    hl = (found.get("hl") or ["en-US"])[0]
    gl = (found.get("gl") or ["US"])[0]
    ceid = (found.get("ceid") or ["US:en"])[0]
    query = urlencode({"hl": hl, "gl": gl, "ceid": ceid})
    return f"https://news.google.com/rss/articles/{art_id}?{query}"


def google_signature(page: str) -> tuple[str, str] | None:
    signature = _GOOGLE_SG.search(page or "")
    timestamp = _GOOGLE_TS.search(page or "")
    if signature and timestamp:
        return signature.group(1), timestamp.group(1)
    return None


def google_batch_body(art_id: str, timestamp: str, signature: str) -> str:
    context = [
        ["X", "X", ["X", "X"], None, None, 1, 1, "US:en", None, 1, None, None, None, None, None, 0, 1],
        "X",
        "X",
        1,
        [1, 1, 1],
        1,
        1,
        None,
        0,
        0,
        None,
        0,
    ]
    stamp: int | str = int(timestamp) if str(timestamp).isdigit() else timestamp
    inner = ["garturlreq", context, art_id, stamp, signature]
    envelope = [["Fbv4je", json.dumps(inner, separators=(",", ":")), None, "0"]]
    return "f.req=" + quote(json.dumps([envelope], separators=(",", ":")))


def google_decoded_url(raw: str) -> str:
    match = _GOOGLE_URL.search(raw or "")
    if not match:
        return ""
    found = match.group(1)
    host = urlparse(found).hostname or ""
    if host == "news.google.com" or host.endswith(".google.com"):
        return ""
    return found


async def resolve_article_url(hass: HomeAssistant, url: str) -> str:
    art_id = google_article_id(url)
    if not art_id:
        return url
    page = await _fetch_raw(hass, google_params_url(url, art_id), 8, _BROWSER)
    found = google_signature(page)
    if not found:
        return ""
    signature, timestamp = found
    raw = await _post_form(hass, _GOOGLE_BATCH, google_batch_body(art_id, timestamp, signature), 8)
    return google_decoded_url(raw)


async def load_article(hass: HomeAssistant, url: str) -> str:
    target = await resolve_article_url(hass, url)
    if not _http_url(target):
        return ""
    return article_body_from_html(await _fetch_raw(hass, target, 8))


def story_feed_urls(pack: str, topic: str, feeds: object, reader_urls: list[str]) -> list[str]:
    urls: list[str] = []
    seen: set[str] = set()
    for url in reader_urls:
        if url and url not in seen:
            seen.add(url)
            urls.append(url)
    _kind, url = choose_feed(pack, topic, feeds)
    if url and url not in seen:
        urls.append(url)
    return urls


def _add_stories(stories: list[dict[str, str]], seen: set[str], found: list[dict[str, str]]) -> None:
    for story in found:
        title = story.get("title") or ""
        if not title or title in seen:
            continue
        if len(stories) >= 30:
            return
        seen.add(title)
        stories.append(story)


async def collect_stories(hass: HomeAssistant, pack: str, topic: str = "", feeds: object = None) -> list[dict[str, str]]:
    _stored, reader_urls = feedreader_sources(hass)
    stories: list[dict[str, str]] = []
    seen: set[str] = set()
    for url in story_feed_urls(pack, topic, feeds, reader_urls):
        _add_stories(stories, seen, stories_from_xml(await _fetch_raw(hass, url), 20))
    if stories or not (topic or "").strip():
        return stories
    _add_stories(stories, seen, stories_from_xml(await _fetch_raw(hass, feed_url(pack)), 20))
    return stories


async def resolve_story(
    hass: HomeAssistant,
    pack: str,
    topic: str,
    feeds: object,
    utterance: str,
    earlier: str = "",
) -> dict[str, str] | None:
    story = pick_story(utterance, await collect_stories(hass, pack, topic, feeds), earlier)
    if story is not None:
        return story
    query = story_query(utterance) or (story_query(earlier) if wants_read(utterance) else "")
    if not query:
        return None
    found = stories_from_xml(await _fetch_raw(hass, search_url(pack, query)), 8)
    return pick_story(query, found)


async def fetch_headlines(hass: HomeAssistant, pack: str, limit: int = _LIMIT) -> list[str]:
    return await fetch_url(hass, feed_url(pack), limit)


async def collect_headlines(hass: HomeAssistant, pack: str, topic: str = "", feeds: object = None) -> list[str]:
    titles: list[str] = []
    seen: set[str] = set()
    stored, urls = feedreader_sources(hass)
    _take(titles, seen, stored)
    if not stored:
        for url in urls:
            if len(titles) >= _LIMIT:
                break
            _take(titles, seen, await fetch_url(hass, url, _LIMIT))
    if len(titles) < _LIMIT:
        _kind, url = choose_feed(pack, topic, feeds)
        _take(titles, seen, await fetch_url(hass, url, _LIMIT))
    return titles
