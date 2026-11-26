"""Capture live HA state before a value intent runs and stamp it on the engine session."""

from __future__ import annotations

import logging
from typing import Any

import aiohttp
from homeassistant.core import HomeAssistant
from homeassistant.helpers.aiohttp_client import async_get_clientsession

from .const import engine_headers, engine_url_candidates

_LOGGER = logging.getLogger(__name__)

VALUE_INTENTS = frozenset(
    {
        "HassLightSet",
        "HassSetPosition",
        "HassClimateSetTemperature",
        "HassSetVolume",
        "HassSetVolumeRelative",
        "HassFanSetSpeed",
    }
)


def capture_priors(hass: HomeAssistant, intents: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Read the attributes undo will restore, before the forward action changes them."""
    out: list[dict[str, Any]] = []
    seen: set[str] = set()
    for item in intents:
        name = str(item.get("name") or "")
        if name not in VALUE_INTENTS:
            continue
        entity_id = _entity_id(item)
        if not entity_id or entity_id in seen:
            continue
        state = hass.states.get(entity_id)
        if state is None:
            continue
        prior = _prior_from_state(name, entity_id, state)
        if prior is None:
            continue
        seen.add(entity_id)
        out.append(prior)
    return out


async def stamp_then_execute(
    hass: HomeAssistant,
    user_input: Any,
    intents: list[dict[str, Any]],
    pack: str,
    assistant: str | None,
    exposed: Any,
    *,
    url: str,
    token: str | None,
    conversation_id: str | None,
    execute_plan: Any,
) -> dict[str, Any]:
    """Stamp priors on the engine session, then run the plan."""
    priors = capture_priors(hass, intents)
    await post_priors(
        hass,
        url=url,
        token=token,
        conversation_id=conversation_id,
        priors=priors,
    )
    return await execute_plan(hass, user_input, intents, pack, assistant, exposed)


async def post_priors(
    hass: HomeAssistant,
    *,
    url: str,
    token: str | None,
    conversation_id: str | None,
    priors: list[dict[str, Any]],
) -> None:
    if not priors or not conversation_id or not str(conversation_id).strip():
        return
    headers = engine_headers(token)
    session = async_get_clientsession(hass)
    body = {"conversation_id": str(conversation_id), "priors": priors}
    last_err: Exception | None = None
    for base in engine_url_candidates(url):
        try:
            async with session.post(
                f"{base}/api/v2/session/undo-priors",
                json=body,
                headers=headers,
                timeout=aiohttp.ClientTimeout(total=5),
            ) as resp:
                if resp.status < 400:
                    return
                last_err = RuntimeError(f"undo-priors HTTP {resp.status}")
        except (aiohttp.ClientError, TimeoutError, OSError) as err:
            last_err = err
    if last_err is not None:
        _LOGGER.debug("Klar undo priors not stamped: %s", last_err)


def _entity_id(item: dict[str, Any]) -> str:
    for slot in item.get("slots") or []:
        if isinstance(slot, dict) and slot.get("name") == "entity_id":
            return str(slot.get("value") or "").strip()
    return ""


def _prior_from_state(intent_name: str, entity_id: str, state: Any) -> dict[str, Any] | None:
    attrs = getattr(state, "attributes", None) or {}
    if not isinstance(attrs, dict):
        attrs = {}
    prior: dict[str, Any] = {"entity_id": entity_id}
    if intent_name == "HassLightSet":
        bri = _brightness_pct(attrs.get("brightness"))
        if bri is not None:
            prior["brightness"] = bri
        color = attrs.get("color_name") or attrs.get("hs_color")
        if isinstance(color, str) and color.strip():
            prior["color"] = color.strip()[:64]
    elif intent_name == "HassSetPosition":
        position = _pct(attrs.get("current_position"))
        if position is None:
            # Open covers often report 100; closed 0 when current_position is missing.
            ha_state = str(getattr(state, "state", "") or "").casefold()
            if ha_state in {"open", "opening"}:
                position = 100
            elif ha_state in {"closed", "closing"}:
                position = 0
        if position is not None:
            prior["position"] = position
    elif intent_name == "HassClimateSetTemperature":
        temp = attrs.get("temperature")
        if temp is None:
            temp = attrs.get("current_temperature")
        if temp is not None:
            try:
                prior["temperature"] = str(float(temp))
            except (TypeError, ValueError):
                return None
    elif intent_name in {"HassSetVolume", "HassSetVolumeRelative"}:
        level = _volume_pct(attrs.get("volume_level"))
        if level is not None:
            prior["volume_level"] = level
    elif intent_name == "HassFanSetSpeed":
        pct = _pct(attrs.get("percentage"))
        if pct is not None:
            prior["percentage"] = pct
    else:
        return None
    return prior if len(prior) > 1 else None


def _brightness_pct(raw: Any) -> int | None:
    try:
        value = float(raw)
    except (TypeError, ValueError):
        return None
    if value < 0:
        return None
    if value <= 100:
        return max(0, min(100, int(round(value))))
    return max(0, min(100, int(round(value / 255.0 * 100.0))))


def _volume_pct(raw: Any) -> int | None:
    try:
        value = float(raw)
    except (TypeError, ValueError):
        return None
    if 0.0 <= value <= 1.0:
        return max(0, min(100, int(round(value * 100.0))))
    if 0.0 <= value <= 100.0:
        return max(0, min(100, int(round(value))))
    return None


def _pct(raw: Any) -> int | None:
    try:
        value = float(raw)
    except (TypeError, ValueError):
        return None
    if value < 0 or value > 100:
        return None
    return int(round(value))
