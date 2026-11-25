"""Config-entry lifecycle against a real Home Assistant core: setup, unload, reload."""

from __future__ import annotations

from engine_fake import ENGINE_URL, FakeEngine
from homeassistant.config_entries import ConfigEntryState
from homeassistant.core import HomeAssistant
from homeassistant.helpers import entity_registry as er
from pytest_homeassistant_custom_component.common import MockConfigEntry

from custom_components.klar_nlu.const import DOMAIN
from custom_components.klar_nlu.panel import PANEL_PATH

AGENT = "conversation.klar_nlu"
SERVICES = ("undo", "teach_alias", "export_asr_boost")


def _platform_domains(hass: HomeAssistant, entry: MockConfigEntry) -> set[str]:
    registry = er.async_get(hass)
    return {item.domain for item in er.async_entries_for_config_entry(registry, entry.entry_id)}


async def test_setup_loads_every_platform(hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine) -> None:
    assert klar.state is ConfigEntryState.LOADED
    assert _platform_domains(hass, klar) == {"ai_task", "conversation", "select", "sensor", "switch"}
    assert hass.states.get(AGENT) is not None
    for service in SERVICES:
        assert hass.services.has_service(DOMAIN, service)
    assert hass.data[DOMAIN][klar.entry_id]["url"] == ENGINE_URL


async def test_setup_pushes_the_home_graph(hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine) -> None:
    assert engine.home_snapshots, "setup must push one home snapshot"
    snapshot = engine.home_snapshots[0]
    assert snapshot["schema_version"] == "1"
    assert AGENT in {row["entity_id"] for row in snapshot["entities"]}


async def test_setup_reads_engine_settings(hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine) -> None:
    assert hass.data[DOMAIN][klar.entry_id]["engine_settings"] == engine.settings


async def test_setup_survives_an_unreachable_engine(
    hass: HomeAssistant, engine: FakeEngine, klar_entry: MockConfigEntry
) -> None:
    engine.reachable = False
    assert await hass.config_entries.async_setup(klar_entry.entry_id)
    await hass.async_block_till_done()
    assert klar_entry.state is ConfigEntryState.LOADED
    assert hass.states.get(AGENT) is not None
    assert await hass.config_entries.async_unload(klar_entry.entry_id)
    await hass.async_block_till_done()


async def test_unload_releases_entities_and_stops_sync(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    sync = hass.data[DOMAIN][klar.entry_id]["sync"]
    assert await hass.config_entries.async_unload(klar.entry_id)
    await hass.async_block_till_done()
    assert klar.state is ConfigEntryState.NOT_LOADED
    assert klar.entry_id not in hass.data[DOMAIN]
    state = hass.states.get(AGENT)
    assert state is None or state.state == "unavailable"
    pushed = len(engine.home_snapshots)
    er.async_get(hass).async_get_or_create("light", "test", "after-unload")
    hass.bus.async_fire("entity_registry_updated", {"action": "create", "entity_id": "light.test_after_unload"})
    await hass.async_block_till_done()
    assert len(engine.home_snapshots) == pushed, "a stopped sync must not push again"
    assert sync._tick is None


async def test_reload_comes_back_loaded(hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine) -> None:
    pushed = len(engine.home_snapshots)
    assert await hass.config_entries.async_reload(klar.entry_id)
    await hass.async_block_till_done()
    assert klar.state is ConfigEntryState.LOADED
    assert hass.states.get(AGENT) is not None
    assert len(engine.home_snapshots) > pushed, "reload must push the home graph again"
    assert _platform_domains(hass, klar) == {"ai_task", "conversation", "select", "sensor", "switch"}


async def test_setup_twice_keeps_one_agent(hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine) -> None:
    for _ in range(2):
        assert await hass.config_entries.async_reload(klar.entry_id)
        await hass.async_block_till_done()
    registry = er.async_get(hass)
    agents = [
        item
        for item in er.async_entries_for_config_entry(registry, klar.entry_id)
        if item.domain == "conversation"
    ]
    assert len(agents) == 1


async def test_unload_removes_services(hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine) -> None:
    assert await hass.config_entries.async_unload(klar.entry_id)
    await hass.async_block_till_done()
    assert [service for service in SERVICES if hass.services.has_service(DOMAIN, service)] == []


async def test_unload_removes_the_operator_panel(
    hass: HomeAssistant, klar: MockConfigEntry, engine: FakeEngine
) -> None:
    assert PANEL_PATH in hass.data["frontend_panels"]
    assert await hass.config_entries.async_unload(klar.entry_id)
    await hass.async_block_till_done()
    assert PANEL_PATH not in hass.data["frontend_panels"]
