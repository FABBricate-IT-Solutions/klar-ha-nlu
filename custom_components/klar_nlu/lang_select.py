"""Resolve Assist language tags to compiled pack ids."""

from __future__ import annotations

from .const import LANGUAGE_ALL, LANGUAGE_SYSTEM, LANGUAGE_VARIANTS, SUPPORTED_LANGUAGES
from .languages import LANGUAGE_NAMES


def _allowed(enabled: list[str] | None) -> list[str]:
    if not enabled:
        return list(SUPPORTED_LANGUAGES)
    allowed = [code for code in enabled if code in SUPPORTED_LANGUAGES]
    return allowed or list(SUPPORTED_LANGUAGES)


def resolve_pack(language: str | None, enabled: list[str] | None = None) -> str:
    allowed = _allowed(enabled)
    if language:
        tag = language.replace("_", "-")
        matched = longest_pack(tag, allowed)
        if matched:
            return matched
        for code in allowed:
            variants = LANGUAGE_VARIANTS.get(code, (code,))
            if any(tag.lower() == variant.lower() for variant in variants):
                return code
    return allowed[0]


def longest_pack(tag: str, allowed: list[str] | None = None) -> str | None:
    pool = _allowed(allowed)
    parts = tag.split("-")
    for length in range(len(parts), 0, -1):
        candidate = "-".join(parts[:length])
        for code in pool:
            if code.lower() == candidate.lower():
                return code
    return None


def speak_tag(pack: str) -> str:
    variants = LANGUAGE_VARIANTS.get(pack, (pack,))
    return variants[0] if variants else pack


def language_lock(pack: str) -> str:
    name = LANGUAGE_NAMES.get(pack) or pack
    if pack == "de" or pack.startswith("de-"):
        return f"Antworte nur auf {name}. Übersetze nicht ins Englische oder in eine andere Sprache."
    if pack == "en" or pack.startswith("en-"):
        return f"Answer only in {name}. Do not translate into German or any other language."
    return (
        f"Answer only in {name} (Klar NLU pack {pack}). "
        f"Do not translate into German, English, or any other language."
    )


def normalize_language_choice(raw: object) -> str:
    if raw is None:
        return LANGUAGE_ALL
    if isinstance(raw, list):
        if not raw:
            return LANGUAGE_ALL
        if raw[0] == LANGUAGE_SYSTEM:
            return LANGUAGE_SYSTEM
        if raw[0] == LANGUAGE_ALL:
            return LANGUAGE_ALL
        packs = [code for code in raw if code in SUPPORTED_LANGUAGES]
        if len(packs) == 1:
            return packs[0]
        if len(packs) > 1:
            return LANGUAGE_ALL
        return LANGUAGE_SYSTEM
    text = str(raw)
    if text in {LANGUAGE_SYSTEM, LANGUAGE_ALL}:
        return text
    if text in SUPPORTED_LANGUAGES:
        return text
    return LANGUAGE_SYSTEM


def enabled_packs(raw: object, hass_language: str | None = None) -> list[str]:
    choice = normalize_language_choice(raw)
    if choice == LANGUAGE_ALL:
        return list(SUPPORTED_LANGUAGES)
    if choice == LANGUAGE_SYSTEM:
        return [resolve_pack(hass_language)]
    return [choice]


def chrome_locale(hass_language: str | None = None) -> str:
    """HA system/frontend language for integration and app chrome, not the NLU pin."""
    return resolve_pack(hass_language)


def engine_language_state(
    raw: object, hass_language: str | None = None
) -> tuple[list[str], str]:
    """Pin parse packs from the NLU option; chrome always follows Home Assistant."""
    packs = enabled_packs(raw, hass_language)
    choice = normalize_language_choice(raw)
    pinned = [] if choice == LANGUAGE_ALL else list(packs)
    return pinned, chrome_locale(hass_language)


def advertise(packs: list[str]) -> list[str]:
    out: list[str] = []
    for pack in packs:
        out.extend(LANGUAGE_VARIANTS.get(pack, (pack,)))
    return out
