from urllib.parse import urlparse

from .languages import LANGUAGE_VARIANTS, SUPPORTED_LANGUAGES

DOMAIN = "klar_nlu"
FOLLOWUP_SESSION = "klar-followup"


def engine_session_id(device_id: object = None, satellite_id: object = None) -> str:
    for candidate in (satellite_id, device_id):
        text = str(candidate or "").strip()
        if text:
            return f"dev:{text}"[:128]
    return FOLLOWUP_SESSION


def parse_session_id(
    assist_id: object = None,
    device_id: object = None,
    satellite_id: object = None,
) -> str:
    text = str(assist_id or "").strip()
    if text:
        return text[:128]
    return engine_session_id(device_id, satellite_id)


def keeps_conversation(decision: object) -> bool:
    return str(decision or "") in {"clarify", "confirm", "execute", "chat"}


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
CONF_QUIET_ACK = "quiet_ack"
CONF_CALENDAR_LLM = "calendar_llm"
CONF_TOKEN = "token"
CONF_CHANNEL = "channel"
ENGINE_VERSION = "2026.8.57"
DEFAULT_ASSIST_FILTER = True
DEFAULT_PERSONALITY = "default"
DEFAULT_REFINE_PROMPT = ""
DEFAULT_REFINE_SPEECH = False
DEFAULT_NLU_RAG = False
DEFAULT_QUIET_ACK = False
DEFAULT_CALENDAR_LLM = False
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


_ADDON_STABLE = "klar-nlu"
_ADDON_STAGING = "klar-nlu-staging"
_HASSIO_TLD = "local.hass.io"


def _normalize_engine_url(url: object) -> str:
    return str(url or "").strip().rstrip("/")


def _engine_host(url: object) -> str:
    return (urlparse(_normalize_engine_url(url)).hostname or "").lower()


def _addon_label(host: str) -> str:
    return host.split(".")[0].lower()


def _addon_kind(host: str) -> str | None:
    label = _addon_label(host)
    if label == _ADDON_STAGING or label.endswith(f"-{_ADDON_STAGING}"):
        return CHANNEL_STAGING
    if label == _ADDON_STABLE or label.endswith(f"-{_ADDON_STABLE}"):
        return CHANNEL_STABLE
    return None


def _supervisor_addon_prefix(host: str) -> str | None:
    label = _addon_label(host)
    if label.endswith(f"-{_ADDON_STAGING}"):
        prefix = label[: -len(_ADDON_STAGING)].rstrip("-")
        return prefix or None
    if label.endswith(f"-{_ADDON_STABLE}"):
        prefix = label[: -len(_ADDON_STABLE)].rstrip("-")
        return prefix or None
    return None


def _url_with_host(url: str, host: str) -> str:
    parsed = urlparse(_normalize_engine_url(url))
    netloc = f"{host}:{parsed.port}" if parsed.port else host
    return parsed._replace(netloc=netloc).geturl()


def engine_url_candidates(url: object) -> list[str]:
    """Try the configured host first, then Supervisor's `.local.hass.io` name."""
    text = _normalize_engine_url(url)
    host = _engine_host(text)
    if not text or not host or "." in host or host in {"localhost", "127.0.0.1"}:
        return [text] if text else []
    fqdn = _url_with_host(text, f"{host}.{_HASSIO_TLD}")
    return [text, fqdn] if fqdn != text else [text]


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
    return _addon_kind(_engine_host(text)) is not None


def _retarget_addon_url(url: str, channel: object) -> str:
    parsed = urlparse(_normalize_engine_url(url))
    host = parsed.hostname or ""
    prefix = _supervisor_addon_prefix(host)
    if prefix is None and _addon_kind(host) is None:
        return addon_url_for_channel(channel)
    slug = _ADDON_STAGING if resolve_channel(channel) == CHANNEL_STAGING else _ADDON_STABLE
    name = f"{prefix}-{slug}" if prefix else slug
    labels = host.split(".")
    new_host = ".".join([name, *labels[1:]]) if len(labels) > 1 else name
    netloc = f"{new_host}:{parsed.port}" if parsed.port else new_host
    return parsed._replace(netloc=netloc).geturl()


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
    if text and _supervisor_addon_prefix(_engine_host(text)):
        return MODE_REMOTE, _retarget_addon_url(text, channel)
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
