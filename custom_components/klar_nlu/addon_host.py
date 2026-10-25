"""Find the Supervisor hostname for an installed Klar app.

Repository installs get a slug like ``8db2ab02_klar_nlu`` and a hostname
``8db2ab02-klar-nlu``. The integration used to assume ``klar-nlu``.
"""

from __future__ import annotations

import logging
import time
from typing import Any

from .const import (
    discovered_addon_hosts,
    installed_klar_channel,
    remember_addon_hosts,
)

_LOGGER = logging.getLogger(__name__)
_REFRESH_S = 60.0
_fetched_at: float | None = None

try:
    from homeassistant.components.hassio.handler import get_supervisor_client
except ImportError:  # pragma: no cover - only outside Home Assistant
    get_supervisor_client = None  # type: ignore[assignment]


def _on_supervisor(hass: Any) -> bool:
    components = getattr(getattr(hass, "config", None), "components", None) or []
    return "hassio" in components


def _hosts_from_info(rows: list[tuple[str, str]]) -> dict[str, str]:
    found: dict[str, str] = {}
    for slug, hostname in rows:
        channel = installed_klar_channel(slug)
        label = str(hostname or "").strip().split(".")[0]
        if channel and label:
            found[channel] = label
    return found


async def async_installed_klar_hosts(hass: Any) -> dict[str, str]:
    """Return ``{channel: hostname}`` for Klar apps Supervisor has installed."""
    if not _on_supervisor(hass) or get_supervisor_client is None:
        return {}
    try:
        client = get_supervisor_client(hass)
        installed = await client.addons.list()
    except Exception as err:  # noqa: BLE001 — Supervisor is a process boundary
        _LOGGER.debug("Klar addon list unavailable: %s", err)
        return {}
    rows: list[tuple[str, str]] = []
    for addon in installed:
        slug = str(getattr(addon, "slug", "") or "")
        if installed_klar_channel(slug) is None:
            continue
        try:
            info = await client.addons.addon_info(slug)
        except Exception as err:  # noqa: BLE001 — one missing addon should not hide the other
            _LOGGER.debug("Klar addon info unavailable for %s: %s", slug, err)
            continue
        rows.append((slug, str(getattr(info, "hostname", "") or "")))
    return _hosts_from_info(rows)


async def async_refresh_klar_hosts(hass: Any, *, force: bool = False) -> dict[str, str]:
    """Refresh the hostname cache at most once a minute, including an empty result."""
    global _fetched_at
    now = time.monotonic()
    if not force and _fetched_at is not None and now - _fetched_at < _REFRESH_S:
        return discovered_addon_hosts()
    previous = discovered_addon_hosts()
    found = await async_installed_klar_hosts(hass)
    remember_addon_hosts(found)
    _fetched_at = now
    if found and found != previous:
        _LOGGER.info(
            "Klar app hostnames: %s",
            ", ".join(f"{key}={value}" for key, value in sorted(found.items())),
        )
    return discovered_addon_hosts()
