from urllib.parse import urlparse

from .languages import LANGUAGE_VARIANTS, SUPPORTED_LANGUAGES

DOMAIN = "klar_nlu"
DEFAULT_URL = "http://127.0.0.1:10520"
DEFAULT_ADDON_URL = "http://klar-nlu:10520"
DEFAULT_STAGING_ADDON_URL = "http://klar-nlu-staging:10520"
STAGING_ADDON_SLUG = "klar_nlu_staging"
CONF_URL = "url"
CONF_MODE = "mode"
CONF_FALLBACK_AGENT = "fallback_agent"
CONF_LANGUAGES = "languages"
LANGUAGE_SYSTEM = "system"
LANGUAGE_ALL = "all"
CONF_ASSIST_FILTER = "assist_filter"
CONF_PERSONALITY = "personality"
CONF_REFINE_PROMPT = "refine_prompt"
CONF_REFINE_SPEECH = "refine_speech"
CONF_NLU_RAG = "nlu_rag"
CONF_TOKEN = "token"
CONF_CHANNEL = "channel"
ENGINE_VERSION = "2026.8.31"
DEFAULT_ASSIST_FILTER = True
DEFAULT_PERSONALITY = "default"
DEFAULT_REFINE_PROMPT = ""
DEFAULT_REFINE_SPEECH = False
DEFAULT_NLU_RAG = False
PERSONALITIES = (
    "default",
    "butler",
    "locker",
    "fuersorglich",
    "party",
    "grantig",
    "sarkastisch",
    "pirat",
    "hippie",
    "gollum",
    "jarvis",
)


def resolve_personality(value: object) -> str:
    name = str(value or DEFAULT_PERSONALITY)
    return name if name in PERSONALITIES else DEFAULT_PERSONALITY


MODE_LOCAL = "local"
MODE_REMOTE = "remote"
CHANNEL_STABLE = "stable"
CHANNEL_STAGING = "staging"
DEFAULT_CHANNEL = CHANNEL_STABLE
GITHUB_REPO = "FABBricate-IT-Solutions/klar-ha-nlu"


def resolve_channel(value: object) -> str:
    return CHANNEL_STAGING if str(value or "") == CHANNEL_STAGING else CHANNEL_STABLE


def _normalize_engine_url(url: object) -> str:
    return str(url or "").strip().rstrip("/")


def addon_url_for_channel(channel: object) -> str:
    if resolve_channel(channel) == CHANNEL_STAGING:
        return DEFAULT_STAGING_ADDON_URL
    return DEFAULT_ADDON_URL


def is_managed_engine_url(url: object) -> bool:
    text = _normalize_engine_url(url)
    if text in {
        "",
        _normalize_engine_url(DEFAULT_URL),
        _normalize_engine_url(DEFAULT_ADDON_URL),
        _normalize_engine_url(DEFAULT_STAGING_ADDON_URL),
    }:
        return True
    host = urlparse(text).hostname or ""
    return host in {"klar-nlu", "klar-nlu-staging"} or host.endswith(
        ("-klar-nlu", "-klar-nlu-staging")
    )


def resolve_engine_target(
    *,
    mode: object,
    channel: object,
    url: object,
    supervisor: bool = False,
) -> tuple[str, str]:
    text = str(url or "").strip()
    if text and not is_managed_engine_url(text):
        return MODE_REMOTE, text
    if resolve_channel(channel) == CHANNEL_STAGING:
        if supervisor or str(mode or "") == MODE_REMOTE:
            return MODE_REMOTE, DEFAULT_STAGING_ADDON_URL
        return MODE_LOCAL, DEFAULT_URL
    if supervisor and (
        str(mode or "") == MODE_REMOTE
        or (
            is_managed_engine_url(text)
            and _normalize_engine_url(text) != _normalize_engine_url(DEFAULT_URL)
        )
    ):
        return MODE_REMOTE, DEFAULT_ADDON_URL
    if str(mode or MODE_LOCAL) == MODE_REMOTE:
        return MODE_REMOTE, DEFAULT_ADDON_URL
    return MODE_LOCAL, DEFAULT_URL


def resolve_engine_url(
    *,
    mode: object,
    channel: object,
    url: object,
    supervisor: bool = False,
) -> str:
    return resolve_engine_target(
        mode=mode, channel=channel, url=url, supervisor=supervisor
    )[1]


def channel_for_addon_slug(slug: object) -> str:
    normalized = str(slug or "").replace("-", "_")
    return CHANNEL_STAGING if normalized == STAGING_ADDON_SLUG else CHANNEL_STABLE


def pick_staging_release(releases: object) -> dict | None:
    if not isinstance(releases, list):
        return None
    for item in releases:
        if not isinstance(item, dict) or not item.get("prerelease"):
            continue
        tag = str(item.get("tag_name") or "")
        if "-staging." in tag:
            return item
    return None
