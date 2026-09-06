"""Advertise Home Assistant Assist tools on Klar's conversation entity (2026.9)."""

from __future__ import annotations

import json
import logging
from collections.abc import Mapping
from typing import Any

from .const import DOMAIN
from .engine_llm import iter_engine_assist_events

_LOGGER = logging.getLogger(__name__)
_MAX_ROUNDS = 10


async def stream_assist_with_ha_tools(
    hass: Any,
    user_input: Any,
    chat_log: Any,
    text: str,
    language: str,
    personality: str,
    *,
    kind: str = "auto",
    nlu_rag: bool = False,
    retrieval: dict[str, Any] | None = None,
    facts: str | list[str] | None = None,
    history: list[tuple[str, str]] | None = None,
    extra_system: str | None = None,
    extra_prompt: str = "",
    conversation_id: str | None = None,
    url: str | None = None,
    token: str | None = None,
    publish: bool = True,
    agent_id: str | None = None,
) -> tuple[str, bool] | None:
    if nlu_rag:
        return None
    if not await provide_llm_data(chat_log, user_input, extra_system):
        return None
    tools = openai_tools_from_chat_log(chat_log)
    if not tools:
        return None
    tool_messages: list[dict[str, Any]] = []
    spoken = ""
    posted = False
    for _ in range(_MAX_ROUNDS):
        collected: list[str] = []
        calls: list[dict[str, str]] = []
        async for event in iter_engine_assist_events(
            hass,
            text,
            language,
            personality,
            kind=kind,
            allow_tools=True,
            nlu_rag=False,
            retrieval=retrieval,
            facts=facts,
            history=history,
            extra_system=extra_system,
            extra_prompt=extra_prompt,
            conversation_id=conversation_id,
            url=url,
            token=token,
            tools=tools,
            tool_messages=tool_messages,
        ):
            kind_name = str(event.get("type") or "")
            if kind_name == "delta":
                piece = str(event.get("text") or "")
                if piece:
                    collected.append(piece)
                    if publish and chat_log is not None:
                        posted = _append_delta(chat_log, agent_id, piece) or posted
            elif kind_name == "tool_call":
                calls.append(
                    {
                        "id": str(event.get("id") or ""),
                        "name": str(event.get("name") or ""),
                        "arguments": str(event.get("arguments") or "{}"),
                    }
                )
        spoken = "".join(collected)
        if not calls:
            return (spoken, posted) if spoken else None
        if not await apply_tool_calls(chat_log, calls):
            return (spoken, posted) if spoken else None
        tool_messages = messages_from_chat_log(chat_log)
        if not unresponded_tool_results(chat_log):
            break
    return (spoken, posted) if spoken else None


async def provide_llm_data(chat_log: Any, user_input: Any, extra_system: str | None) -> bool:
    provide = getattr(chat_log, "async_provide_llm_data", None)
    if provide is None:
        return False
    context = None
    as_ctx = getattr(user_input, "as_llm_context", None)
    if callable(as_ctx):
        try:
            context = as_ctx(DOMAIN)
        except TypeError:
            context = as_ctx()
    try:
        await provide(context, ["assist"], None, extra_system)
        return getattr(chat_log, "llm_api", None) is not None
    except Exception as err:  # noqa: BLE001 — HA LLM API is a system boundary
        _LOGGER.debug("Klar could not provide Assist LLM data: %s", err)
        return False


def openai_tools_from_chat_log(chat_log: Any) -> list[dict[str, Any]]:
    api = getattr(chat_log, "llm_api", None)
    rows = getattr(api, "tools", None) if api is not None else None
    if not rows:
        return []
    out: list[dict[str, Any]] = []
    for tool in rows:
        name = str(getattr(tool, "name", "") or "")
        if not name:
            continue
        parameters = getattr(tool, "parameters", None)
        if hasattr(parameters, "model_dump"):
            parameters = parameters.model_dump()
        elif hasattr(parameters, "dict"):
            parameters = parameters.dict()
        if not isinstance(parameters, dict):
            parameters = {"type": "object", "properties": {}}
        out.append(
            {
                "type": "function",
                "function": {
                    "name": name,
                    "description": str(getattr(tool, "description", "") or ""),
                    "parameters": parameters,
                },
            }
        )
    return out


async def apply_tool_calls(chat_log: Any, calls: list[dict[str, str]]) -> bool:
    add = getattr(chat_log, "async_add_assistant_content", None)
    if add is None:
        return await _call_llm_api(chat_log, calls)
    tool_inputs = []
    for call in calls:
        tool_inputs.append(_tool_input(call))
    content = _assistant_content(tool_inputs)
    try:
        result = add(content)
        if hasattr(result, "__aiter__"):
            async for _item in result:
                pass
        elif hasattr(result, "__await__"):
            await result
        return True
    except Exception as err:  # noqa: BLE001 — HA tool execution is a system boundary
        _LOGGER.debug("Klar Assist tool call failed: %s", err)
        return False


async def _call_llm_api(chat_log: Any, calls: list[dict[str, str]]) -> bool:
    api = getattr(chat_log, "llm_api", None)
    call_tool = getattr(api, "async_call_tool", None)
    if call_tool is None:
        return False
    try:
        for call in calls:
            await call_tool(_tool_input(call))
        return True
    except Exception as err:  # noqa: BLE001 — HA tool execution is a system boundary
        _LOGGER.debug("Klar Assist llm_api call failed: %s", err)
        return False


def messages_from_chat_log(chat_log: Any) -> list[dict[str, Any]]:
    rows = list(getattr(chat_log, "content", None) or [])
    out: list[dict[str, Any]] = []
    for item in rows:
        role = str(getattr(item, "role", "") or getattr(item, "type", "") or "")
        if role in {"assistant", "tool"} or getattr(item, "tool_calls", None):
            message: dict[str, Any] = {"role": "assistant" if getattr(item, "tool_calls", None) else role}
            text = str(getattr(item, "content", "") or getattr(item, "text", "") or "")
            if text:
                message["content"] = text
            tool_calls = getattr(item, "tool_calls", None)
            if tool_calls:
                message["role"] = "assistant"
                message["tool_calls"] = [_openai_tool_call(call) for call in tool_calls]
            tool_call_id = getattr(item, "tool_call_id", None) or getattr(item, "id", None)
            if role == "tool" or getattr(item, "tool_result", None) is not None:
                message["role"] = "tool"
                message["tool_call_id"] = str(tool_call_id or "")
                if not text:
                    message["content"] = _tool_result_text(item)
            if message.get("role") in {"assistant", "tool"}:
                out.append(message)
    return out


def unresponded_tool_results(chat_log: Any) -> bool:
    return bool(getattr(chat_log, "unresponded_tool_results", False))


def _append_delta(chat_log: Any, agent_id: str | None, piece: str) -> bool:
    add = getattr(chat_log, "async_add_delta_content_stream", None)
    add_text = getattr(chat_log, "async_add_assistant_content_without_tools", None)
    if add_text is None:
        return False
    try:
        add_text(_delta_content(agent_id, piece))
        return True
    except Exception:  # noqa: BLE001 — streaming is best-effort
        del add
        return False


def _tool_input(call: Mapping[str, str]) -> Any:
    name = str(call.get("name") or "")
    raw = str(call.get("arguments") or "{}")
    try:
        args = json.loads(raw) if raw else {}
    except json.JSONDecodeError:
        args = {}
    if not isinstance(args, dict):
        args = {}
    return _SimpleToolInput(str(call.get("id") or name), name, args)


def _openai_tool_call(call: Any) -> dict[str, Any]:
    name = str(getattr(call, "tool_name", None) or getattr(call, "name", "") or "")
    args = getattr(call, "tool_args", None) or getattr(call, "arguments", None) or {}
    if not isinstance(args, str):
        args = json.dumps(args)
    return {
        "id": str(getattr(call, "id", "") or name),
        "type": "function",
        "function": {"name": name, "arguments": args},
    }


def _tool_result_text(item: Any) -> str:
    result = getattr(item, "tool_result", None)
    if result is None:
        result = getattr(item, "content", "")
    if isinstance(result, str):
        return result
    try:
        return json.dumps(result)
    except TypeError:
        return str(result)


def _assistant_content(tool_inputs: list[Any]) -> Any:
    return _SimpleAssistant(tool_inputs)


def _delta_content(agent_id: str | None, text: str) -> Any:
    return _SimpleDelta(agent_id, text)


class _SimpleToolInput:
    def __init__(self, id: str, tool_name: str, tool_args: dict[str, Any]) -> None:
        self.id = id
        self.tool_name = tool_name
        self.tool_args = tool_args
        self.name = tool_name
        self.arguments = tool_args


class _SimpleAssistant:
    def __init__(self, tool_calls: list[Any]) -> None:
        self.tool_calls = tool_calls
        self.content = ""
        self.role = "assistant"


class _SimpleDelta:
    def __init__(self, agent_id: str | None, content: str) -> None:
        self.agent_id = agent_id
        self.content = content
        self.role = "assistant"
