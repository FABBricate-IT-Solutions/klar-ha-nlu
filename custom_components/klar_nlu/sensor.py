from __future__ import annotations

from homeassistant.components.sensor import SensorEntity
from homeassistant.config_entries import ConfigEntry
from homeassistant.core import Event, HomeAssistant, callback
from homeassistant.helpers.device_registry import DeviceInfo
from homeassistant.helpers.entity_platform import AddEntitiesCallback

from .const import DOMAIN

EVENT_LAST_TURN = f"{DOMAIN}_last_turn"

_KINDS = (
    ("last_heard", "mdi:ear-hearing"),
    ("last_decision", "mdi:source-branch"),
    ("last_speech", "mdi:message-reply-text"),
    ("last_area", "mdi:map-marker"),
)


def remember_turn(
    hass: HomeAssistant,
    entry_id: str,
    text: str,
    speech: str,
    decision: str,
    area: str | None,
) -> None:
    stored = (hass.data.get(DOMAIN) or {}).get(entry_id)
    if stored is None:
        return
    stored["last_turn"] = {
        "last_heard": (text or "")[:255],
        "last_decision": (decision or "")[:64],
        "last_speech": (speech or "")[:255],
        "last_area": (area or "")[:128],
    }
    hass.bus.async_fire(EVENT_LAST_TURN, {"entry_id": entry_id})


async def async_setup_entry(
    hass: HomeAssistant,
    entry: ConfigEntry,
    async_add_entities: AddEntitiesCallback,
) -> None:
    async_add_entities([KlarTurnSensor(entry, kind, icon) for kind, icon in _KINDS])


class KlarTurnSensor(SensorEntity):
    _attr_has_entity_name = True
    _attr_should_poll = False

    def __init__(self, entry: ConfigEntry, kind: str, icon: str) -> None:
        self._entry = entry
        self._kind = kind
        self._attr_unique_id = f"{entry.entry_id}_{kind}"
        self._attr_translation_key = kind
        self._attr_icon = icon
        self._attr_device_info = DeviceInfo(
            identifiers={(DOMAIN, entry.entry_id)},
            name="Klar NLU",
            manufacturer="FABBricate IT Solutions",
        )

    @property
    def extra_state_attributes(self) -> dict[str, str]:
        return {"klar_kind": self._kind}

    @property
    def native_value(self) -> str | None:
        stored = (self.hass.data.get(DOMAIN) or {}).get(self._entry.entry_id) or {}
        turn = stored.get("last_turn") or {}
        value = turn.get(self._kind)
        return value or None

    async def async_added_to_hass(self) -> None:
        await super().async_added_to_hass()

        @callback
        def _updated(event: Event) -> None:
            if event.data.get("entry_id") == self._entry.entry_id:
                self.async_write_ha_state()

        self.async_on_remove(self.hass.bus.async_listen(EVENT_LAST_TURN, _updated))
