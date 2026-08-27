"""LLM-only rewrite of finished NLU replies."""

from __future__ import annotations

import asyncio
import logging
import re
from collections.abc import AsyncIterator
from typing import Any
from uuid import uuid4

try:
    from homeassistant.components import conversation
    from homeassistant.components.conversation import AssistantContent
    from homeassistant.core import Context, HomeAssistant
except ImportError:  # stdlib tests load this module without Home Assistant
    conversation = None  # type: ignore[assignment]
    AssistantContent = None  # type: ignore[assignment,misc]
    Context = Any
    HomeAssistant = Any

try:
    from .fallback import can_use_fallback_agent
except ImportError:  # stdlib tests load this module without a package

    def can_use_fallback_agent(controls_home: bool, chat: bool = False) -> bool:
        del chat
        return not controls_home

try:
    from .refine_voices import _PERSONALITY, _RULES, voice_block
    from .speech import style
except ImportError:  # stdlib tests load this module without a package
    from refine_voices import _PERSONALITY, _RULES, voice_block
    from speech import style

_LOGGER = logging.getLogger(__name__)
_INTENT = re.compile(r"\bHass[A-Z][A-Za-z]+\b")
_DIGITS = re.compile(r"\d+")
_NUM_WORD = re.compile(
    r"\b(?:null|eins|zwei|drei|vier|fünf|sechs|sieben|acht|neun|zehn|"
    r"elf|zwölf|dreizehn|vierzehn|fünfzehn|sechzehn|siebzehn|achtzehn|neunzehn|"
    r"zwanzig|dreissig|dreißig|vierzig|fünfzig|sechzig|siebzig|achtzig|neunzig|"
    r"hundert|tausend|(?:ein|zwei|drei|vier|fünf|sechs|sieben|acht|neun)und(?:zwanzig|"
    r"dreissig|dreißig|vierzig|fünfzig|sechzig|siebzig|achtzig|neunzig)|"
    r"zero|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|"
    r"thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen|twenty|"
    r"thirty|forty|fifty|sixty|seventy|eighty|ninety|hundred|thousand)\b",
    re.IGNORECASE,
)

_INPUT = {
    "de": "{speech}",
    "en": "{speech}",
}

_THINKING_OFF = {"chat_template_kwargs": {"enable_thinking": False}}
_MODEL_KEYS = ("chat_model", "model", "llm_model")


def should_refine(enabled: bool, agent_id: str | None, speech: str) -> bool:
    return bool(enabled and agent_id and speech.strip())


def isolated_conversation_id() -> str:
    return f"klar-nested-{uuid4()}"


def skip_rewrite(decision: str) -> bool:
    """LLM fallback already applied the personality prompt — a second pass rewrites after TTS started."""
    return decision in {"chat", "llm", "chime"}


def nested_llm_session(agent_id: str, language: str | None, prompt: str | None) -> dict[str, Any]:
    return {
        "language": language,
        "agent_id": agent_id,
        "device_id": None,
        "satellite_id": None,
        "extra_system_prompt": prompt,
    }


def drop_same_turn_assistant(content: Any) -> None:
    if not isinstance(content, list) or len(content) < 2:
        return
    if getattr(content[-1], "role", None) == "assistant" and getattr(content[-2], "role", None) == "user":
        content.pop()


_SENTENCE = re.compile(
    r".+?(?:(?:\.\.\.|…|[.!?。！？])[\"'»”’]*)(?=\s+\S|\s*$)",
    re.DOTALL,
)
_ABBREV_TAIL = re.compile(
    r"(?:^|[\s.(])(?:z\.B|u\.a|d\.h|bzw|vgl|usw|etc|ca|dr|nr|st|mr|mrs|ms|prof|vs|[a-zäöü])\.$",
    re.IGNORECASE,
)


def speech_chunks(speech: str) -> list[str]:
    text = speech.strip()
    if not text:
        return []
    chunks = [match.group(0) for match in _SENTENCE.finditer(text)]
    consumed = sum(len(chunk) for chunk in chunks)
    if consumed < len(text):
        chunks.append(text[consumed:])
    merged: list[str] = []
    for chunk in chunks:
        if merged and _ABBREV_TAIL.search(merged[-1].rstrip()):
            merged[-1] += chunk
        elif chunk.strip():
            merged.append(chunk)
    return merged or [text]


async def iter_speech_deltas(speech: str) -> AsyncIterator[dict[str, str]]:
    chunks = speech_chunks(speech)
    if not chunks:
        return
    yield {"role": "assistant", "content": chunks[0]}
    for chunk in chunks[1:]:
        await asyncio.sleep(0)
        yield {"content": chunk}


def _assistant_content(agent_id: str | None, speech: str) -> Any:
    if AssistantContent is None:
        return speech
    return AssistantContent(agent_id=agent_id, content=speech)


async def emit_assistant_speech(chat_log: Any, agent_id: str | None, speech: str) -> None:
    drop_same_turn_assistant(getattr(chat_log, "content", None))
    streamer = getattr(chat_log, "async_add_delta_content_stream", None)
    if callable(streamer) and speech.strip():
        posted = streamer(agent_id, iter_speech_deltas(speech))
        if hasattr(posted, "__aiter__"):
            async for _ in posted:
                pass
            return
    body = _assistant_content(agent_id, speech)
    posted = getattr(chat_log, "async_add_assistant_content", None)
    result = posted(body) if callable(posted) else None
    if hasattr(result, "__aiter__"):
        async for _ in result:
            pass
        return
    chat_log.async_add_assistant_content_without_tools(body)


async def async_finish_speech(
    hass: HomeAssistant,
    enabled: bool,
    agent_id: str | None,
    controls_home: bool,
    speech: str,
    context: Context,
    language: str | None,
    pack: str,
    personality: str,
    extra_prompt: str | None,
) -> str:
    if not should_refine(enabled, agent_id, speech):
        return style(speech, personality, pack)
    refined = await async_refine_speech(
        hass, str(agent_id), controls_home, speech, context, language, pack, personality, extra_prompt
    )
    return refined or style(speech, personality, pack)


_STAMP_BAN = (
    "zur kenntnis genommen",
    "notiert",
    "vermerkt",
    "besorgt",
    "soweit gemeldet",
    "duly noted",
    "taken into account",
    "noted.",
    "enregistré",
    "enregistre",
    "pris en note",
    "fehlinterpretation",
    "genoteerd",
)


def refine_prompt(pack: str, personality: str, extra: str | None) -> str:
    if personality not in _PERSONALITY:
        personality = "default"
    custom = (extra or "").strip()
    stock = voice_block(pack, personality)
    voice = custom if _usable_extra(custom, pack) else stock
    if pack == "en" or pack.startswith("en-"):
        rules = _RULES["en"]
    elif pack == "de" or pack.startswith("de-"):
        rules = _RULES["de"]
    else:
        rules = _RULES["meta"]
    lock = _language_lock(pack)
    return f"{lock}\n\n{rules}\n\n{voice}\n\n{lock}"


def _usable_extra(custom: str, pack: str) -> bool:
    if not custom:
        return False
    german = pack == "de" or pack.startswith("de-")
    if not german and any(marker in custom for marker in ("Stimme:", "Schalt-Bestätigungen", "Antworte nur auf Deutsch")):
        return False
    return True


def _language_lock(pack: str) -> str:
    try:
        from .lang_select import language_lock

        return language_lock(pack)
    except ImportError:
        pass
    try:
        from lang_select import language_lock

        return language_lock(pack)
    except ImportError:
        if pack == "de" or pack.startswith("de-"):
            return "Antworte nur auf Deutsch. Übersetze nicht in eine andere Sprache."
        if pack == "en" or pack.startswith("en-"):
            return "Answer only in English. Do not translate into German or any other language."
        return f"Answer only in the Klar NLU language ({pack}). Do not translate into German."


def refine_input(speech: str, pack: str) -> str:
    template = _INPUT.get(pack, "{speech}")
    return template.format(speech=speech.strip())


def clean_refined(text: str) -> str:
    speech = (text or "").strip().strip("\"'`“”«»")
    if "\n" in speech:
        speech = " ".join(line.strip() for line in speech.splitlines() if line.strip())
    return speech.strip()


def accept_refined(original: str, refined: str) -> str | None:
    speech = clean_refined(refined)
    if not speech or speech.endswith(("...", "…")):
        return None
    if speech.endswith("?") and not original.rstrip().endswith("?"):
        return None
    if _INTENT.search(speech):
        return None
    source_nums = set(_DIGITS.findall(original))
    result_nums = set(_DIGITS.findall(speech))
    if source_nums != result_nums:
        return None
    if not source_nums and _NUM_WORD.search(speech):
        return None
    if len(speech) > max(len(original) * 6, 280):
        return None
    folded = speech.casefold()
    if any(ban in folded for ban in _STAMP_BAN):
        return None
    return speech


def refine_extra_body() -> dict[str, Any]:
    return dict(_THINKING_OFF)


def speech_from_completion(result: Any) -> str:
    choices = getattr(result, "choices", None) or []
    if not choices:
        return ""
    message = getattr(choices[0], "message", None)
    return str(getattr(message, "content", None) or "").strip()


def _mapping(value: Any) -> dict[str, Any]:
    if isinstance(value, dict):
        return value
    data = getattr(value, "data", None)
    return data if isinstance(data, dict) else {}


def _first_model(*sources: Any) -> str | None:
    for source in sources:
        data = _mapping(source)
        for key in _MODEL_KEYS:
            model = str(data.get(key) or "").strip()
            if model:
                return model
    return None


def _openai_client(raw: Any) -> Any:
    if raw is not None and hasattr(raw, "chat"):
        return raw
    inner = getattr(raw, "client", None)
    if inner is not None and hasattr(inner, "chat"):
        return inner
    return None


def llm_client_and_model(hass: HomeAssistant, agent_id: str) -> tuple[Any, str] | None:
    if conversation is None:
        return None
    try:
        agent = conversation.async_get_agent(hass, agent_id)
    except Exception:  # noqa: BLE001 — agent lookup is a system boundary
        return None
    if agent is None:
        return None
    entry = getattr(agent, "entry", None) or getattr(agent, "_entry", None)
    client = _openai_client(getattr(entry, "runtime_data", None))
    if client is None:
        client = _openai_client(getattr(agent, "client", None) or getattr(agent, "_client", None))
    if client is None:
        return None
    model = _first_model(getattr(agent, "subentry", None), getattr(entry, "options", None), getattr(entry, "data", None))
    if not model:
        return None
    return client, model


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
    prompt = refine_prompt(pack, personality, extra_prompt)
    _LOGGER.debug("LLM-Refine Stimme %s", personality)
    user = refine_input(speech, pack)
    raw = await _async_refine_raw(
        hass, agent_id, user, prompt, language or pack, context, controls_home
    )
    return accept_refined(speech, raw or "")


async def _async_refine_raw(
    hass: HomeAssistant,
    agent_id: str,
    user: str,
    prompt: str,
    language: str,
    context: Context,
    controls_home: bool,
) -> str | None:
    resolved = llm_client_and_model(hass, agent_id)
    if resolved is not None:
        client, model = resolved
        try:
            result = await client.chat.completions.create(
                model=model,
                messages=[
                    {"role": "system", "content": prompt},
                    {"role": "user", "content": user},
                ],
                max_tokens=128,
                temperature=0.65,
                extra_body=refine_extra_body(),
            )
            return speech_from_completion(result)
        except Exception as err:  # noqa: BLE001 — client shape varies by agent
            _LOGGER.debug("LLM-Refine direkt fehlgeschlagen, converse: %s", err)
    if not can_use_fallback_agent(controls_home):
        _LOGGER.warning("LLM-Refine %s hat Assist-Werkzeuge — converse übersprungen", agent_id)
        return None
    try:
        result = await conversation.async_converse(
            hass,
            user,
            isolated_conversation_id(),
            context,
            **nested_llm_session(agent_id, language, prompt),
        )
    except Exception as err:  # noqa: BLE001 — other agent is a system boundary
        _LOGGER.warning("LLM-Refine fehlgeschlagen: %s", err)
        return None
    return speech_from_result(result)
