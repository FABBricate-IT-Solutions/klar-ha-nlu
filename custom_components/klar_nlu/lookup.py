"""Look up a factual question before the LLM may speak.

Country size, population, capital and head of state come from Wikidata.
Anything else comes from the Wikipedia intro. The model may only repeat
what that text contains.
"""

from __future__ import annotations

import json
import logging
import re
import time
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
_UA = "KlarNLU/2026.10.2 (https://github.com/FABBricate-IT-Solutions/klar-ha-nlu)"
_EXTRACT = 1200
_SPEECH = 700
_GLUE = frozenset(
    """
    der die das den dem des ein eine einer einem einen und oder ist sind war hat haben
    wie wer was welche welcher welches wo wann warum wieso von zu zum zur für fuer mit
    the a an is are was of who what which where when why to for in on
    denn mal bitte eigentlich doch eben halt einfach noch gerade auch schon
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


def search_title(payload: object, query: str = "") -> str:
    if not isinstance(payload, dict):
        return ""
    rows = (payload.get("query") or {}).get("search") if isinstance(payload.get("query"), dict) else None
    if not isinstance(rows, list):
        return ""
    titles: list[str] = []
    for row in rows[:5]:
        if not isinstance(row, dict):
            continue
        title = str(row.get("title") or "").strip()
        folded = title.casefold()
        if not title or folded.startswith(("liste ", "list of ", "begriffsklärung", "disambiguation")):
            continue
        titles.append(title)
    if not titles:
        return ""
    needles = [word for word in _words(query) if word not in _GLUE and len(word) >= 4]
    scored = [(_title_score(title, needles), title) for title in titles]
    best = max(scored, default=(0, ""), key=lambda item: item[0])
    if best[0] > 0:
        return best[1]
    return titles[0]


def _title_score(title: str, needles: list[str]) -> int:
    folded = title.casefold().split("(", 1)[0].strip()
    score = sum(len(word) for word in needles if word in folded)
    if folded in needles:
        score += 100
    return score


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


_KINDS = (
    ("area", ("groß", "gross", "größe", "groesse", "fläche", "flaeche", "quadratmeter", "quadratkilometer", "area", "size", "big", "large")),
    ("population", ("einwohner", "bevölkerung", "bevoelkerung", "population")),
    ("head", ("präsident", "praesident", "bundespräsident", "bundespraesident", "president", "staatsoberhaupt")),
    ("government", ("kanzler", "kanzlerin", "bundeskanzler", "chancellor", "regierungschef")),
    ("capital", ("hauptstadt", "capital")),
)
_CACHE: dict[tuple[str, str, str], tuple[float, str]] = {}
_CACHE_TTL = 6 * 60 * 60
_PROPS = {"area": "P2046", "population": "P1082", "head": "P35", "government": "P6", "capital": "P36"}
_SKIP_ENTITY = ("song", "album", "film", "submarine", "olymp", "former", "television", "bicycle", "football", "painting")
_COUNTRY_DESC = ("country", "republik", "republic", "kingdom", "königreich", "staat", "land in")
_REGION_DESC = ("federated state", "state in", "bundesland")


def fact_kind(text: str) -> str:
    words = set(_words(text))
    for kind, cues in _KINDS:
        if words.intersection(cues):
            return kind
    return ""


def fact_subject(text: str, kind: str) -> str:
    cues = next((set(words) for name, words in _KINDS if name == kind), set())
    words = _words(text)
    kept = [word for word in words if word not in _GLUE and word not in cues and word not in {"viele", "viel", "how", "many"}]
    if kept:
        return " ".join(kept)
    if set(words).intersection({"bundespräsident", "bundespraesident", "bundeskanzler"}):
        return "deutschland"
    return ""


def pick_country(payload: object) -> str:
    if not isinstance(payload, dict):
        return ""
    rows = payload.get("search")
    if not isinstance(rows, list):
        return ""
    found: list[tuple[str, str]] = []
    for row in rows[:5]:
        if not isinstance(row, dict):
            continue
        desc = str(row.get("description") or "").casefold()
        eid = str(row.get("id") or "")
        if not eid.startswith("Q") or any(hint in desc for hint in _SKIP_ENTITY):
            continue
        found.append((eid, desc))
    for hints in (_COUNTRY_DESC, _REGION_DESC):
        for eid, desc in found:
            if any(hint in desc for hint in hints):
                return eid
    return ""


def preferred_value(payload: object, prop: str) -> object:
    if not isinstance(payload, dict):
        return None
    rows = (payload.get("claims") or {}).get(prop) if isinstance(payload.get("claims"), dict) else None
    if not isinstance(rows, list) or not rows:
        return None
    chosen = [row for row in rows if isinstance(row, dict) and row.get("rank") == "preferred"] or rows[:1]
    mainsnak = chosen[0].get("mainsnak") if isinstance(chosen[0], dict) else None
    if not isinstance(mainsnak, dict) or mainsnak.get("snaktype") != "value":
        return None
    value = (mainsnak.get("datavalue") or {}).get("value")
    return value


def plain_amount(value: object) -> str:
    if not isinstance(value, dict):
        return ""
    raw = str(value.get("amount") or "").lstrip("+")
    if not raw:
        return ""
    try:
        number = float(raw)
    except ValueError:
        return ""
    if number >= 100:
        return str(int(round(number)))
    return raw


def entity_label(payload: object, eid: str, lang: str) -> str:
    if not isinstance(payload, dict):
        return ""
    entities = payload.get("entities")
    ent = entities.get(eid) if isinstance(entities, dict) else None
    labels = ent.get("labels") if isinstance(ent, dict) else None
    if not isinstance(labels, dict):
        return ""
    for code in (lang, "en", "mul"):
        row = labels.get(code)
        if isinstance(row, dict) and str(row.get("value") or "").strip():
            return str(row["value"]).strip()
    return ""


def spoken_number(amount: str, pack: str) -> str:
    if not amount.isdigit():
        return amount
    number = int(amount)
    if number < 1_000_000:
        return amount
    de = pack == "de" or pack.startswith("de-")
    rounded = round(number / 1_000_000, 1)
    if abs(rounded - round(rounded)) < 0.05:
        shown = str(int(round(rounded)))
    else:
        shown = f"{rounded:.1f}".replace(".", ",") if de else f"{rounded:.1f}"
    if de:
        word = "Million" if shown == "1" else "Millionen"
    else:
        word = "million"
    return f"{shown} {word}"


def measure_sentence(pack: str, kind: str, place: str, amount: str) -> str:
    de = pack == "de" or pack.startswith("de-")
    if kind == "area":
        return f"{place} hat eine Fläche von {amount} Quadratkilometern." if de else f"{place} has an area of {amount} square kilometres."
    if kind == "population":
        spoken = spoken_number(amount, pack)
        return f"{place} hat {spoken} Einwohner." if de else f"{place} has a population of {spoken}."
    return ""


def office_sentence(pack: str, kind: str, place: str, who: str) -> str:
    de = pack == "de" or pack.startswith("de-")
    if kind == "head":
        return f"Staatsoberhaupt von {place} ist {who}." if de else f"The head of state of {place} is {who}."
    if kind == "government":
        return f"Regierungschef von {place} ist {who}." if de else f"The head of government of {place} is {who}."
    if kind == "capital":
        return f"Die Hauptstadt von {place} ist {who}." if de else f"The capital of {place} is {who}."
    return ""


def wikidata_search_url(lang: str, subject: str) -> str:
    params = urlencode(
        {"action": "wbsearchentities", "search": subject, "language": lang, "type": "item", "format": "json", "limit": "5"}
    )
    return f"https://www.wikidata.org/w/api.php?{params}"


def wikidata_claims_url(eid: str, prop: str) -> str:
    params = urlencode({"action": "wbgetclaims", "entity": eid, "property": prop, "format": "json"})
    return f"https://www.wikidata.org/w/api.php?{params}"


def wikidata_labels_url(ids: list[str], lang: str) -> str:
    languages = "en|mul" if lang == "en" else f"{lang}|en|mul"
    params = urlencode(
        {"action": "wbgetentities", "ids": "|".join(ids), "props": "labels", "languages": languages, "format": "json"}
    )
    return f"https://www.wikidata.org/w/api.php?{params}"


async def wikidata_sentence(hass: HomeAssistant, pack: str, kind: str, subject: str, session: Any) -> str:
    key = (pack, kind, subject.casefold())
    cached = _CACHE.get(key)
    if cached is not None and time.monotonic() - cached[0] < _CACHE_TTL:
        return cached[1]
    sentence = await _wikidata_sentence(hass, pack, kind, subject, session)
    if sentence:
        _CACHE[key] = (time.monotonic(), sentence)
    return sentence


async def _wikidata_sentence(hass: HomeAssistant, pack: str, kind: str, subject: str, session: Any) -> str:
    lang = "de" if pack == "de" or pack.startswith("de-") else "en"
    prop = _PROPS.get(kind, "")
    if not prop:
        return ""
    eid = pick_country(_json(await _get(hass, wikidata_search_url(lang, subject), session)))
    if not eid:
        return ""
    value = preferred_value(_json(await _get(hass, wikidata_claims_url(eid, prop), session)), prop)
    if isinstance(value, dict) and value.get("id"):
        labels = _json(await _get(hass, wikidata_labels_url([eid, str(value["id"])], lang), session))
        place = entity_label(labels, eid, lang) or subject
        who = entity_label(labels, str(value["id"]), lang)
        return office_sentence(pack, kind, place, who) if who else ""
    amount = plain_amount(value)
    if not amount:
        return ""
    labels = _json(await _get(hass, wikidata_labels_url([eid], lang), session))
    place = entity_label(labels, eid, lang) or subject
    return measure_sentence(pack, kind, place, amount)


async def lookup_facts(hass: HomeAssistant, pack: str, text: str, session: Any = None) -> list[str]:
    query = lookup_query(text)
    if not query:
        return []
    kind = fact_kind(query)
    subject = fact_subject(query, kind) if kind else ""
    if kind and subject:
        sentence = await wikidata_sentence(hass, pack, kind, subject, session)
        if sentence:
            return [sentence]
    raw_search = await _get(hass, search_url(pack, query), session)
    title = search_title(_json(raw_search), query)
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
