"""Where the news briefing reads from: one feed per language pack, sections, topic search."""

from __future__ import annotations

from urllib.parse import urlencode

# A publisher feed in the language of the pack. Packs without one read the Google News
# edition of their language. kw has no news feed of its own and reads the British one.
_FEEDS = {
    "de": "https://www.tagesschau.de/infoservices/alle-meldungen-100~rss2.xml",
    "de-CH": "https://www.tagesschau.de/infoservices/alle-meldungen-100~rss2.xml",
    "de-AT": "https://www.tagesschau.de/infoservices/alle-meldungen-100~rss2.xml",
    "en": "https://feeds.bbci.co.uk/news/world/rss.xml",
    "en-GB": "https://feeds.bbci.co.uk/news/world/rss.xml",
    "kw": "https://feeds.bbci.co.uk/news/rss.xml",
    "fr": "https://www.franceinfo.fr/titres.rss",
    "es": "https://feeds.bbci.co.uk/mundo/rss.xml",
    "pt-BR": "https://feeds.bbci.co.uk/portuguese/rss.xml",
    "ar": "https://feeds.bbci.co.uk/arabic/rss.xml",
    "fa": "https://feeds.bbci.co.uk/persian/rss.xml",
    "ur": "https://feeds.bbci.co.uk/urdu/rss.xml",
    "hi": "https://feeds.bbci.co.uk/hindi/rss.xml",
    "bn": "https://feeds.bbci.co.uk/bengali/rss.xml",
    "mr": "https://feeds.bbci.co.uk/marathi/rss.xml",
    "ne": "https://feeds.bbci.co.uk/nepali/rss.xml",
    "gu": "https://feeds.bbci.co.uk/gujarati/rss.xml",
    "pa": "https://feeds.bbci.co.uk/punjabi/rss.xml",
    "ta": "https://feeds.bbci.co.uk/tamil/rss.xml",
    "te": "https://feeds.bbci.co.uk/telugu/rss.xml",
    "kn": "https://www.prajavani.net/stories.rss",
    "sw": "https://feeds.bbci.co.uk/swahili/rss.xml",
    "ja": "https://www.nhk.or.jp/rss/news/cat0.xml",
    "zh-CN": "https://feeds.bbci.co.uk/zhongwen/simp/rss.xml",
    "zh-TW": "https://feeds.bbci.co.uk/zhongwen/trad/rss.xml",
    "zh-HK": "https://feeds.bbci.co.uk/zhongwen/trad/rss.xml",
    "th": "https://feeds.bbci.co.uk/thai/rss.xml",
    "da": "https://www.dr.dk/nyheder/service/feeds/allenyheder",
    "is": "https://www.ruv.is/rss/frettir",
    "ga": "https://www.rte.ie/feeds/rss/?index=/gaeilge/",
    "cy": "https://feeds.bbci.co.uk/cymrufyw/rss.xml",
    "eu": "https://www.eitb.eus/eu/rss/albisteak/",
    "gl": "https://www.nosdiario.gal/rss/",
    "lb": "https://www.rtl.lu/rss/headlines",
    "af": "https://maroelamedia.co.za/feed/",
    "hr": "https://www.index.hr/rss/vijesti",
    "hy": "https://hetq.am/hy/rss",
    "ka": "https://netgazeti.ge/feed/",
    "mn": "https://ikon.mn/rss",
}
# Google News editions as (hl, gl, ceid). A language missing here has no edition, and
# Google answers a search in it with the edition of another language.
_EDITIONS = {
    "ar": ("ar", "EG", "EG:ar"),
    "bg": ("bg", "BG", "BG:bg"),
    "bn": ("bn", "BD", "BD:bn"),
    "ca": ("ca-ES", "ES", "ES:ca"),
    "cs": ("cs", "CZ", "CZ:cs"),
    "de": ("de", "DE", "DE:de"),
    "de-AT": ("de", "AT", "AT:de"),
    "de-CH": ("de", "CH", "CH:de"),
    "el": ("el", "GR", "GR:el"),
    "en": ("en-US", "US", "US:en"),
    "en-GB": ("en-GB", "GB", "GB:en"),
    "es": ("es", "ES", "ES:es"),
    "et": ("et-EE", "EE", "EE:et"),
    "fi": ("fi-FI", "FI", "FI:fi"),
    "fr": ("fr", "FR", "FR:fr"),
    "gu": ("gu-IN", "IN", "IN:gu"),
    "he": ("he", "IL", "IL:he"),
    "hi": ("hi", "IN", "IN:hi"),
    "hu": ("hu", "HU", "HU:hu"),
    "id": ("id", "ID", "ID:id"),
    "it": ("it", "IT", "IT:it"),
    "ja": ("ja", "JP", "JP:ja"),
    "ko": ("ko", "KR", "KR:ko"),
    "lt": ("lt", "LT", "LT:lt"),
    "lv": ("lv", "LV", "LV:lv"),
    "ml": ("ml", "IN", "IN:ml"),
    "mr": ("mr", "IN", "IN:mr"),
    "ms": ("ms-MY", "MY", "MY:ms"),
    "nb": ("no", "NO", "NO:no"),
    "nl": ("nl", "NL", "NL:nl"),
    "pa": ("pa-IN", "IN", "IN:pa"),
    "pl": ("pl", "PL", "PL:pl"),
    "pt": ("pt-PT", "PT", "PT:pt-150"),
    "pt-BR": ("pt-BR", "BR", "BR:pt-419"),
    "ro": ("ro", "RO", "RO:ro"),
    "sk": ("sk", "SK", "SK:sk"),
    "sl": ("sl", "SI", "SI:sl"),
    "sr": ("sr", "RS", "RS:sr"),
    "sv": ("sv", "SE", "SE:sv"),
    "ta": ("ta", "IN", "IN:ta"),
    "te": ("te", "IN", "IN:te"),
    "th": ("th", "TH", "TH:th"),
    "tr": ("tr", "TR", "TR:tr"),
    "uk": ("uk", "UA", "UA:uk"),
    "vi": ("vi", "VN", "VN:vi"),
    "zh-CN": ("zh-CN", "CN", "CN:zh-Hans"),
    "zh-TW": ("zh-TW", "TW", "TW:zh-Hant"),
    "zh-HK": ("zh-HK", "HK", "HK:zh-Hant"),
}
DEFAULT_FEED = _FEEDS["en"]
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


def feed_url(pack: str) -> str:
    """The pack's own feed, else the Google News edition of its language, else world news in English."""
    for code in (pack, pack.split("-", 1)[0]):
        if code in _FEEDS:
            return _FEEDS[code]
        if code in _EDITIONS:
            hl, gl, ceid = _EDITIONS[code]
            return "https://news.google.com/rss?" + urlencode({"hl": hl, "gl": gl, "ceid": ceid})
    return DEFAULT_FEED


def edition(pack: str) -> tuple[str, str, str] | None:
    """The Google News edition of the pack as (hl, gl, ceid), if its language has one."""
    return _EDITIONS.get(pack) or _EDITIONS.get(pack.split("-", 1)[0])


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
    hl, gl, ceid = edition(raw) or (lang, "US", f"US:{lang}")
    query = " ".join((topic or "").split())[:80]
    return "https://news.google.com/rss/search?" + urlencode({"q": query, "hl": hl, "gl": gl, "ceid": ceid})
