"""Stream LLM tokens into Home Assistant chat-log deltas."""

from __future__ import annotations

import inspect
from collections.abc import AsyncIterator, Callable
from typing import Any

try:
    from .refine import (
        drop_same_turn_assistant,
        pop_complete_sentences,
        refine_extra_body,
        speech_from_stream_delta,
    )
except ImportError:
    from refine import (  # type: ignore[no-redef]
        drop_same_turn_assistant,
        pop_complete_sentences,
        refine_extra_body,
        speech_from_stream_delta,
    )

Hold = Callable[[str], bool | None]


async def iter_completion_tokens(
    client: Any,
    model: str,
    user: str,
    system: str,
    max_tokens: int = 768,
) -> AsyncIterator[str]:
    create = client.chat.completions.create
    kwargs: dict[str, Any] = {
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "max_tokens": max_tokens,
        "temperature": 0.65,
        "stream": True,
        "extra_body": refine_extra_body(),
    }
    try:
        stream = create(**kwargs)
    except TypeError:
        kwargs.pop("extra_body", None)
        stream = create(**kwargs)
    if inspect.isawaitable(stream):
        stream = await stream
    if hasattr(stream, "__aiter__"):
        async for chunk in stream:
            text = speech_from_stream_delta(chunk)
            if text:
                yield text
        return
    for chunk in stream:
        text = speech_from_stream_delta(chunk)
        if text:
            yield text


async def iter_token_deltas(
    tokens: AsyncIterator[str],
    collected: list[str],
    hold: Hold | None = None,
) -> AsyncIterator[dict[str, str]]:
    first = True
    pending: list[str] = []
    released = hold is None

    def delta(text: str) -> dict[str, str]:
        nonlocal first
        if first:
            first = False
            return {"role": "assistant", "content": text}
        return {"content": text}

    async for token in tokens:
        collected.append(token)
        if released:
            yield delta(token)
            continue
        pending.append(token)
        speech = "".join(collected)
        decision = hold(speech) if hold else True
        if decision is None:
            return
        if decision is False and not pop_complete_sentences(speech)[0]:
            continue
        if decision is False:
            continue
        released = True
        for part in pending:
            yield delta(part)
        pending.clear()
    if not released and collected:
        decision = hold("".join(collected)) if hold else True
        if decision:
            yield delta("".join(collected))


async def emit_delta_stream(chat_log: Any, agent_id: str | None, deltas: AsyncIterator[dict[str, str]]) -> bool:
    if chat_log is None:
        async for _ in deltas:
            pass
        return False
    drop_same_turn_assistant(getattr(chat_log, "content", None))
    streamer = getattr(chat_log, "async_add_delta_content_stream", None)
    if not callable(streamer):
        async for _ in deltas:
            pass
        return False
    posted = streamer(agent_id, deltas)
    if hasattr(posted, "__aiter__"):
        async for _ in posted:
            pass
    return True


async def stream_chat(
    client: Any,
    model: str,
    user: str,
    system: str,
    chat_log: Any,
    agent_id: str | None,
    *,
    hold: Hold | None = None,
    max_tokens: int = 768,
) -> tuple[str, bool]:
    collected: list[str] = []
    tokens = iter_completion_tokens(client, model, user, system, max_tokens)
    posted = await emit_delta_stream(chat_log, agent_id, iter_token_deltas(tokens, collected, hold))
    speech = "".join(collected)
    if not speech:
        return "", False
    if hold is not None and hold(speech) is not True:
        return speech, False
    return speech, posted
