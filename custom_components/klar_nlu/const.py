DOMAIN = "klar_nlu"
DEFAULT_URL = "http://127.0.0.1:10520"
DEFAULT_ADDON_URL = "http://klar-nlu:10520"
CONF_URL = "url"
CONF_MODE = "mode"
CONF_FALLBACK_AGENT = "fallback_agent"
CONF_LANGUAGES = "languages"
CONF_ASSIST_FILTER = "assist_filter"
CONF_PERSONALITY = "personality"
CONF_TOKEN = "token"
ENGINE_VERSION = "2026.8.7"
DEFAULT_ASSIST_FILTER = True
DEFAULT_PERSONALITY = "default"
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
MODE_LOCAL = "local"
MODE_REMOTE = "remote"
GITHUB_REPO = "FABBricate-IT-Solutions/klar-ha-nlu"
SUPPORTED_LANGUAGES = ("de", "en")
LANGUAGE_VARIANTS = {
    "de": ("de", "de-DE", "de-CH"),
    "en": ("en", "en-US", "en-GB"),
}
