"""Push live Home Assistant registries to the Klar engine."""

from __future__ import annotations

import asyncio
import logging
from typing import Any

from aiohttp import ClientError, ClientTimeout
from homeassistant.components.homeassistant.exposed_entities import async_should_expose
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import Event, HomeAssistant, callback
from homeassistant.helpers import area_registry, device_registry, entity_registry, floor_registry, label_registry
from homeassistant.helpers.aiohttp_client import async_get_clientsession

from .const import CONF_ASSIST_FILTER, DEFAULT_ASSIST_FILTER, DEFAULT_URL, CONF_URL, engine_url_candidates

_LOGGER = logging.getLogger(__name__)
_EVENTS = (
    "entity_registry_updated",
    "device_registry_updated",
    "area_registry_updated",
    "floor_registry_updated",
    "label_registry_updated",
    "exposed_entities_updated",
)
_DEBOUNCE_S = 0.6
_PUSH_EVERY_S = 60.0
_RETRY_S = 15.0
_ID_CAP = 128
_NAME_CAP = 256
_ALIAS_CAP = 32
_HOME_SCHEMA = "1"


class HomeGraphSync:
    """Push a versioned home-graph snapshot after setup and registry changes."""

    def __init__(self, hass: HomeAssistant, entry: ConfigEntry, url: str, token: str | None) -> None:
        self.hass = hass
        self._entry = entry
        self._url = url.rstrip("/")
        self._token = token
        self._unsubs: list[Any] = []
        self._debounce: asyncio.TimerHandle | None = None
        self._tick: asyncio.Task[None] | None = None

    async def async_start(self) -> None:
        await self.async_push()
        for event in _EVENTS:
            self._unsubs.append(self.hass.bus.async_listen(event, self._on_change))
        self._tick = self.hass.async_create_task(self._loop())

    async def async_stop(self) -> None:
        if self._debounce is not None:
            self._debounce.cancel()
            self._debounce = None
        if self._tick is not None:
            self._tick.cancel()
            try:
                await self._tick
            except asyncio.CancelledError:
                pass
            self._tick = None
        while self._unsubs:
            self._unsubs.pop()()

    async def _loop(self) -> None:
        delay = _PUSH_EVERY_S
        while True:
            await asyncio.sleep(delay)
            delay = _PUSH_EVERY_S if await self.async_push() else _RETRY_S

    @callback
    def _on_change(self, _event: Event) -> None:
        if self._debounce is not None:
            self._debounce.cancel()
        self._debounce = self.hass.loop.call_later(_DEBOUNCE_S, self._schedule)

    def _schedule(self) -> None:
        self._debounce = None
        self.hass.async_create_task(self.async_push())

    async def async_push(self) -> bool:
        snapshot = self.build_snapshot()
        session = async_get_clientsession(self.hass)
        headers = {"X-Klar-Token": self._token} if self._token else {}
        last_err: Exception | None = None
        for base in engine_url_candidates(self._url):
            try:
                async with session.post(
                    f"{base}/api/v2/home",
                    json=snapshot,
                    headers=headers,
                    timeout=ClientTimeout(total=8),
                ) as resp:
                    if resp.status >= 400:
                        _LOGGER.warning("Klar home snapshot rejected: %s", resp.status)
                        return False
                    return True
            except (ClientError, TimeoutError, OSError) as err:
                last_err = err
                continue
        if last_err is not None:
            _LOGGER.debug("Klar home snapshot not pushed: %s", last_err)
        return False

    def build_snapshot(self) -> dict[str, Any]:
        er = entity_registry.async_get(self.hass)
        dr = device_registry.async_get(self.hass)
        ar = area_registry.async_get(self.hass)
        fr = floor_registry.async_get(self.hass)
        lr = label_registry.async_get(self.hass)
        assist_filter = self._entry.options.get(CONF_ASSIST_FILTER, DEFAULT_ASSIST_FILTER)
        entities = [_entity(entry) for entry in er.entities.values() if not getattr(entry, "disabled_by", None)]
        devices = [_device(device) for device in dr.devices.values()]
        areas = [_area(area) for area in ar.areas.values()]
        floors = [_floor(floor) for floor in fr.floors.values()]
        labels = [_label(label) for label in lr.labels.values()]
        assist = None
        if assist_filter:
            assist = [
                entry.entity_id
                for entry in er.entities.values()
                if not getattr(entry, "disabled_by", None) and _exposed(self.hass, entry.entity_id)
            ]
        return {
            "schema_version": _HOME_SCHEMA,
            "entities": entities,
            "devices": devices,
            "areas": areas,
            "floors": floors,
            "labels": labels,
            "assist": assist,
            "registered_intents": _registered_intents(self.hass),
        }


def _registered_intents(hass: HomeAssistant) -> list[str]:
    try:
        from .intents import registered_intent_names

        return sorted(registered_intent_names(hass))[:64]
    except Exception:  # noqa: BLE001 — intent registry is a system boundary
        return []


def _exposed(hass: HomeAssistant, entity_id: str) -> bool:
    try:
        return bool(async_should_expose(hass, "conversation", entity_id))
    except Exception:  # noqa: BLE001 — expose store is a system boundary
        return False


def _entity(entry: Any) -> dict[str, Any]:
    return {
        "entity_id": _cap(getattr(entry, "entity_id", ""), _ID_CAP),
        "name": _opt(getattr(entry, "name", None), _NAME_CAP),
        "original_name": _opt(getattr(entry, "original_name", None), _NAME_CAP),
        "has_entity_name": bool(getattr(entry, "has_entity_name", False)),
        "area_id": _opt(getattr(entry, "area_id", None), _ID_CAP),
        "device_id": _opt(getattr(entry, "device_id", None), _ID_CAP),
        "platform": _opt(getattr(entry, "platform", None), _ID_CAP),
        "aliases": _aliases(getattr(entry, "aliases", None)),
        "labels": _aliases(getattr(entry, "labels", None)),
        "disabled": False,
    }


def _device(device: Any) -> dict[str, Any]:
    return {
        "id": _cap(getattr(device, "id", ""), _ID_CAP),
        "name": _opt(getattr(device, "name", None), _NAME_CAP),
        "name_by_user": _opt(getattr(device, "name_by_user", None), _NAME_CAP),
        "area_id": _opt(getattr(device, "area_id", None), _ID_CAP),
    }


def _area(area: Any) -> dict[str, Any]:
    return {
        "id": _cap(getattr(area, "id", None) or getattr(area, "area_id", ""), _ID_CAP),
        "name": _cap(getattr(area, "name", ""), _NAME_CAP),
        "aliases": _aliases(getattr(area, "aliases", None)),
        "floor_id": _opt(getattr(area, "floor_id", None), _ID_CAP),
    }


def _floor(floor: Any) -> dict[str, Any]:
    return {
        "floor_id": _cap(getattr(floor, "floor_id", None) or getattr(floor, "id", ""), _ID_CAP),
        "name": _cap(getattr(floor, "name", ""), _NAME_CAP),
        "aliases": _aliases(getattr(floor, "aliases", None)),
        "level": getattr(floor, "level", None),
    }


def _label(label: Any) -> dict[str, Any]:
    return {
        "label_id": _cap(getattr(label, "label_id", None) or getattr(label, "id", ""), _ID_CAP),
        "name": _cap(getattr(label, "name", ""), _NAME_CAP),
    }


def _aliases(values: Any) -> list[str]:
    if not values:
        return []
    out: list[str] = []
    for value in list(values)[:_ALIAS_CAP]:
        text = _opt(value, _NAME_CAP)
        if text:
            out.append(text)
    return out


def _cap(value: Any, maximum: int) -> str:
    text = str(value or "")
    return text[:maximum]


def _opt(value: Any, maximum: int) -> str | None:
    if value in (None, ""):
        return None
    return _cap(value, maximum)


def engine_url(entry: ConfigEntry) -> str:
    return str(entry.options.get(CONF_URL) or entry.data.get(CONF_URL) or DEFAULT_URL)
