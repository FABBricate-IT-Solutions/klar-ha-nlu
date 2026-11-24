"""Fixtures for the tier that runs Klar inside a real Home Assistant core.

Run with the pinned test package (see ``requirements.txt`` next to this file):

    pytest tests/ha_real

The stub tests in ``tests/ha`` stay on plain ``unittest``; this folder has no
``__init__.py`` so ``unittest discover`` does not try to import it.
"""

from __future__ import annotations

from collections.abc import AsyncGenerator, Generator
from typing import Any
from unittest.mock import patch

import pytest
from calendars import MemoryStore
from engine_fake import ENGINE_URL, TIME_ZONE, FakeEngine
from homeassistant.config_entries import ConfigEntryState
from homeassistant.core import HomeAssistant, ServiceRegistry
from homeassistant.setup import async_setup_component
from pytest_homeassistant_custom_component.common import MockConfigEntry
from pytest_homeassistant_custom_component.test_util.aiohttp import AiohttpClientMocker
from support import ServiceLog

from custom_components.klar_nlu import calendar_session
from custom_components.klar_nlu.const import CONF_MODE, CONF_URL, DOMAIN, MODE_REMOTE


@pytest.fixture(autouse=True)
def _custom_integrations(enable_custom_integrations: None) -> None:
    """Let Home Assistant load ``custom_components/klar_nlu`` from the repository."""


@pytest.fixture(autouse=True)
def _forget_listed_events() -> Generator[None]:
    """The events Klar last listed live in a module-level store; keep tests apart."""
    calendar_session._STORE.clear()
    yield
    calendar_session._STORE.clear()


@pytest.fixture
def engine(aioclient_mock: AiohttpClientMocker) -> FakeEngine:
    fake = FakeEngine()
    fake.install(aioclient_mock)
    return fake


@pytest.fixture
async def klar_entry(hass: HomeAssistant) -> MockConfigEntry:
    """A Klar config entry that is added but not set up yet."""
    await hass.config.async_set_time_zone(TIME_ZONE)
    assert await async_setup_component(hass, "homeassistant", {})
    entry = MockConfigEntry(
        domain=DOMAIN,
        title="Klar NLU",
        data={CONF_MODE: MODE_REMOTE, CONF_URL: ENGINE_URL},
        unique_id=DOMAIN,
    )
    entry.add_to_hass(hass)
    return entry


@pytest.fixture
async def klar(
    hass: HomeAssistant, engine: FakeEngine, klar_entry: MockConfigEntry
) -> AsyncGenerator[MockConfigEntry]:
    """A loaded Klar config entry talking to the scripted engine."""
    assert await hass.config_entries.async_setup(klar_entry.entry_id)
    await hass.async_block_till_done()
    assert klar_entry.state is ConfigEntryState.LOADED
    yield klar_entry
    if klar_entry.state is ConfigEntryState.LOADED:
        await hass.config_entries.async_unload(klar_entry.entry_id)
        await hass.async_block_till_done()


@pytest.fixture
def service_log() -> Generator[ServiceLog]:
    """Record calls into the real service registry without changing what they do."""
    log = ServiceLog()
    real_call = ServiceRegistry.async_call

    async def recording_call(
        registry: ServiceRegistry, domain: str, service: str, service_data: Any = None, *args: Any, **kwargs: Any
    ) -> Any:
        data = dict(service_data or {})
        log.asked.append((f"{domain}.{service}", data, registry.has_service(domain, service)))
        log.contexts.append(kwargs.get("context"))
        return await real_call(registry, domain, service, service_data, *args, **kwargs)

    with patch.object(ServiceRegistry, "async_call", recording_call):
        yield log


@pytest.fixture
def memory_store() -> Generator[None]:
    """Keep Home Assistant's local calendar off the disk."""
    with patch("homeassistant.components.local_calendar.LocalCalendarStore", MemoryStore):
        yield
