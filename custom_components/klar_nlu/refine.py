"""LLM-only rewrite of finished NLU replies. Product accept/prompt live in the engine."""

from __future__ import annotations

import asyncio
import logging
import re
from collections.abc import AsyncIterator, Mapping
from typing import Any
from uuid import uuid4

try:
    from homeassistant.components.conversation import AssistantContent
    from homeassistant.core import Context, HomeAssistant
except ImportError:  # stdlib tests load this module without Home Assistant
    AssistantContent = None  # type: ignore[assignment,misc]
    Context = Any
    HomeAssistant = Any

try:
    from .speech import style
except ImportError:  # stdlib tests load this module without a package
    from speech import style

_LOGGER = logging.getLogger(__name__)

_SENTENCE = re.compile(
    r".+?(?:(?:\.\.\.|…|[.!?。！？])[\"'»”’]*)(?=\s+\S|\s*$)",
    re.DOTALL,
)
_ABBREV_TAIL = re.compile(
    r"(?:^|[\s.(])(?:z\.B|u\.a|d\.h|bzw|vgl|usw|etc|ca|dr|nr|st|mr|mrs|ms|prof|vs|[a-zäöü])\.$",
    re.IGNORECASE,
)
_END = re.compile(r"(?:(?:\.\.\.|…|[.!?。！？])[\"'»”’]*)$")


def should_refine(enabled: bool, agent_id: str | None, speech: str) -> bool:
    del agent_id
    return bool(enabled and speech.strip())


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
    conversation_id: str | None = None,
    chat_log: Any = None,
    publish_agent_id: str | None = None,
) -> tuple[str, bool]:
    if not should_refine(enabled, agent_id, speech):
        return style(speech, personality, pack), False
    refined, posted = await async_refine_speech(
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
        conversation_id,
        chat_log,
        publish_agent_id,
    )
    return (refined or style(speech, personality, pack), posted)


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
    conversation_id: str | None = None,
    chat_log: Any = None,
    publish_agent_id: str | None = None,
) -> tuple[str | None, bool]:
    del agent_id, controls_home, context, allow_tools
    if hass is None:
        return None, False
    try:
        # Cycle: engine_llm → stream → refine. Keep this import in the function.
        from .engine_llm import EngineRefineMissing, EngineUnavailable, complete_engine_refine, stream_engine_refine

        if chat_log is not None:
            streamed = await stream_engine_refine(
                hass,
                speech,
                language or pack,
                personality,
                extra_prompt or "",
                chat_log,
                publish_agent_id,
                conversation_id=conversation_id,
            )
            if streamed is not None:
                text, posted, accepted = streamed
                return (text if accepted else None, posted)
        text, accepted = await complete_engine_refine(
            hass,
            speech,
            language or pack,
            personality,
            extra_prompt or "",
            conversation_id=conversation_id,
        )
        return (text if accepted else None), False
    except EngineRefineMissing:
        _LOGGER.debug("Klar LLM refine route missing")
    except EngineUnavailable:
        _LOGGER.debug("Klar LLM refine unavailable")
    except Exception as err:  # noqa: BLE001 — engine HTTP is a system boundary
        _LOGGER.debug("Klar LLM refine skipped: %s", err)
    return None, False
