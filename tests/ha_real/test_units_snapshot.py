"""H-05 / H-06: unit system from HA and forecast kinds that actually exist."""

from __future__ import annotations

from engine_fake import FakeEngine, intent
from ha_entities import FakeClimate, add_entities
from homeassistant.components.climate import HVACMode
from homeassistant.const import UnitOfTemperature
from homeassistant.core import HomeAssistant
from homeassistant.util.unit_system import US_CUSTOMARY_SYSTEM
from pytest_homeassistant_custom_component.common import MockConfigEntry
from support import spoken_input

from custom_components.klar_nlu import dispatch, weather_forecast
from custom_components.klar_nlu.engine import async_sync_unit_system
from custom_components.klar_nlu.units import ha_unit_system


async def test_imperial_home_sends_fahrenheit_unchanged(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    # H-05: a US home must not turn 97 °F into 207.
    hass.config.units = US_CUSTOMARY_SYSTEM
    assert ha_unit_system(hass) == "imperial"

    climate = FakeClimate("Hot Tub", "hot-tub", [HVACMode.HEAT, HVACMode.OFF], HVACMode.HEAT)
    climate._attr_temperature_unit = UnitOfTemperature.FAHRENHEIT
    climate._attr_current_temperature = 97
    climate._attr_target_temperature = 100
    await add_entities(hass, "climate", climate)

    result = await dispatch.handle_intent(
        hass,
        spoken_input("what is the hot tub temperature"),
        intent("HassClimateGetTemperature", entity_id=climate.entity_id),
        "en",
        "conversation",
        lambda _entity_id: True,
    )
    assert result.ok, result.error
    assert engine.speech_snapshots, "speech/render was not called"
    snapshot = engine.speech_snapshots[-1]
    assert snapshot["unit_system"] == "imperial"
    entity = snapshot["entities"][0]
    assert entity["attributes"]["current_temperature"] == 97
    assert "F" in str(entity["attributes"].get("temperature_unit") or "")
    # Acceptance: the spoken path keeps 97 (FakeEngine echoes a fixed string, so
    # assert the snapshot the engine would render from).
    assert 97 in (
        entity["attributes"]["current_temperature"],
        entity["attributes"].get("temperature"),
    )


async def test_setup_copies_ha_unit_system_to_the_engine(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    hass.config.units = US_CUSTOMARY_SYSTEM
    # Re-run the seed path the config entry uses after setup.
    await async_sync_unit_system(hass, klar)
    assert engine.settings.get("unit_system") == "imperial"


async def test_twice_daily_forecast_is_used_when_daily_is_missing(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine, monkeypatch
) -> None:
    # H-06: a weather entity with only twice_daily must still yield day rows.
    calls: list[str] = []

    async def fake_kind(_hass, entity_id: str, kind: str, limit: int):
        calls.append(kind)
        if kind == "twice_daily":
            return [
                {
                    "datetime": "2026-09-12T06:00:00+00:00",
                    "condition": "rainy",
                    "temperature": 70.0,
                    "templow": 55.0,
                    "temperature_unit": "°F",
                }
            ]
        return []

    monkeypatch.setattr(weather_forecast, "_kind", fake_kind)
    daily, hourly = await weather_forecast.forecasts(hass, "weather.nws")
    assert calls == ["daily", "twice_daily", "hourly"]
    assert daily[0]["temperature"] == 70.0
    assert daily[0]["temperature_unit"] == "°F"
    assert hourly == []
