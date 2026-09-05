"""LLM-only rewrite of finished NLU replies."""

from __future__ import annotations

import asyncio
import logging
import re
from collections.abc import AsyncIterator, Mapping
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

    def can_use_fallback_agent(
        controls_home: bool, chat: bool = False, allow_tools: bool = False
    ) -> bool:
        del chat
        return allow_tools or not controls_home

try:
    from .clock_speech import strip_clock_seconds
    from .refine_voices import _PERSONALITY, _RULES, voice_block
    from .speech import style
except ImportError:  # stdlib tests load this module without a package
    from clock_speech import strip_clock_seconds
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
    return decision in {"chat", "llm", "chime", "error"}


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


_END = re.compile(r"(?:(?:\.\.\.|…|[.!?。！？])[\"'»”’]*)$")


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


def sentence_finished(chunk: str) -> bool:
    text = chunk.rstrip()
    return bool(text) and bool(_END.search(text)) and not _ABBREV_TAIL.search(text)


def pop_complete_sentences(buf: str) -> tuple[list[str], str]:
    chunks = speech_chunks(buf) if buf.strip() else []
    if not chunks:
        return [], buf
    complete: list[str] = []
    rest: list[str] = []
    for index, chunk in enumerate(chunks):
        last = index == len(chunks) - 1
        if rest or (last and not sentence_finished(chunk)):
            rest.append(chunk)
        else:
            complete.append(chunk)
    return complete, "".join(rest)


def speech_from_stream_delta(chunk: Any) -> str:
    if isinstance(chunk, dict):
        choices = chunk.get("choices") or []
        if not choices:
            return ""
        delta = choices[0].get("delta") if isinstance(choices[0], dict) else None
        if isinstance(delta, dict):
            return str(delta.get("content") or "")
        return ""
    choices = getattr(chunk, "choices", None) or []
    if not choices:
        return ""
    delta = getattr(choices[0], "delta", None)
    if isinstance(delta, dict):
        return str(delta.get("content") or "")
    return str(getattr(delta, "content", None) or "")


async def iter_speech_deltas(speech: str) -> AsyncIterator[dict[str, str]]:
    chunks = speech_chunks(speech)
    if not chunks:
        return
    yield {"role": "assistant"}
    for chunk in chunks:
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
    allow_tools: bool = False,
) -> str:
    if not should_refine(enabled, agent_id, speech):
        return style(speech, personality, pack)
    refined = await async_refine_speech(
        hass,
        str(agent_id),
        controls_home,
        speech,
        context,
        language,
        pack,
        personality,
        extra_prompt,
        allow_tools,
    )
    return refined or style(speech, personality, pack)


_LIGHT_CLAIM = ("licht", "light", "lampe", "lamp")
_WEATHER_WORDS = (
    "weather",
    "forecast",
    "degrees",
    "celsius",
    "fahrenheit",
    "humidity",
    "precipitation",
    "sunny",
    "cloudy",
    "rain",
    "rainy",
    "raining",
    "wetter",
    "vorhersage",
    "regen",
    "sonnig",
    "regnerisch",
    "bewölkt",
    "bewolkt",
)
_WEATHER_STEMS = ("°c", "°f", "luftfeucht")
_WEATHER_WORD = re.compile(r"\b(?:" + "|".join(_WEATHER_WORDS) + r")\b")


def _weather_claim(text: str) -> bool:
    fold = (text or "").casefold()
    if any(stem in fold for stem in _WEATHER_STEMS):
        return True
    return bool(_WEATHER_WORD.search(fold))


def _invents_weather(original: str, refined: str) -> bool:
    return _weather_claim(refined) and not _weather_claim(original)
_FAIL_CLAIM = ("nicht geklappt", "did not work", "nicht erreichbar", "not available")
_DONE_CLAIM = ("ist an", "is on", "läuft", "playing", "eingeschaltet")
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
    return strip_clock_seconds(speech.strip())


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
    original_fold = original.casefold()
    if any(ban in folded for ban in _STAMP_BAN):
        return None
    if any(word in folded for word in _LIGHT_CLAIM) and not any(word in original_fold for word in _LIGHT_CLAIM):
        return None
    if _invents_weather(original, speech):
        return None
    if any(word in original_fold for word in _FAIL_CLAIM) and any(word in folded for word in _DONE_CLAIM):
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
    if isinstance(value, Mapping):
        return dict(value)
    data = getattr(value, "data", None)
    if isinstance(data, Mapping):
        return dict(data)
    options = getattr(value, "options", None)
    if isinstance(options, Mapping):
        return dict(options)
    return {}


def _model_name(value: Any) -> str | None:
    if isinstance(value, str) and value.strip():
        return value.strip()
    model = getattr(value, "model", None)
    if isinstance(model, str) and model.strip():
        return model.strip()
    return None


def _first_model(*sources: Any) -> str | None:
    for source in sources:
        named = _model_name(source)
        if named:
            return named
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
    client = None
    for raw in (
        getattr(entry, "runtime_data", None),
        getattr(agent, "client", None),
        getattr(agent, "_client", None),
        getattr(agent, "openai", None),
        getattr(getattr(agent, "coordinator", None), "client", None),
    ):
        client = _openai_client(raw)
        if client is not None:
            break
    if client is None:
        _LOGGER.warning("LLM-Stream ohne OpenAI-Client für %s", agent_id)
        return None
    model = _first_model(
        getattr(agent, "model", None),
        getattr(agent, "subentry", None),
        getattr(agent, "options", None),
        getattr(entry, "options", None),
        getattr(entry, "data", None),
    )
    if not model:
        _LOGGER.warning("LLM-Stream ohne Modell für %s", agent_id)
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
    allow_tools: bool = False,
) -> str | None:
    if conversation is None:
        return None
    try:
        # Cycle: engine_llm → stream → refine. Keep this import in the function.
        from .engine_llm import EngineRefineMissing, EngineUnavailable, complete_engine_refine

        text, accepted = await complete_engine_refine(
            hass,
            speech,
            language or pack,
            personality,
            extra_prompt or "",
        )
        return text if accepted else None
    except EngineRefineMissing:
        _LOGGER.debug("Klar LLM refine route missing")
    except EngineUnavailable:
        _LOGGER.debug("Klar LLM refine unavailable")
    except Exception as err:  # noqa: BLE001 — engine HTTP is a system boundary
        _LOGGER.debug("Klar LLM refine skipped: %s", err)
    prompt = refine_prompt(pack, personality, extra_prompt)
    _LOGGER.debug("LLM-Refine Stimme %s", personality)
    user = refine_input(speech, pack)
    raw = await _async_refine_raw(
        hass, agent_id, user, prompt, language or pack, context, controls_home, allow_tools
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
    allow_tools: bool = False,
) -> str | None:
    try:
        from .engine_llm import complete_engine_chat

        engine_text = await complete_engine_chat(
            hass,
            [
                {"role": "system", "content": prompt},
                {"role": "user", "content": user},
            ],
            max_tokens=128,
            temperature=0.65,
        )
        if engine_text:
            return engine_text
    except Exception as err:  # noqa: BLE001 — engine HTTP is a system boundary
        _LOGGER.debug("Klar LLM refine skipped: %s", err)
    if not can_use_fallback_agent(controls_home, allow_tools=allow_tools):
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
