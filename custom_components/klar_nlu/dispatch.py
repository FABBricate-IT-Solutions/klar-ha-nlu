"""Run a parsed Klar intent through Home Assistant."""

from __future__ import annotations

import logging
from collections.abc import Callable
from typing import Any

from homeassistant.components.conversation import ConversationInput
from homeassistant.core import HomeAssistant
from homeassistant.helpers import intent

from .intents import (
    ENTITY_SERVICES,
    LIST_INTENTS,
    MASS_INTENTS,
    TIMER_INTENTS,
    area_label,
    item_slots,
    list_slots,
    timer_slots,
)
from .speech import from_handled, queue_speech

_LOGGER = logging.getLogger(__name__)


async def handle_intent(
    hass: HomeAssistant,
    user_input: ConversationInput,
    item: dict,
    pack: str,
    assistant: str | None,
    exposed: Callable[[str], bool],
) -> str | None:
    name = item.get("name")
    if not name:
        return None
    slots = item_slots(item)
    if name in MASS_INTENTS:
        return await run_mass(hass, name, slots, pack, item)
    if "entity_id" in slots:
        entity_id = str(slots["entity_id"].get("value") or "")
        spoken = await run_entity(hass, name, entity_id, slots, pack, item, exposed)
        if spoken:
            return spoken
        state = hass.states.get(entity_id)
        if state is not None:
            slots["name"] = {"value": state.name}
        if "." in entity_id:
            slots.setdefault("domain", {"value": entity_id.split(".", 1)[0]})
        slots.pop("area", None)
    if name in LIST_INTENTS:
        name, slots = list_slots(hass, name, slots)
    if name in TIMER_INTENTS:
        slots = timer_slots(slots)
        if name == "HassStartTimer" and not any(key in slots for key in ("hours", "minutes", "seconds")):
            return None
    if name == "HassGetState" and slots.get("device_class", {}).get("value") == "temperature":
        slots.pop("domain", None)
        speech = await invoke_intent(hass, user_input, name, slots, pack, item, assistant)
        if speech:
            return speech
        climate = {key: val for key, val in slots.items() if key != "device_class"}
        return await invoke_intent(hass, user_input, "HassClimateGetTemperature", climate, pack, item, assistant)
    if name == "HassGetState" and "area" in slots and "entity_id" not in slots:
        label = area_label(hass, str(slots["area"].get("value") or ""))
        if label:
            item = {
                **item,
                "slots": [*(item.get("slots") or []), {"name": "area_name", "value": label}],
            }
    return await invoke_intent(hass, user_input, name, slots, pack, item, assistant)


async def invoke_intent(
    hass: HomeAssistant,
    user_input: ConversationInput,
    name: str,
    slots: dict[str, Any],
    pack: str,
    item: dict,
    assistant: str | None,
) -> str | None:
    try:
        handled = await intent.async_handle(
            hass,
            "klar_nlu",
            name,
            slots,
            user_input.text,
            user_input.context,
            user_input.language or pack,
            assistant=assistant,
        )
    except Exception as err:  # noqa: BLE001 — HA intent system is a boundary
        _LOGGER.debug("Intent %s nicht ausgeführt: %s", name, err)
        return None
    return from_handled(handled, pack, {**item, "name": name})


async def run_entity(
    hass: HomeAssistant,
    name: str,
    entity_id: str,
    slots: dict[str, Any],
    pack: str,
    item: dict,
    exposed: Callable[[str], bool],
) -> str | None:
    if "." not in entity_id or hass.states.get(entity_id) is None:
        return None
    if not exposed(entity_id):
        return None
    domain = entity_id.split(".", 1)[0]
    data: dict[str, Any] = {"entity_id": entity_id}
    if name == "HassLightSet" and domain == "light":
        service = "turn_on"
        if (bri := slots.get("brightness", {}).get("value")) is not None:
            try:
                data["brightness_pct"] = max(0, min(100, int(bri)))
            except (TypeError, ValueError):
                pass
        if color := slots.get("color", {}).get("value"):
            data["color_name"] = str(color)
    elif domain == "media_player" and name in MEDIA_SERVICES:
        service = MEDIA_SERVICES[name]
        if name == "HassSetVolume":
            try:
                data["volume_level"] = max(0.0, min(1.0, float(slots["volume_level"]["value"]) / 100.0))
            except (KeyError, TypeError, ValueError):
                return None
        elif name == "HassSetVolumeRelative":
            step = str(slots.get("volume_step", {}).get("value") or "")
            service = "volume_down" if step == "down" else "volume_up"
        elif name in {"HassMediaPlayerMute", "HassMediaPlayerUnmute"}:
            data["is_volume_muted"] = name == "HassMediaPlayerMute"
    else:
        service = ENTITY_SERVICES.get(name)
        if not service:
            return None
    try:
        await hass.services.async_call(domain, service, data, blocking=True)
    except Exception as err:  # noqa: BLE001 — HA services are a boundary
        _LOGGER.debug("Gerät %s nicht geschaltet: %s", entity_id, err)
        return None
    state = hass.states.get(entity_id)
    attrs = getattr(state, "attributes", None) or {}
    pretty = ""
    if isinstance(attrs, dict):
        pretty = str(attrs.get("friendly_name") or "")
    pretty = pretty or str(getattr(state, "name", None) or "")
    spoken = {**item, "name": name, "slots": [*(item.get("slots") or []), {"name": "name", "value": pretty}]}
    return from_handled(None, pack, spoken)


MEDIA_SERVICES = {
    "HassMediaPause": "media_pause",
    "HassMediaUnpause": "media_play",
    "HassMediaNext": "media_next_track",
    "HassMediaPrevious": "media_previous_track",
    "HassMediaPlayerMute": "volume_mute",
    "HassMediaPlayerUnmute": "volume_mute",
    "HassSetVolume": "volume_set",
    "HassSetVolumeRelative": "volume_up",
}


async def run_mass(
    hass: HomeAssistant,
    name: str,
    slots: dict[str, Any],
    pack: str,
    item: dict,
) -> str | None:
    entity_id = str(slots.get("entity_id", {}).get("value") or "")
    if name == "MassFavorite":
        button = favorite_button(hass, entity_id)
        if not button:
            return None
        try:
            await hass.services.async_call("button", "press", {"entity_id": button}, blocking=True)
        except Exception as err:  # noqa: BLE001 — HA services are a boundary
            _LOGGER.debug("Favorit für %s nicht gesetzt: %s", entity_id, err)
            return None
        return from_handled(None, pack, {**item, "name": name})
    if not entity_id:
        return None
    try:
        if name == "MassGetQueue":
            response = await call_with_response(hass, "music_assistant", "get_queue", {}, {"entity_id": entity_id})
            return queue_speech(response, hass.states.get(entity_id), pack)
        if name == "MassTransferQueue":
            data = clean_service_data(slots, ["source_player", "auto_play"])
            await hass.services.async_call("music_assistant", "transfer_queue", data, blocking=True, target={"entity_id": entity_id})
        elif name == "MassPlayMedia":
            data = clean_service_data(slots, ["media_id", "media_type", "artist", "album", "enqueue", "radio_mode", "username"])
            if "radio_mode" in data:
                data["radio_mode"] = str(data["radio_mode"]).lower() == "true"
            await hass.services.async_call("music_assistant", "play_media", data, blocking=True, target={"entity_id": entity_id})
        else:
            return None
    except Exception as err:  # noqa: BLE001 — Music Assistant is a service boundary
        _LOGGER.debug("Music Assistant Intent %s fehlgeschlagen: %s", name, err)
        return None
    return from_handled(None, pack, {**item, "name": name})


async def call_with_response(
    hass: HomeAssistant,
    domain: str,
    service: str,
    data: dict[str, Any],
    target: dict[str, Any],
) -> Any:
    try:
        return await hass.services.async_call(domain, service, data, blocking=True, target=target, return_response=True)
    except TypeError:
        await hass.services.async_call(domain, service, data, blocking=True, target=target)
        return None


def clean_service_data(slots: dict[str, Any], names: list[str]) -> dict[str, Any]:
    data: dict[str, Any] = {}
    for name in names:
        value = slots.get(name, {}).get("value")
        if value not in (None, ""):
            data[name] = value
    return data


def favorite_button(hass: HomeAssistant, player: str) -> str:
    base = player.split(".", 1)[-1].removesuffix("_2")
    for state in hass.states.async_all("button"):
        entity_id = str(state.entity_id)
        label = f"{entity_id} {getattr(state, 'name', '')}".lower()
        if ("favorisieren" in label or "favorite" in label) and (not base or base in entity_id):
            return entity_id
    return ""
