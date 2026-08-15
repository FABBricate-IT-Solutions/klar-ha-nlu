"""LLM-only rewrite of finished NLU replies."""

from __future__ import annotations

import asyncio
import logging
from typing import Any
from uuid import uuid4

try:
    from homeassistant.components import conversation
    from homeassistant.core import Context, HomeAssistant
except ImportError:  # stdlib tests load this module without Home Assistant
    conversation = None  # type: ignore[assignment]
    Context = Any
    HomeAssistant = Any

try:
    from .fallback import can_use_fallback_agent
except ImportError:  # stdlib tests load this module without a package

    def can_use_fallback_agent(controls_home: bool, chat: bool) -> bool:
        return (not controls_home) or chat

_LOGGER = logging.getLogger(__name__)
_TIMEOUT = 4

_PERSONALITY = {
    "default": {
        "de": "natürlich, schlicht und freundlich",
        "en": "natural, plain, and friendly",
    },
    "butler": {
        "de": "höflich, knapp und butlerhaft",
        "en": "polite, concise, and butler-like",
    },
    "locker": {
        "de": "locker, direkt und entspannt",
        "en": "casual, direct, and relaxed",
    },
    "fuersorglich": {
        "de": "warm, fürsorglich und kurz",
        "en": "warm, caring, and short",
    },
    "party": {
        "de": "lebendig, positiv und kurz",
        "en": "lively, upbeat, and short",
    },
    "grantig": {
        "de": "knurrig, aber hilfreich und kurz",
        "en": "grumpy, but helpful and short",
    },
    "sarkastisch": {
        "de": "trocken sarkastisch, aber eindeutig",
        "en": "dryly sarcastic, but clear",
    },
    "pirat": {
        "de": "piratenhaft, knapp und verständlich",
        "en": "pirate-like, concise, and clear",
    },
    "hippie": {
        "de": "entspannt, friedlich und kurz",
        "en": "chill, peaceful, and short",
    },
    "gollum": {
        "de": "gollumartig, aber verständlich und kurz",
        "en": "gollum-like, but clear and short",
    },
}

_BASE = {
    "de": (
        "Formuliere ausschließlich den gegebenen Satz natürlicher um. "
        "Füge keine neuen Fakten hinzu. Ändere keine Geräte, Räume, Zahlen, "
        "Temperaturen, Prozentwerte, Zustände wie an/aus/offen/zu oder Namen. "
        "Rufe keine Home-Assistant-Werkzeuge auf und steuere keine Geräte. "
        "Antworte in derselben Sprache mit genau einer kurzen gesprochenen Antwort. "
        "Gib nur den finalen Satz zurück, keine Erklärung."
    ),
    "en": (
        "Rewrite only the given sentence so it sounds more natural. "
        "Do not add facts. Do not change devices, rooms, numbers, temperatures, "
        "percentages, states such as on/off/open/closed, or names. "
        "Do not call Home Assistant tools and do not control devices. "
        "Answer in the same language with exactly one short spoken reply. "
        "Return only the final sentence, no explanation."
    ),
}


def should_refine(
    enabled: bool,
    agent_id: str | None,
    speech: str,
    chat: bool,
    briefing: bool,
) -> bool:
    return bool(enabled and agent_id and speech.strip() and not chat and not briefing)


def refine_prompt(pack: str, personality: str, extra: str | None) -> str:
    base = _BASE.get(pack, _BASE["de"])
    style = (_PERSONALITY.get(personality) or _PERSONALITY["default"]).get(
        pack,
        _PERSONALITY["default"]["de"],
    )
    custom = (extra or "").strip()
    if pack == "en":
        prompt = f"{base}\n\nPersonality: {style}."
        if custom:
            prompt = f"{prompt}\nAdditional style instruction: {custom}"
        return prompt
    prompt = f"{base}\n\nPersönlichkeit: {style}."
    if custom:
        prompt = f"{prompt}\nZusätzliche Stil-Anweisung: {custom}"
    return prompt


def speech_from_result(result: Any) -> str:
    speech = getattr(result, "response", None)
    speech = getattr(speech, "speech", None) or {}
    plain = speech.get("plain") if isinstance(speech, dict) else None
    if not isinstance(plain, dict):
        return ""
    return str(plain.get("speech") or "").strip()


async def async_refine_speech(
    hass: HomeAssistant,
    agent_id: str,
    controls_home: bool,
    speech: str,
    context: Context,
    language: str | None,
    pack: str,
    personality: str,
    extra_prompt: str | None,
) -> str | None:
    if conversation is None:
        return None
    if not can_use_fallback_agent(controls_home, True):
        _LOGGER.warning(
            "LLM-Refine %s hat Assist-Werkzeuge — chat-only erzwungen",
            agent_id,
        )
        return None
    try:
        result = await asyncio.wait_for(
            conversation.async_converse(
                hass,
                speech,
                f"klar-refine-{uuid4()}",
                context,
                language=language or pack,
                agent_id=agent_id,
                device_id=None,
                satellite_id=None,
                extra_system_prompt=refine_prompt(pack, personality, extra_prompt),
            ),
            timeout=_TIMEOUT,
        )
    except Exception as err:  # noqa: BLE001 — other agent is a system boundary
        _LOGGER.warning("LLM-Refine fehlgeschlagen: %s", err)
        return None
    refined = speech_from_result(result)
    return refined or None
