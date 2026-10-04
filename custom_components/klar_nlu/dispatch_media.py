"""Music Assistant, Alexa, and media-player helpers for intent dispatch."""

from __future__ import annotations

import logging
from collections.abc import Callable
from typing import Any

from homeassistant.core import HomeAssistant
from homeassistant.helpers import entity_registry

from .dispatch_result import IntentStepResult, fail, ok
from .speech_render import spoken_after_execute, try_engine_speech
from .speech_snapshot import entity_from_state

_LOGGER = logging.getLogger(__name__)

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
    exposed: Callable[[str], bool],
) -> IntentStepResult:
    entity_id = str(slots.get("entity_id", {}).get("value") or "")
    state = hass.states.get(entity_id) if entity_id.startswith("media_player.") else None
    if state is None or not exposed(entity_id) or media_missing(state):
        return fail("mass_target_unavailable")
    if name == "MassFavorite":
        button = favorite_button(hass, entity_id)
        if not button:
            return fail("favorite_button_missing")
        try:
            await hass.services.async_call("button", "press", {"entity_id": button}, blocking=True)
        except Exception as err:  # noqa: BLE001 — HA services are a boundary
            _LOGGER.debug("Favorit für %s nicht gesetzt: %s", entity_id, err)
            return fail(str(err) or "favorite_failed")
        extra = [row] if (row := entity_from_state(state)) else None
        return ok(await spoken_after_execute(hass, pack, "default", {**item, "name": name}, extra_entities=extra))
    try:
        if name == "MassGetQueue":
            response = await call_with_response(hass, "music_assistant", "get_queue", {}, {"entity_id": entity_id})
            extra = [row] if (row := entity_from_state(state)) else None
            queue = [{"title": str(item.get("name") or item.get("title") or "")} for item in _queue_rows(response)]
            spoken = await try_engine_speech(
                hass,
                pack,
                "default",
                {**item, "name": name},
                extra_entities=extra,
                media_queue=queue,
            )
            return ok(spoken)
        if name == "MassTransferQueue":
            data = clean_service_data(slots, ["source_player", "auto_play"])
            source_player = str(data.get("source_player") or "")
            source_state = (
                hass.states.get(source_player)
                if source_player.startswith("media_player.")
                else None
            )
            if (
                source_state is None
                or source_player == entity_id
                or not exposed(source_player)
                or media_missing(source_state)
            ):
                return fail("transfer_source_unavailable")
            await hass.services.async_call(
                "music_assistant", "transfer_queue", data, blocking=True, target={"entity_id": entity_id}
            )
        elif name == "MassPlayMedia":
            data = clean_service_data(
                slots, ["media_id", "media_type", "artist", "album", "enqueue", "radio_mode", "username"]
            )
            if "radio_mode" in data:
                data["radio_mode"] = str(data["radio_mode"]).lower() == "true"
            await play_mass_media(hass, entity_id, data)
            state = hass.states.get(entity_id) or state
            query = str(data.get("media_id") or "")
            if not media_query_matches(query, state):
                return fail("mass_wrong_match")
        else:
            return fail("unsupported_mass_intent")
    except Exception as err:  # noqa: BLE001 — Music Assistant is a service boundary
        _LOGGER.debug("Music Assistant Intent %s fehlgeschlagen: %s", name, err)
        return fail(str(err) or "mass_failed")
    extra = [row] if (row := entity_from_state(state)) else None
    return ok(await spoken_after_execute(hass, pack, "default", {**item, "name": name}, extra_entities=extra))


async def call_with_response(
    hass: HomeAssistant,
    domain: str,
    service: str,
    data: dict[str, Any],
    target: dict[str, Any],
) -> Any:
    try:
        return await hass.services.async_call(
            domain, service, data, blocking=True, target=target, return_response=True
        )
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


def media_missing(state: Any) -> bool:
    return str(getattr(state, "state", "")).lower() in {"unavailable", "unknown"}


_MEDIA_STOP = frozenset(
    {
        "a",
        "an",
        "and",
        "das",
        "dem",
        "den",
        "der",
        "des",
        "die",
        "ein",
        "eine",
        "einem",
        "einen",
        "einer",
        "or",
        "the",
        "und",
    }
)
_GENERIC_MUSIC = frozenset({"musik", "music", "lied", "song", "track", "titel", "radio"})


async def play_mass_media(hass: HomeAssistant, entity_id: str, data: dict[str, Any]) -> None:
    """Play on Music Assistant; retry without media_type when typed search fails."""
    try:
        await hass.services.async_call(
            "music_assistant", "play_media", data, blocking=True, target={"entity_id": entity_id}
        )
        return
    except Exception as err:  # noqa: BLE001 — Music Assistant is a service boundary
        if "media_type" not in data:
            raise
        _LOGGER.debug("Music Assistant typed play failed for %s (%s); retry without media_type", entity_id, err)
    retry = {key: value for key, value in data.items() if key != "media_type"}
    await hass.services.async_call(
        "music_assistant", "play_media", retry, blocking=True, target={"entity_id": entity_id}
    )


def media_query_matches(query: str, state: Any) -> bool:
    """Reject fuzzy MASS hits that barely overlap the spoken search."""
    folded = " ".join(query.casefold().split())
    if not folded or folded in _GENERIC_MUSIC or "://" in folded:
        return True
    wanted = _media_tokens(folded)
    if not wanted:
        return True
    attrs = getattr(state, "attributes", None) or {}
    if not isinstance(attrs, dict):
        return True
    hay = " ".join(
        str(attrs.get(key) or "")
        for key in ("media_title", "media_artist", "media_album_name", "media_album_artist")
    ).casefold()
    if not hay.strip():
        return True
    have = _media_tokens(hay)
    if not have:
        return True
    hits = sum(1 for token in wanted if _token_hit(token, have))
    need = max(1, (len(wanted) + 1) // 2)
    return hits >= need


def _media_tokens(text: str) -> set[str]:
    out: set[str] = set()
    for raw in text.replace("(", " ").replace(")", " ").replace("[", " ").replace("]", " ").split():
        token = "".join(ch for ch in raw if ch.isalnum())
        if len(token) >= 2 and token not in _MEDIA_STOP:
            out.add(token)
    return out


def _token_hit(token: str, have: set[str]) -> bool:
    if token in have:
        return True
    if len(token) < 4:
        return False
    return any(token in other or other in token for other in have if len(other) >= 4)


def _queue_rows(response: Any) -> list[dict[str, Any]]:
    if isinstance(response, list):
        return [row for row in response if isinstance(row, dict)]
    if not isinstance(response, dict):
        return []
    for key in ("items", "queue", "queue_items", "media_items"):
        nested = response.get(key)
        if isinstance(nested, list):
            return [row for row in nested if isinstance(row, dict)]
        if isinstance(nested, dict):
            found = _queue_rows(nested)
            if found:
                return found
    return []


async def start_idle_music(
    hass: HomeAssistant,
    entity_id: str,
    pack: str,
    item: dict,
    exposed: Callable[[str], bool],
) -> IntentStepResult | None:
    state = hass.states.get(entity_id) if entity_id.startswith("media_player.") else None
    if state is None or not exposed(entity_id) or media_missing(state):
        return None
    if str(getattr(state, "state", "")).lower() not in {"idle", "off", "on", "standby"}:
        return None
    if music_assistant_player(hass, entity_id):
        query = "Musik" if pack == "de" or pack.startswith("de-") else "music"
        return await run_mass(
            hass,
            "MassPlayMedia",
            {"entity_id": {"value": entity_id}, "media_id": {"value": query}},
            pack,
            {
                **item,
                "name": "MassPlayMedia",
                "slots": [*(item.get("slots") or []), {"name": "media_id", "value": query}],
            },
            exposed,
        )
    device_id = alexa_device_id(hass, entity_id)
    if not device_id:
        return None
    command = "spiel Musik" if pack == "de" or pack.startswith("de-") else "play music"
    try:
        await hass.services.async_call(
            "alexa_devices",
            "send_text_command",
            {"device_id": device_id, "text_command": command},
            blocking=True,
        )
    except Exception as err:  # noqa: BLE001 — Alexa is a service boundary
        _LOGGER.debug("Alexa-Wiedergabe für %s fehlgeschlagen: %s", entity_id, err)
        return fail(str(err) or "alexa_play_failed")
    spoken = {
        **item,
        "name": "HassMediaSearchAndPlay",
        "slots": [*(item.get("slots") or []), {"name": "name", "value": state.name}],
    }
    extra = [row] if (row := entity_from_state(state)) else None
    return ok(await spoken_after_execute(hass, pack, "default", spoken, extra_entities=extra))


def tv_request(text: str) -> bool:
    folded = f" {(text or '').casefold()} "
    return "fernseher" in folded or "television" in folded or " tv " in folded or folded.startswith(" tv")


def tv_named(entity_id: str, state: Any) -> bool:
    attrs = getattr(state, "attributes", None) or {} if state is not None else {}
    name = str(attrs.get("friendly_name") or "") if isinstance(attrs, dict) else ""
    name = name or str(getattr(state, "name", "") or "")
    blob = f"{entity_id} {name}".casefold()
    return "tv" in blob or "fernseher" in blob or "television" in blob


def registry_entry(hass: HomeAssistant, entity_id: str) -> Any:
    try:
        return entity_registry.async_get(hass).async_get(entity_id)
    except Exception:  # noqa: BLE001 — registry is a system boundary
        return None


def alexa_device_id(hass: HomeAssistant, entity_id: str) -> str:
    entry = registry_entry(hass, entity_id)
    if entry is None or getattr(entry, "platform", None) != "alexa_devices":
        return ""
    return str(getattr(entry, "device_id", "") or "")


def music_assistant_player(hass: HomeAssistant, entity_id: str) -> bool:
    if not entity_id.startswith("media_player."):
        return False
    state = hass.states.get(entity_id)
    attrs = getattr(state, "attributes", None) or {}
    if isinstance(attrs, dict) and (
        attrs.get("mass_player_type") or "music assistant" in str(attrs.get("source") or "").lower()
    ):
        return True
    entry = registry_entry(hass, entity_id)
    return bool(entry is not None and getattr(entry, "platform", None) == "music_assistant")


def favorite_button(hass: HomeAssistant, player: str) -> str:
    base = player.split(".", 1)[-1].removesuffix("_2")
    for state in hass.states.async_all("button"):
        entity_id = str(state.entity_id)
        label = f"{entity_id} {getattr(state, 'name', '')}".lower()
        if ("favorisieren" in label or "favorite" in label) and (not base or base in entity_id):
            return entity_id
    return ""
