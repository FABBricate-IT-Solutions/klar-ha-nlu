"""Home Assistant unit system as Klar snapshot / engine settings data (H-05)."""

from __future__ import annotations

from typing import Any


def ha_unit_system(hass: Any) -> str:
    """Return ``imperial`` or ``metric`` from Home Assistant's configured units."""
    config = getattr(hass, "config", None)
    units = getattr(config, "units", None)
    label = str(getattr(units, "temperature_unit", "") or "").lower().replace(" ", "")
    if "fahrenheit" in label or "°f" in label or label == "f":
        return "imperial"
    return "metric"
