"""Resolve Assist language tags to compiled pack ids."""

from __future__ import annotations

from .const import LANGUAGE_ALL, LANGUAGE_SYSTEM, LANGUAGE_VARIANTS, SUPPORTED_LANGUAGES


def resolve_pack(language: str | None, enabled: list[str] | None = None) -> str:
    if language:
        tag = language.replace("_", "-")
        matched = longest_pack(tag)
        if matched:
            return matched
        for code, variants in LANGUAGE_VARIANTS.items():
            if any(tag.lower() == variant.lower() for variant in variants):
                return code
    if enabled:
        return enabled[0]
    return SUPPORTED_LANGUAGES[0]


def longest_pack(tag: str) -> str | None:
    parts = tag.split("-")
    for length in range(len(parts), 0, -1):
        candidate = "-".join(parts[:length])
        for code in SUPPORTED_LANGUAGES:
            if code.lower() == candidate.lower():
                return code
    return None


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


def advertise(packs: list[str]) -> list[str]:
    out: list[str] = []
    for pack in packs:
        out.extend(LANGUAGE_VARIANTS.get(pack, (pack,)))
    return out
