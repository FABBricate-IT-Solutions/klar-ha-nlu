"""Read OpenAI-compatible endpoint fields from a Home Assistant conversation agent."""

from __future__ import annotations

from typing import Any
from urllib.parse import urlparse

try:
    from homeassistant.components import conversation as ha_conversation
except ImportError:
    ha_conversation = None  # type: ignore[assignment]

_MODEL_KEYS = ("chat_model", "llm_model", "model")
_BASE_KEYS = ("openai_api_base", "openai_base_url", "llm_base_url", "api_base", "base_url")
_KEY_KEYS = ("openai_api_key", "llm_api_key", "api_key")
_DEFAULT_BASE = "https://api.openai.com/v1"


def openai_compatible_endpoint(hass: Any, agent_id: str | None) -> dict[str, str] | None:
    """Return base_url, api_key, and model when the agent speaks OpenAI HTTP."""
    if not agent_id or ha_conversation is None:
        return None
    try:
        agent = ha_conversation.async_get_agent(hass, agent_id)
    except Exception:  # noqa: BLE001 — agent lookup is a system boundary
        return None
    if agent is None:
        return None
    entry = getattr(agent, "entry", None) or getattr(agent, "_entry", None)
    client = _openai_client(agent, entry)
    model = _pick(_MODEL_KEYS, getattr(agent, "model", None), getattr(agent, "subentry", None), getattr(entry, "options", None), getattr(entry, "data", None))
    base = _normalize_base(getattr(client, "base_url", None)) or _normalize_base(getattr(client, "_base_url", None))
    if not base:
        base = _pick(_BASE_KEYS, getattr(agent, "options", None), getattr(entry, "options", None), getattr(entry, "data", None))
        base = _normalize_base(base)
    key = str(getattr(client, "api_key", None) or getattr(client, "_api_key", None) or "").strip()
    if not key:
        key = _pick(_KEY_KEYS, getattr(agent, "options", None), getattr(entry, "options", None), getattr(entry, "data", None))
    if not model:
        return None
    if not base:
        if not key:
            return None
        base = _DEFAULT_BASE
    return {"base_url": base, "api_key": key, "model": model}


def _openai_client(agent: Any, entry: Any) -> Any:
    runtime = getattr(entry, "runtime_data", None)
    for raw in (
        runtime,
        getattr(runtime, "client", None) if runtime is not None else None,
        getattr(agent, "client", None),
        getattr(agent, "_client", None),
        getattr(agent, "openai", None),
        getattr(getattr(agent, "coordinator", None), "client", None),
    ):
        if raw is not None and hasattr(raw, "chat"):
            return raw
        inner = getattr(raw, "client", None)
        if inner is not None and hasattr(inner, "chat"):
            return inner
    return None


def _mapping(source: Any) -> dict[str, Any]:
    if source is None:
        return {}
    if isinstance(source, dict):
        return source
    data = getattr(source, "data", None)
    if isinstance(data, dict):
        return data
    try:
        return dict(source)
    except (TypeError, ValueError):
        return {}


def _pick(keys: tuple[str, ...], *sources: Any) -> str:
    for source in sources:
        if isinstance(source, str) and source.strip() and "model" in keys:
            return source.strip()
        data = _mapping(source)
        for key in keys:
            value = str(data.get(key) or "").strip()
            if value:
                return value
        named = getattr(source, "model", None)
        if "model" in keys and isinstance(named, str) and named.strip():
            return named.strip()
    return ""


def _normalize_base(raw: Any) -> str:
    text = str(raw or "").strip().rstrip("/")
    if not text:
        return ""
    parsed = urlparse(text)
    if parsed.scheme not in {"http", "https"} or parsed.username or parsed.password:
        return ""
    if not parsed.path or parsed.path == "/":
        return f"{text}/v1"
    return text
