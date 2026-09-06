"""Call engine post-execute speech. Missing route fails closed (no Python templates)."""

from __future__ import annotations

import logging
from datetime import datetime, timezone
from typing import Any

from .engine_llm import EngineSpeechMissing, complete_engine_speech_render
from .speech_snapshot import build_snapshot, entities_from_handled

_LOGGER = logging.getLogger(__name__)


def _unit_system(hass: Any) -> str:
    stored = getattr(hass, "data", None) or {}
    if not isinstance(stored, dict):
        return "metric"
    domain = stored.get("klar_nlu")
    if not isinstance(domain, dict):
        return "metric"
    for payload in domain.values():
        if not isinstance(payload, dict):
            continue
        settings = payload.get("engine_settings")
        if isinstance(settings, dict) and str(settings.get("unit_system") or "").lower() == "imperial":
            return "imperial"
    return "metric"


def _now() -> str:
    try:
        from homeassistant.util import dt as dt_util

        return dt_util.now().isoformat()
    except ImportError:
        return datetime.now(timezone.utc).isoformat()


async def try_engine_speech(
    hass: Any,
    pack: str,
    personality: str,
    item: dict[str, Any],
    handled: Any = None,
    outcome: str = "success",
    *,
    calendar_events: list[dict[str, Any]] | None = None,
    media_queue: list[dict[str, Any]] | None = None,
    extra_entities: list[dict[str, Any]] | None = None,
    url: str | None = None,
    token: str | None = None,
) -> str | None:
    snapshot = build_snapshot(
        language=pack,
        personality=personality,
        now=_now(),
        intent=item,
        outcome=outcome,
        entities=extra_entities if extra_entities is not None else entities_from_handled(handled, item),
        calendar_events=calendar_events,
        media_queue=media_queue,
        unit_system=_unit_system(hass),
    )
    try:
        return await complete_engine_speech_render(hass, snapshot, url=url, token=token)
    except EngineSpeechMissing:
        _LOGGER.debug("Klar speech render route missing")
        return None
    except Exception as err:  # noqa: BLE001 — engine HTTP is a system boundary
        _LOGGER.debug("Klar speech render skipped: %s", err)
        return None


async def spoken_after_execute(
    hass: Any,
    pack: str,
    personality: str,
    item: dict[str, Any],
    handled: Any = None,
    outcome: str = "success",
    *,
    calendar_events: list[dict[str, Any]] | None = None,
    media_queue: list[dict[str, Any]] | None = None,
    extra_entities: list[dict[str, Any]] | None = None,
    url: str | None = None,
    token: str | None = None,
) -> str | None:
    spoken = await try_engine_speech(
        hass,
        pack,
        personality,
        item,
        handled,
        outcome,
        calendar_events=calendar_events,
        media_queue=media_queue,
        extra_entities=extra_entities,
        url=url,
        token=token,
    )
    return spoken or None
