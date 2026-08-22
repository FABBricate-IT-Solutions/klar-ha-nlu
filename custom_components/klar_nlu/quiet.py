"""Quiet replies: skip TTS on simple on/off and play a short chime."""

from __future__ import annotations

import logging
import math
import struct
from typing import Any

try:
    from .const import DOMAIN
except ImportError:  # stdlib tests load this module without a package
    DOMAIN = "klar_nlu"

_LOGGER = logging.getLogger(__name__)

EVENT_ACK = f"{DOMAIN}_ack"
CHIME_PATH = f"/api/{DOMAIN}/chime.wav"
SIMPLE_INTENTS = frozenset({"HassTurnOn", "HassTurnOff"})
BLOCKED_DOMAINS = frozenset(
    {"scene", "script", "cover", "lock", "climate", "fan", "media_player", "vacuum"}
)
SIMPLE_DOMAINS = frozenset({"light", "switch"})
_CHIME_WAV: bytes | None = None


def quiet_ack_applies(executed: dict[str, Any] | None, intents: list[dict[str, Any]] | None) -> bool:
    if not executed or executed.get("outcome") != "success":
        return False
    steps = executed.get("steps") or []
    if len(steps) != 1 or steps[0].get("status") != "success":
        return False
    if len(intents or []) != 1:
        return False
    item = intents[0]
    if str(item.get("name") or "") not in SIMPLE_INTENTS:
        return False
    return _simple_target(item)


def _simple_target(item: dict[str, Any]) -> bool:
    slots = {
        str(slot.get("name")): str(slot.get("value") or "")
        for slot in item.get("slots") or []
        if isinstance(slot, dict) and slot.get("name")
    }
    domain = slots.get("domain") or ""
    entity_id = slots.get("entity_id") or ""
    prefix = entity_id.split(".", 1)[0] if "." in entity_id else ""
    if prefix in BLOCKED_DOMAINS or domain in BLOCKED_DOMAINS:
        return False
    if domain in SIMPLE_DOMAINS or prefix in SIMPLE_DOMAINS:
        return True
    if slots.get("area") or slots.get("floor"):
        return domain in {"", "light", "switch"}
    return False


def chime_wav() -> bytes:
    global _CHIME_WAV
    if _CHIME_WAV is None:
        _CHIME_WAV = _render_chime()
    return _CHIME_WAV


def _render_chime() -> bytes:
    rate = 16000
    count = int(rate * 0.14)
    samples = bytearray()
    half = count // 2
    for index in range(count):
        freq = 880.0 if index < half else 1175.0
        ramp = min(1.0, index / 180.0, (count - index) / 360.0)
        value = int(14000 * ramp * math.sin(2.0 * math.pi * freq * index / rate))
        samples.extend(struct.pack("<h", max(-32767, min(32767, value))))
    data = bytes(samples)
    header = struct.pack(
        "<4sI4s4sIHHIIHH4sI",
        b"RIFF",
        36 + len(data),
        b"WAVE",
        b"fmt ",
        16,
        1,
        1,
        rate,
        rate * 2,
        2,
        16,
        b"data",
        len(data),
    )
    return header + data


async def async_setup_chime(hass: Any) -> None:
    stored = hass.data.setdefault(DOMAIN, {})
    if stored.get("chime_view"):
        return
    try:
        from homeassistant.components.http import HomeAssistantView
    except ImportError:
        return

    class KlarChimeView(HomeAssistantView):
        url = CHIME_PATH
        name = f"api:{DOMAIN}:chime"
        requires_auth = False

        async def get(self, request: Any) -> Any:
            from aiohttp import web

            return web.Response(body=chime_wav(), content_type="audio/wav")

    hass.http.register_view(KlarChimeView())
    stored["chime_view"] = True


async def play_chime(hass: Any, user_input: Any) -> None:
    satellite_id = str(getattr(user_input, "satellite_id", None) or "")
    device_id = str(getattr(user_input, "device_id", None) or "")
    if await _esphome_ack(hass, satellite_id):
        return
    media_id = _chime_url(hass)
    if satellite_id and "." in satellite_id:
        if await _announce(hass, satellite_id, media_id):
            return
    for entity_id in _sibling_entities(hass, satellite_id, device_id, "assist_satellite"):
        if await _announce(hass, entity_id, media_id):
            return
    for entity_id in _sibling_entities(hass, satellite_id, device_id, "media_player"):
        if await _play_media(hass, entity_id, media_id):
            return
    hass.bus.async_fire(EVENT_ACK, {"satellite_id": satellite_id, "device_id": device_id})


def _chime_url(hass: Any) -> str:
    try:
        from homeassistant.helpers.network import get_url

        return f"{get_url(hass, prefer_external=False)}{CHIME_PATH}"
    except Exception:  # noqa: BLE001 — URL helper is a system boundary
        base = str(getattr(getattr(hass, "config", None), "internal_url", None) or "").rstrip("/")
        return f"{base}{CHIME_PATH}" if base else CHIME_PATH


async def _esphome_ack(hass: Any, satellite_id: str) -> bool:
    slug = satellite_id.split(".", 1)[-1] if "." in satellite_id else satellite_id
    if not slug:
        return False
    name = f"{slug}_play_ack"
    try:
        if not hass.services.has_service("esphome", name):
            return False
        await hass.services.async_call("esphome", name, {}, blocking=False)
        return True
    except Exception as err:  # noqa: BLE001 — device service is optional
        _LOGGER.debug("Quiet ack via esphome.%s failed: %s", name, err)
        return False


async def _announce(hass: Any, entity_id: str, media_id: str) -> bool:
    data: dict[str, Any] = {"preannounce": False}
    if media_id:
        data["media_id"] = media_id
    else:
        data["preannounce"] = True
    return await _call(hass, "assist_satellite", "announce", data, entity_id)


async def _play_media(hass: Any, entity_id: str, media_id: str) -> bool:
    if not media_id:
        return False
    return await _call(
        hass,
        "media_player",
        "play_media",
        {
            "media_content_id": media_id,
            "media_content_type": "music",
            "announce": True,
        },
        entity_id,
    )


async def _call(hass: Any, domain: str, service: str, data: dict[str, Any], entity_id: str) -> bool:
    try:
        await hass.services.async_call(
            domain,
            service,
            data,
            target={"entity_id": entity_id},
            blocking=False,
        )
        return True
    except Exception as err:  # noqa: BLE001 — satellite audio is optional
        _LOGGER.debug("Quiet ack via %s.%s failed: %s", domain, service, err)
        return False


def _sibling_entities(hass: Any, satellite_id: str, device_id: str, domain: str) -> list[str]:
    found: list[str] = []
    try:
        from homeassistant.helpers import device_registry, entity_registry
    except ImportError:
        return found
    registry = entity_registry.async_get(hass)
    devices = device_registry.async_get(hass)
    wanted = {item for item in (device_id,) if item}
    for candidate in (satellite_id,):
        entity = registry.async_get(candidate) if candidate else None
        if entity and entity.device_id:
            wanted.add(entity.device_id)
    if not wanted:
        return found
    for entity in registry.entities.values():
        if entity.device_id in wanted and entity.entity_id.startswith(f"{domain}."):
            if entity.entity_id not in found:
                found.append(entity.entity_id)
    del devices
    return found
