"""Home Assistant AI Task: text or structured JSON via Klar engine chat."""

from __future__ import annotations

import json
import logging
from collections.abc import Mapping
from datetime import timedelta
from typing import Any

try:
    from aiohttp import ClientTimeout
    from homeassistant.components.ai_task import (
        AITaskEntity,
        AITaskEntityFeature,
        GenDataTask,
        GenDataTaskResult,
    )
    from homeassistant.config_entries import ConfigEntry
    from homeassistant.core import HomeAssistant
    from homeassistant.exceptions import HomeAssistantError
    from homeassistant.helpers.aiohttp_client import async_get_clientsession
    from homeassistant.helpers.device_registry import DeviceInfo
    from homeassistant.helpers.entity_platform import AddEntitiesCallback
    from homeassistant.helpers.event import async_track_time_interval
except ImportError:  # stdlib tests load helpers without Home Assistant
    class ClientTimeout:  # type: ignore[no-redef]
        def __init__(self, total: int = 0) -> None:
            self.total = total

    AITaskEntity = object  # type: ignore[misc,assignment]
    AITaskEntityFeature = None  # type: ignore[assignment]
    GenDataTask = Any
    GenDataTaskResult = Any
    ConfigEntry = Any
    HomeAssistant = Any
    HomeAssistantError = RuntimeError  # type: ignore[misc,assignment]
    async_get_clientsession = None  # type: ignore[assignment]
    DeviceInfo = Any
    AddEntitiesCallback = Any
    async_track_time_interval = None  # type: ignore[assignment]

from .const import CONF_TOKEN, CONF_URL, DEFAULT_URL, DOMAIN, engine_headers, engine_url_candidates
from .engine_llm import complete_engine_chat, engine_target

_PROBE_TIMEOUT = ClientTimeout(total=5)

_LOGGER = logging.getLogger(__name__)
_PROBE = timedelta(seconds=60)
_ROLES = frozenset({"system", "user", "assistant"})


def json_object(text: str) -> dict[str, Any] | None:
    raw = _json_object_text(text)
    if raw is None:
        return None
    try:
        data = json.loads(raw)
    except json.JSONDecodeError:
        return None
    return data if isinstance(data, dict) else None


def structure_system(structure: object) -> str:
    return (
        "Reply with a single JSON object that matches this schema. "
        "No markdown, no prose.\n"
        f"{_structure_text(structure)}"
    )


def messages_from_log(
    content: object,
    instructions: str = "",
    structure: object = None,
) -> list[dict[str, str]]:
    messages: list[dict[str, str]] = []
    items = content if isinstance(content, (list, tuple)) else []
    for item in items:
        role, text = _role_text(item)
        if role in _ROLES and text:
            messages.append({"role": role, "content": text})
    if structure is not None:
        messages.insert(0, {"role": "system", "content": structure_system(structure)})
    if instructions.strip() and not any(row["role"] == "user" for row in messages):
        messages.append({"role": "user", "content": instructions.strip()})
    if not messages:
        messages.append({"role": "user", "content": instructions.strip() or "Generate the requested data."})
    return messages


def _json_object_text(text: str) -> str | None:
    trimmed = text.strip()
    body = _fenced(trimmed) or trimmed
    start = body.find("{")
    end = body.rfind("}")
    if start < 0 or end <= start:
        return None
    return body[start : end + 1]


def _fenced(text: str) -> str | None:
    if not text.startswith("```"):
        return None
    rest = text[3:]
    if rest.lower().startswith("json"):
        rest = rest[4:]
    rest = rest.lstrip("\n")
    return rest.split("```", 1)[0].strip()


def _structure_text(structure: object) -> str:
    if structure is None:
        return ""
    if isinstance(structure, str):
        return structure
    if isinstance(structure, Mapping):
        return _dump(structure)
    schema = getattr(structure, "schema", None)
    if isinstance(schema, Mapping):
        return _dump({str(key): str(value) for key, value in schema.items()})
    return str(structure)


def _dump(value: object) -> str:
    try:
        return json.dumps(value, ensure_ascii=False)
    except (TypeError, ValueError):
        return str(value)


def _role_text(item: object) -> tuple[str, str]:
    if isinstance(item, Mapping):
        return str(item.get("role") or ""), _content_text(item.get("content"))
    role = str(getattr(item, "role", "") or "")
    return role, _content_text(getattr(item, "content", None))


def _content_text(raw: object) -> str:
    if isinstance(raw, str):
        return raw.strip()
    if isinstance(raw, list):
        parts: list[str] = []
        for item in raw:
            if isinstance(item, str):
                parts.append(item)
            elif isinstance(item, Mapping) and item.get("type") == "text":
                parts.append(str(item.get("text") or ""))
        return "".join(parts).strip()
    if raw is None:
        return ""
    return str(raw).strip()


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    async_add_entities([KlarAITaskEntity(hass, entry)])


class KlarAITaskEntity(AITaskEntity):
    _attr_has_entity_name = True
    _attr_translation_key = "ai_task"
    _attr_supported_features = (
        AITaskEntityFeature.GENERATE_DATA if AITaskEntityFeature is not None else 0
    )

    def __init__(self, hass: HomeAssistant, entry: ConfigEntry) -> None:
        self.hass = hass
        self._entry = entry
        self._configured = False
        self._attr_unique_id = f"{entry.entry_id}_ai_task"
        self._attr_device_info = DeviceInfo(
            identifiers={(DOMAIN, entry.entry_id)},
            name="Klar NLU",
            manufacturer="FABBricate IT Solutions",
        )

    @property
    def available(self) -> bool:
        return self._configured

    async def async_added_to_hass(self) -> None:
        await self._refresh_available()
        if async_track_time_interval is None:
            return
        self.async_on_remove(async_track_time_interval(self.hass, self._on_interval, _PROBE))

    async def _on_interval(self, _now: object) -> None:
        await self._refresh_available()

    async def _refresh_available(self) -> None:
        ready = await self._endpoint_configured()
        if ready == self._configured:
            return
        self._configured = ready
        self.async_write_ha_state()

    async def _async_generate_data(self, task: GenDataTask, chat_log: Any) -> GenDataTaskResult:
        await self._refresh_available()
        if not self._configured:
            raise HomeAssistantError("Klar has no LLM endpoint configured.")
        instructions = str(getattr(task, "instructions", "") or "")
        structure = getattr(task, "structure", None)
        messages = messages_from_log(getattr(chat_log, "content", None), instructions, structure)
        text = await complete_engine_chat(
            self.hass,
            messages,
            url=self._url,
            token=self._token(),
            max_tokens=768,
            temperature=0.2 if structure is not None else 0.65,
        )
        if not text:
            raise HomeAssistantError("Klar LLM chat returned no text.")
        conversation_id = str(getattr(chat_log, "conversation_id", "") or "")
        if structure is None:
            return GenDataTaskResult(conversation_id=conversation_id, data=text)
        data = json_object(text)
        if data is None:
            raise HomeAssistantError("Klar LLM chat did not return a JSON object.")
        return GenDataTaskResult(conversation_id=conversation_id, data=data)

    @property
    def _url(self) -> str:
        return (
            self._entry.options.get(CONF_URL) or self._entry.data.get(CONF_URL) or DEFAULT_URL
        ).rstrip("/")

    def _token(self) -> str | None:
        stored = (self.hass.data.get(DOMAIN) or {}).get(self._entry.entry_id) or {}
        token = stored.get("token") or self._entry.options.get(CONF_TOKEN) or self._entry.data.get(CONF_TOKEN)
        return str(token) if token else None

    async def _endpoint_configured(self) -> bool:
        target = engine_target(self.hass, self._url, self._token())
        if target is None or async_get_clientsession is None:
            return False
        base, tok = target
        session = async_get_clientsession(self.hass)
        headers = engine_headers(tok, extra={"Accept": "application/json"})
        last_err: Exception | None = None
        for host in engine_url_candidates(base):
            try:
                async with session.get(
                    f"{host}/api/v2/llm/endpoint",
                    headers=headers,
                    timeout=_PROBE_TIMEOUT,
                ) as resp:
                    if resp.status >= 400:
                        continue
                    payload = await resp.json()
            except Exception as err:  # noqa: BLE001 — engine HTTP is a system boundary
                last_err = err
                continue
            if isinstance(payload, Mapping):
                return bool(payload.get("configured"))
        if last_err is not None:
            _LOGGER.debug("Klar AI Task endpoint probe failed: %s", last_err)
        return False
