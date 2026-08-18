from .languages import LANGUAGE_VARIANTS, SUPPORTED_LANGUAGES

DOMAIN = "klar_nlu"
DEFAULT_URL = "http://127.0.0.1:10520"
DEFAULT_ADDON_URL = "http://klar-nlu:10520"
CONF_URL = "url"
CONF_MODE = "mode"
CONF_FALLBACK_AGENT = "fallback_agent"
CONF_LANGUAGES = "languages"
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
