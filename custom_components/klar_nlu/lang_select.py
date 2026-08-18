"""Resolve Assist language tags to compiled pack ids."""

from __future__ import annotations

from .const import LANGUAGE_VARIANTS, SUPPORTED_LANGUAGES


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


def enabled_packs(raw: object) -> list[str]:
    if not isinstance(raw, list) or not raw:
        return list(SUPPORTED_LANGUAGES)
    packs = [code for code in raw if code in SUPPORTED_LANGUAGES]
    return packs or list(SUPPORTED_LANGUAGES)


def advertise(packs: list[str]) -> list[str]:
    out: list[str] = []
    for pack in packs:
        out.extend(LANGUAGE_VARIANTS.get(pack, (pack,)))
    return out
