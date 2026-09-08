from __future__ import annotations

from pathlib import Path

from homeassistant.components.frontend import add_extra_js_url, async_register_built_in_panel
from homeassistant.core import HomeAssistant

from .const import DOMAIN
from .ui_proxy import addon_ingress_active

try:
    from homeassistant.components.frontend import async_remove_panel
except ImportError:

    def async_remove_panel(hass: HomeAssistant, frontend_url_path: str) -> None:
        panels = hass.data.get("frontend_panels")
        if isinstance(panels, dict):
            panels.pop(frontend_url_path, None)

try:
    from homeassistant.components.http import StaticPathConfig
except ImportError:
    StaticPathConfig = None

_WWW = Path(__file__).parent / "www"
_CARD = "/klar_nlu/klar-home-card.js"
_PANEL_JS = "/klar_nlu/panel.js"
PANEL_PATH = "klar-nlu"
PANEL_TITLE = "Klar NLU"
PANEL_ICON = "mdi:brain"
PANEL_ELEMENT = "klar-nlu-panel"


def _dashboard_config(url_path: str) -> dict[str, object]:
    return {
        "id": url_path,
        "mode": "storage",
        "icon": "mdi:waveform",
        "title": "Klar",
        "url_path": url_path,
        "show_in_sidebar": True,
        "require_admin": False,
    }


def operator_panel_config() -> dict[str, object]:
    return {
        "_panel_custom": {
            "name": PANEL_ELEMENT,
            "embed_iframe": False,
            "trust_external": False,
            "js_url": _PANEL_JS,
        }
    }


async def async_setup_panel(hass: HomeAssistant) -> None:
    await _async_register_card(hass)
    if addon_ingress_active(hass):
        _remove_operator_panel(hass)
        return
    _register_operator_panel(hass)


async def async_unload_panel(hass: HomeAssistant) -> None:
    remaining = [
        key
        for key, payload in (hass.data.get(DOMAIN) or {}).items()
        if key not in {"panel", "ui_proxy", "ui_cookie_secret", "operator_panel"}
        and isinstance(payload, dict)
    ]
    if remaining:
        return
    _remove_operator_panel(hass)


async def _async_register_card(hass: HomeAssistant) -> None:
    if hass.data.get(DOMAIN, {}).get("panel"):
        return
    hass.data.setdefault(DOMAIN, {})["panel"] = True
    if StaticPathConfig is not None:
        try:
            await hass.http.async_register_static_paths(
                [StaticPathConfig("/klar_nlu", str(_WWW), False)]
            )
        except (AttributeError, TypeError):
            hass.http.register_static_path("/klar_nlu", str(_WWW), cache_headers=False)
    else:
        hass.http.register_static_path("/klar_nlu", str(_WWW), cache_headers=False)
    add_extra_js_url(hass, _CARD)


def _register_operator_panel(hass: HomeAssistant) -> None:
    if hass.data.get(DOMAIN, {}).get("operator_panel"):
        return
    try:
        async_register_built_in_panel(
            hass,
            component_name="custom",
            sidebar_title=PANEL_TITLE,
            sidebar_icon=PANEL_ICON,
            frontend_url_path=PANEL_PATH,
            config=operator_panel_config(),
            require_admin=True,
            update=True,
        )
    except TypeError:
        try:
            async_register_built_in_panel(
                hass,
                component_name="custom",
                sidebar_title=PANEL_TITLE,
                sidebar_icon=PANEL_ICON,
                frontend_url_path=PANEL_PATH,
                config=operator_panel_config(),
                require_admin=True,
            )
        except ValueError:
            return
    except ValueError:
        return
    hass.data.setdefault(DOMAIN, {})["operator_panel"] = True


def _remove_operator_panel(hass: HomeAssistant) -> None:
    store = hass.data.get(DOMAIN)
    if isinstance(store, dict):
        store.pop("operator_panel", None)
    try:
        async_remove_panel(hass, PANEL_PATH)
    except (AttributeError, KeyError, ValueError):
        return
