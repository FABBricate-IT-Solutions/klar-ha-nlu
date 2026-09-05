"""Call Klar's OpenAI-compatible chat API. HA only glues Assist streaming."""

from __future__ import annotations

import json
import logging
from collections.abc import AsyncIterator, Mapping
from typing import Any

try:
    from aiohttp import ClientError, ClientTimeout
    from homeassistant.core import HomeAssistant
    from homeassistant.helpers.aiohttp_client import async_get_clientsession
except ImportError:  # stdlib tests load helpers without Home Assistant
    ClientError = Exception  # type: ignore[misc,assignment]
    class ClientTimeout:  # type: ignore[no-redef]
        def __init__(self, total: int = 0) -> None:
            self.total = total

    HomeAssistant = Any
    async_get_clientsession = None  # type: ignore[assignment]

from .const import DOMAIN, engine_url_candidates
from .stream import emit_delta_stream, iter_token_deltas

_LOGGER = logging.getLogger(__name__)
_CHAT_TIMEOUT = ClientTimeout(total=120)


def engine_target(hass: HomeAssistant, url: str | None = None, token: str | None = None) -> tuple[str, str | None] | None:
    if url:
        return url.rstrip("/"), token
    stored = hass.data.get(DOMAIN) or {}
    for item in stored.values():
        if not isinstance(item, dict):
            continue
        found = str(item.get("url") or "").strip()
        if found:
            tok = item.get("token")
            return found.rstrip("/"), str(tok) if tok else None
    return None


def _session(hass: HomeAssistant) -> Any:
    if async_get_clientsession is None:
        return None
    return async_get_clientsession(hass)


async def complete_engine_refine(
    hass: HomeAssistant,
    speech: str,
    language: str,
    personality: str,
    extra_prompt: str = "",
    *,
    url: str | None = None,
    token: str | None = None,
) -> tuple[str, bool]:
    target = engine_target(hass, url, token)
    if target is None:
        raise EngineUnavailable
    base, tok = target
    session = _session(hass)
    if session is None:
        raise EngineUnavailable
    headers = {"X-Klar-Token": tok, "Accept": "application/json"} if tok else {"Accept": "application/json"}
    body = {
        "speech": speech,
        "language": language,
        "personality": personality,
        "extra_prompt": extra_prompt,
        "stream": False,
    }
    last_err: Exception | None = None
    for host in engine_url_candidates(base):
        try:
            async with session.post(
                f"{host}/api/v2/llm/refine",
                json=body,
                headers=headers,
                timeout=_CHAT_TIMEOUT,
            ) as resp:
                if resp.status == 404:
                    raise EngineRefineMissing
                if resp.status == 503:
                    raise EngineUnavailable
                resp.raise_for_status()
                payload = await resp.json()
        except EngineRefineMissing:
            raise
        except EngineUnavailable:
            raise
        except (ClientError, TimeoutError, OSError, ValueError) as err:
            last_err = err
            continue
        parsed = _refine_result(payload)
        if parsed is None:
            raise EngineUnavailable
        return parsed
    if last_err is not None:
        _LOGGER.debug("Klar LLM refine failed: %s", last_err)
    raise EngineUnavailable


def _refine_result(payload: object) -> tuple[str, bool] | None:
    if not isinstance(payload, Mapping):
        return None
    text = str(payload.get("text") or "").strip()
    if payload.get("type") == "done" and text:
        return text, bool(payload.get("accepted"))
    if text:
        return text, bool(payload.get("accepted"))
    return None


async def complete_engine_chat(
    hass: HomeAssistant,
    messages: list[dict[str, str]],
    *,
    url: str | None = None,
    token: str | None = None,
    max_tokens: int = 768,
    temperature: float = 0.65,
) -> str | None:
    target = engine_target(hass, url, token)
    if target is None:
        return None
    base, tok = target
    session = _session(hass)
    if session is None:
        return None
    headers = {"X-Klar-Token": tok, "Accept": "application/json"} if tok else {"Accept": "application/json"}
    body = {"messages": messages, "stream": False, "max_tokens": max_tokens, "temperature": temperature}
    last_err: Exception | None = None
    for host in engine_url_candidates(base):
        try:
            async with session.post(
                f"{host}/api/v2/llm/chat",
                json=body,
                headers=headers,
                timeout=_CHAT_TIMEOUT,
            ) as resp:
                if resp.status == 503:
                    return None
                resp.raise_for_status()
                payload = await resp.json()
        except (ClientError, TimeoutError, OSError, ValueError) as err:
            last_err = err
            continue
        text = _event_text(payload)
        if text:
            return text
        return None
    if last_err is not None:
        _LOGGER.debug("Klar LLM complete failed: %s", last_err)
    return None


async def stream_engine_chat(
    hass: HomeAssistant,
    messages: list[dict[str, str]],
    chat_log: Any,
    agent_id: str | None,
    *,
    url: str | None = None,
    token: str | None = None,
    hold: Any = None,
    max_tokens: int = 768,
    temperature: float = 0.65,
) -> tuple[str, bool] | None:
    target = engine_target(hass, url, token)
    if target is None:
        return None
    collected: list[str] = []

    async def tokens() -> AsyncIterator[str]:
        async for delta in iter_engine_tokens(
            hass, messages, url=url, token=token, max_tokens=max_tokens, temperature=temperature
        ):
            yield delta

    try:
        posted = await emit_delta_stream(chat_log, agent_id, iter_token_deltas(tokens(), collected, hold))
    except EngineUnavailable:
        return None
    except Exception as err:  # noqa: BLE001 — engine HTTP is a system boundary
        _LOGGER.debug("Klar LLM stream failed: %s", err)
        return None
    speech = "".join(collected)
    if not speech:
        return None
    if hold is not None and hold(speech) is not True:
        return speech, False
    return speech, posted


async def iter_engine_tokens(
    hass: HomeAssistant,
    messages: list[dict[str, str]],
    *,
    url: str | None = None,
    token: str | None = None,
    max_tokens: int = 768,
    temperature: float = 0.65,
) -> AsyncIterator[str]:
    target = engine_target(hass, url, token)
    if target is None:
        raise EngineUnavailable
    base, tok = target
    session = _session(hass)
    if session is None:
        raise EngineUnavailable
    headers = {"X-Klar-Token": tok, "Accept": "text/event-stream"} if tok else {"Accept": "text/event-stream"}
    body = {"messages": messages, "stream": True, "max_tokens": max_tokens, "temperature": temperature}
    last_err: Exception | None = None
    for host in engine_url_candidates(base):
        try:
            async with session.post(
                f"{host}/api/v2/llm/chat",
                json=body,
                headers=headers,
                timeout=_CHAT_TIMEOUT,
            ) as resp:
                if resp.status == 503:
                    raise EngineUnavailable
                resp.raise_for_status()
                async for event in _iter_sse_json(resp.content):
                    if event.get("type") == "delta":
                        text = str(event.get("text") or "")
                        if text:
                            yield text
                    elif event.get("type") == "error":
                        raise RuntimeError(str(event.get("message") or "llm error"))
            return
        except EngineUnavailable:
            raise
        except (ClientError, TimeoutError, OSError) as err:
            last_err = err
            continue
    if last_err is not None:
        raise last_err
    raise EngineUnavailable


class EngineUnavailable(Exception):
    """Klar has no LLM endpoint configured."""


class EngineRefineMissing(Exception):
    """Engine has no POST /api/v2/llm/refine yet."""


def _event_text(payload: object) -> str:
    if not isinstance(payload, Mapping):
        return ""
    if payload.get("type") == "done":
        return str(payload.get("text") or "").strip()
    return str(payload.get("text") or "").strip()


async def _iter_sse_json(content: Any) -> AsyncIterator[dict[str, Any]]:
    buf = b""
    async for chunk in content.iter_any():
        buf += chunk
        while b"\n\n" in buf:
            raw, buf = buf.split(b"\n\n", 1)
            for line in raw.replace(b"\r\n", b"\n").split(b"\n"):
                if not line.startswith(b"data:"):
                    continue
                payload = line[5:].strip()
                if not payload or payload == b"[DONE]":
                    continue
                try:
                    data = json.loads(payload)
                except json.JSONDecodeError:
                    continue
                if isinstance(data, dict):
                    yield data
