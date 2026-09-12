#!/usr/bin/env python3
"""Daily weather forecasts fail soft and keep only spoken fields."""

from __future__ import annotations

import asyncio
import importlib.util
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PKG = ROOT / "custom_components" / "klar_nlu"


def _load():
    path = PKG / "weather_forecast.py"
    spec = importlib.util.spec_from_file_location("weather_forecast", path)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load weather_forecast")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


forecast = _load()


class WeatherForecastTests(unittest.TestCase):
    def test_skips_non_weather(self) -> None:
        async def run() -> list:
            return await forecast.daily_forecasts(object(), "climate.wohnzimmer")

        self.assertEqual(asyncio.run(run()), [])

    def test_maps_daily_response(self) -> None:
        class Services:
            async def async_call(self, domain, service, data, blocking=False, return_response=False):
                self.seen = (domain, service, data, blocking, return_response)
                return {
                    "weather.home": {
                        "forecast": [
                            {
                                "datetime": "2026-09-12T00:00:00+02:00",
                                "condition": "rainy",
                                "temperature": 18,
                                "templow": 11,
                                "precipitation": 2.4,
                                "precipitation_probability": 80,
                                "wind_speed": 12,
                            }
                        ]
                    }
                }

        hass = type("Hass", (), {"services": Services()})()

        async def run() -> list:
            return await forecast.daily_forecasts(hass, "weather.home")

        days = asyncio.run(run())
        self.assertEqual(days[0]["condition"], "rainy")
        self.assertEqual(days[0]["temperature"], 18.0)
        self.assertNotIn("wind_speed", days[0])
        self.assertEqual(hass.services.seen[0], "weather")
        self.assertEqual(hass.services.seen[1], "get_forecasts")


if __name__ == "__main__":
    unittest.main()
