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
    return str(decision or "") in {"clarify", "confirm", "chat"}


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
CONF_ALLOW_LLM_TOOLS = "allow_llm_tools"
CONF_TOKEN = "token"
CONF_CHANNEL = "channel"
CONF_PRODUCT_IN_ENGINE = "product_in_engine"
TOKEN_HEADER = "x-klar-token"
ENGINE_VERSION = "2026.10.2"
DEFAULT_ASSIST_FILTER = True
DEFAULT_PERSONALITY = "default"
DEFAULT_REFINE_PROMPT = ""
DEFAULT_REFINE_SPEECH = False
DEFAULT_NLU_RAG = False
DEFAULT_QUIET_ACK = False
DEFAULT_CALENDAR_LLM = False
DEFAULT_ALLOW_LLM_TOOLS = False
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


def engine_headers(token: object, extra: dict[str, str] | None = None) -> dict[str, str]:
    """Attach the write token when configured. Empty token stays tokenless (loopback)."""
    headers = dict(extra or {})
    text = str(token or "").strip()
    if text:
        headers[TOKEN_HEADER] = text
    return headers


def resolve_personality(value: object) -> str:
    name = str(value or DEFAULT_PERSONALITY)
    if name in PERSONALITIES or name == "custom":
        return name
    return DEFAULT_PERSONALITY


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


def _alias_candidates(url: str) -> list[str]:
    text = _normalize_engine_url(url)
    host = _engine_host(text)
    if not text or not host or "." in host or host in {"localhost", "127.0.0.1"}:
        return [text] if text else []
    fqdn = _url_with_host(text, f"{host}.{_HASSIO_TLD}")
    return [text, fqdn] if fqdn != text else [text]


def engine_url_candidates(url: object) -> list[str]:
    """Try a discovered Supervisor hostname, then the configured host and its FQDN."""
    text = _normalize_engine_url(url)
    if not text:
        return []
    ordered: list[str] = []
    for candidate in (apply_discovered_host(text), text):
        for item in _alias_candidates(candidate):
            if item not in ordered:
                ordered.append(item)
    return ordered


def addon_url_for_channel(channel: object) -> str:
    if resolve_channel(channel) == CHANNEL_STAGING:
        return DEFAULT_STAGING_ADDON_URL
    return DEFAULT_ADDON_URL


def installed_klar_channel(slug: object) -> str | None:
    """Channel for a Supervisor slug, including `<repo>_<slug>` installs."""
    name = str(slug or "").strip().replace("-", "_").lower()
    if not name:
        return None
    if name == STAGING_ADDON_SLUG or name.endswith(f"_{STAGING_ADDON_SLUG}"):
        return CHANNEL_STAGING
    if name == "klar_nlu" or name.endswith("_klar_nlu"):
        return CHANNEL_STABLE
    return None


_discovered_addon_hosts: dict[str, str] = {}


def remember_addon_hosts(hosts: dict[str, str] | None) -> None:
    """Cache Supervisor hostnames so later engine calls can use them."""
    _discovered_addon_hosts.clear()
    for channel, hostname in (hosts or {}).items():
        if str(channel) not in {CHANNEL_STABLE, CHANNEL_STAGING}:
            continue
        label = str(hostname or "").strip().split(".")[0]
        if label:
            _discovered_addon_hosts[str(channel)] = label


def discovered_addon_hosts() -> dict[str, str]:
    return dict(_discovered_addon_hosts)


def apply_discovered_host(url: str, hosts: dict[str, str] | None = None) -> str:
    """Use Supervisor's real hostname when the stored URL still says `klar-nlu`."""
    text = _normalize_engine_url(url)
    host = _engine_host(text)
    kind = _addon_kind(host)
    if not text or kind is None or _supervisor_addon_prefix(host):
        return text
    table = _discovered_addon_hosts if hosts is None else hosts
    discovered = str(table.get(kind) or "").strip().split(".")[0]
    if not discovered or discovered == _addon_label(host):
        return text
    return _url_with_host(text, discovered)


def is_addon_engine_url(url: object) -> bool:
    """True when the engine URL is the Supervisor Klar addon, not loopback or LAN."""
    return _addon_kind(_engine_host(url)) is not None


def addon_sidebar_path(url: object) -> str | None:
    kind = _addon_kind(_engine_host(url))
    if kind == CHANNEL_STAGING:
        return "klar_nlu_staging"
    if kind == CHANNEL_STABLE:
        return "klar_nlu"
    return None


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
    addon_hosts: dict[str, str] | None = None,
) -> tuple[str, str]:
    del supervisor
    text = str(url or "").strip()
    if text and not is_managed_engine_url(text):
        return MODE_REMOTE, text
    if str(mode or MODE_LOCAL) == MODE_LOCAL:
        return MODE_LOCAL, DEFAULT_URL
    if text and _supervisor_addon_prefix(_engine_host(text)):
        return MODE_REMOTE, _retarget_addon_url(text, channel)
    chosen = addon_url_for_channel(channel)
    if addon_hosts:
        chosen = apply_discovered_host(chosen, addon_hosts)
    return MODE_REMOTE, chosen


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
