"""Stream LLM tokens into Home Assistant chat-log deltas."""

from __future__ import annotations

import inspect
from collections.abc import AsyncIterator, Callable
from typing import Any

try:
    from .refine import drop_same_turn_assistant, pop_complete_sentences
except ImportError:
    from refine import (  # type: ignore[no-redef]
        drop_same_turn_assistant,
        pop_complete_sentences,
    )

Hold = Callable[[str], bool | None]


async def iter_token_deltas(
    tokens: AsyncIterator[str],
    collected: list[str],
    hold: Hold | None = None,
) -> AsyncIterator[dict[str, str]]:
    opened = False
    pending: list[str] = []
    released = hold is None

    def open_block() -> dict[str, str] | None:
        nonlocal opened
        if opened:
            return None
        opened = True
        return {"role": "assistant"}

    async def publish(text: str) -> AsyncIterator[dict[str, str]]:
        role = open_block()
        if role is not None:
            yield role
        yield {"content": text}

    async for token in tokens:
        collected.append(token)
        if released:
            async for delta in publish(token):
                yield delta
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
            async for delta in publish(part):
                yield delta
        pending.clear()
    if not released and collected:
        decision = hold("".join(collected)) if hold else True
        if decision:
            async for delta in publish("".join(collected)):
                yield delta


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
    if inspect.isawaitable(posted):
        posted = await posted
    if hasattr(posted, "__aiter__"):
        async for _ in posted:
            pass
        return True
    return posted is not False
