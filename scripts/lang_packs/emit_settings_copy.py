#!/usr/bin/env python3
"""Emit Settings chrome JSON for every compiled Assist locale."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from lang_packs.web_ui_settings import KEYS
from lang_packs.web_ui_keys import FALLBACKS

DEST = Path(__file__).with_name("web_ui_settings_copy.json")


def v(default: str, locker: str, caring: str, grumpy: str, sarcastic: str, pirate: str) -> dict[str, str]:
    return {
        "personalityDefault": default,
        "personalityButler": "Butler",
        "personalityLocker": locker,
        "personalityFuersorglich": caring,
        "personalityParty": "Party",
        "personalityGrantig": grumpy,
        "personalitySarkastisch": sarcastic,
        "personalityPirat": pirate,
        "personalityHippie": "Hippie",
        "personalityGollum": "Gollum",
        "personalityJarvis": "Jarvis",
    }


def pack(fields: dict[str, str], voice: dict[str, str]) -> dict[str, str]:
    out = {key: FALLBACKS[key] for key in KEYS}
    out.update(voice)
    out.update(fields)
    return {key: out[key] for key in KEYS}


def main() -> None:
    from lang_packs.settings_i18n import LOCALES

    copy = {code: pack(fields, voice) for code, fields, voice in LOCALES}
    expected = {
        "fr", "nl", "es", "it", "pt", "ca", "ro", "pt-BR", "gl",
        "de-CH", "de-AT", "en-GB", "af", "lb", "cy", "eu", "ga", "kw",
        "da", "nb", "sv", "fi", "is", "et", "lt", "lv",
        "cs", "sk", "pl", "hu", "hr", "sl", "bg", "sr", "sr-Latn", "uk",
        "zh-CN", "zh-TW", "zh-HK", "ja", "ko", "th", "vi", "id", "ms", "mn",
        "ar", "he", "fa", "ur", "tr", "el", "hy", "ka",
        "hi", "bn", "gu", "kn", "ml", "mr", "ta", "te", "pa", "ne", "sw",
    }
    if set(copy) != expected:
        raise SystemExit(f"locale drift missing={sorted(expected-set(copy))} extra={sorted(set(copy)-expected)}")
    DEST.write_text(json.dumps(copy, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print("wrote", DEST.relative_to(DEST.parents[2]), "n=", len(copy))


if __name__ == "__main__":
    main()
