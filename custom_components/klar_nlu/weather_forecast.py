"""HA forecasts for weather speech. Fail soft."""

from __future__ import annotations

import logging
from typing import Any

_LOGGER = logging.getLogger(__name__)
MAX_DAYS = 8
MAX_HOURS = 48


def _num(value: Any) -> float | None:
    if isinstance(value, bool) or value is None:
        return None
    if isinstance(value, (int, float)):
        return float(value)
    try:
        return float(str(value))
    except ValueError:
        return None


def _row(row: dict[str, Any]) -> dict[str, Any]:
    return {
        "datetime": str(row.get("datetime") or "")[:64],
        "condition": str(row.get("condition") or "")[:64],
        "temperature": _num(row.get("temperature")),
        "templow": _num(row.get("templow")),
        "precipitation": _num(row.get("precipitation")),
        "precipitation_probability": _num(row.get("precipitation_probability")),
    }


async def _kind(hass: Any, entity_id: str, kind: str, limit: int) -> list[dict[str, Any]]:
    services = getattr(hass, "services", None)
    call = getattr(services, "async_call", None)
    if not callable(call):
        return []
    try:
        response = await call(
            "weather",
            "get_forecasts",
            {"type": kind, "entity_id": entity_id},
            blocking=True,
            return_response=True,
        )
    except Exception as err:  # noqa: BLE001 — weather service is a boundary
        _LOGGER.debug("weather.get_forecasts %s skipped: %s", kind, err)
        return []
    payload = response.get(entity_id) if isinstance(response, dict) else None
    days = payload.get("forecast") if isinstance(payload, dict) else None
    if not isinstance(days, list):
        return []
    return [_row(row) for row in days[:limit] if isinstance(row, dict)]


async def daily_forecasts(hass: Any, entity_id: str) -> list[dict[str, Any]]:
    daily, _hourly = await forecasts(hass, entity_id)
    return daily


async def forecasts(hass: Any, entity_id: str) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    if not entity_id.startswith("weather."):
        return [], []
    return await _kind(hass, entity_id, "daily", MAX_DAYS), await _kind(hass, entity_id, "hourly", MAX_HOURS)
