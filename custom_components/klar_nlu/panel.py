from __future__ import annotations

from pathlib import Path

from homeassistant.components.frontend import add_extra_js_url
from homeassistant.core import HomeAssistant

from .const import DOMAIN

_WWW = Path(__file__).parent / "www"
_CARD = "/klar_nlu/klar-home-card.js"


async def async_setup_panel(hass: HomeAssistant) -> None:
    if hass.data.get(DOMAIN, {}).get("panel"):
        return
    hass.data.setdefault(DOMAIN, {})["panel"] = True
    try:
        from homeassistant.components.http import StaticPathConfig

        await hass.http.async_register_static_paths(
            [StaticPathConfig("/klar_nlu", str(_WWW), False)]
        )
    except (AttributeError, TypeError, ImportError):
        hass.http.register_static_path("/klar_nlu", str(_WWW), cache_headers=False)
    add_extra_js_url(hass, _CARD)
    await _async_register_dashboard(hass)


async def _async_register_dashboard(hass: HomeAssistant) -> None:
    """Sidebar Lovelace view with klar-home-card so a family sees it on first run."""
    try:
        from homeassistant.components.frontend import async_register_built_in_panel
        from homeassistant.components.lovelace.dashboard import LovelaceStorage
    except ImportError:
        return
    data = hass.data.get("lovelace")
    if data is None:
        return
    url_path = "klar-nlu"
    dashboards = getattr(data, "dashboards", None)
    if not isinstance(dashboards, dict) or url_path in dashboards:
        return
    config = {
        "mode": "storage",
        "icon": "mdi:waveform",
        "title": "Klar",
        "url_path": url_path,
        "show_in_sidebar": True,
        "require_admin": False,
    }
    try:
        dash = LovelaceStorage(hass, config)
        await dash.async_load(False)
        current = getattr(dash, "config", None) or {}
        if not current.get("views"):
            await dash.async_save(
                {
                    "title": "Klar",
                    "views": [
                        {
                            "title": "Klar",
                            "path": "klar",
                            "cards": [{"type": "custom:klar-home-card"}],
                        }
                    ],
                }
            )
        dashboards[url_path] = dash
        async_register_built_in_panel(
            hass,
            component_name="lovelace",
            sidebar_title="Klar",
            sidebar_icon="mdi:waveform",
            frontend_url_path=url_path,
            config={"mode": "storage"},
            require_admin=False,
        )
    except (AttributeError, TypeError, ValueError, RuntimeError):
        return
