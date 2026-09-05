"""Personality wrap at Assist finish. Post-execute templates live in the engine."""

from __future__ import annotations

try:
    from .clock_speech import finish_clock_speech, strip_clock_seconds
except ImportError:
    from clock_speech import finish_clock_speech, strip_clock_seconds
try:
    from .speech_locale import SPEECH_PACKS
except ImportError:
    try:
        from speech_locale import SPEECH_PACKS
    except ImportError:
        SPEECH_PACKS = {}

_WRAP = 0


def _locale(pack: str) -> dict:
    return SPEECH_PACKS.get(pack) or SPEECH_PACKS.get("en") or {}


def style(speech: str, personality: str, pack: str) -> str:
    global _WRAP
    if personality in {"", "default"}:
        return speech
    variants = list((_locale(pack).get("personality") or {}).get(personality) or [])
    if not variants:
        variants = list((_locale("en").get("personality") or {}).get(personality) or [])
    if not variants:
        return speech
    _WRAP += 1
    prefix = variants[(hash(speech) + _WRAP) % len(variants)]
    if not prefix or speech.startswith(prefix.strip()):
        return speech
    return f"{prefix}{speech}"


__all__ = ["finish_clock_speech", "strip_clock_seconds", "style"]
