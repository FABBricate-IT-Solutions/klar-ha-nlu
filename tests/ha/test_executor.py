#!/usr/bin/env python3
"""Structured execute results and confirm secrecy."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import AsyncMock, patch

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = "klar_executor_test"


def _module(name: str) -> types.ModuleType:
    module = types.ModuleType(name)
    module.__path__ = []
    return module


def _load(name: str, rel: str) -> types.ModuleType:
    path = ROOT / "custom_components" / "klar_nlu" / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def _load_stack() -> tuple[types.ModuleType, types.ModuleType, types.ModuleType]:
    homeassistant = _module("homeassistant")
    components = _module("homeassistant.components")
    conversation = types.ModuleType("homeassistant.components.conversation")
    conversation.ConversationInput = object
    core = types.ModuleType("homeassistant.core")
    core.HomeAssistant = object
    helpers = _module("homeassistant.helpers")
    intent = types.ModuleType("homeassistant.helpers.intent")
    intent.async_handle = AsyncMock()
    area_registry = types.ModuleType("homeassistant.helpers.area_registry")
    entity_registry = types.ModuleType("homeassistant.helpers.entity_registry")
    floor_registry = types.ModuleType("homeassistant.helpers.floor_registry")
    device_registry = types.ModuleType("homeassistant.helpers.device_registry")
    helpers.intent = intent
    helpers.area_registry = area_registry
    helpers.entity_registry = entity_registry
    helpers.floor_registry = floor_registry
    helpers.device_registry = device_registry
    package = _module(PACKAGE)
    modules = {
        "homeassistant": homeassistant,
        "homeassistant.components": components,
        "homeassistant.components.conversation": conversation,
        "homeassistant.core": core,
        "homeassistant.helpers": helpers,
        "homeassistant.helpers.intent": intent,
        "homeassistant.helpers.area_registry": area_registry,
        "homeassistant.helpers.entity_registry": entity_registry,
        "homeassistant.helpers.floor_registry": floor_registry,
        "homeassistant.helpers.device_registry": device_registry,
        PACKAGE: package,
    }
    with patch.dict(sys.modules, modules):
        contracts = _load(f"{PACKAGE}.contracts", "contracts.py")
        sys.modules[f"{PACKAGE}.contracts"] = contracts
        _load(f"{PACKAGE}.clock_speech", "clock_speech.py")
        _load(f"{PACKAGE}.speech_place", "speech_place.py")
        _load("clock_speech", "clock_speech.py")
        _load("speech_place", "speech_place.py")
        speech = _load(f"{PACKAGE}.speech", "speech.py")
        sys.modules[f"{PACKAGE}.speech"] = speech
        _load(f"{PACKAGE}.speech_locale", "speech_locale.py")
        _load(f"{PACKAGE}.calendar_session", "calendar_session.py")
        _load("calendar_session", "calendar_session.py")
        package.calendar_entity = _load(f"{PACKAGE}.calendar_entity", "calendar_entity.py")
        package.calendar_say = _load(f"{PACKAGE}.calendar_say", "calendar_say.py")
        _load("calendar_entity", "calendar_entity.py")
        _load("calendar_say", "calendar_say.py")
        _load(f"{PACKAGE}.calendar_ha", "calendar_ha.py")
        _load(f"{PACKAGE}.languages", "languages.py")
        _load(f"{PACKAGE}.const", "const.py")
        _load(f"{PACKAGE}.lang_select", "lang_select.py")
        intents = _load(f"{PACKAGE}.intents", "intents.py")
        sys.modules[f"{PACKAGE}.intents"] = intents
        _load(f"{PACKAGE}.speech_status_device", "speech_status_device.py")
        _load(f"{PACKAGE}.speech_status", "speech_status.py")
        _load(f"{PACKAGE}.floor_query", "floor_query.py")
        _load(f"{PACKAGE}.dispatch_result", "dispatch_result.py")
        _load(f"{PACKAGE}.refine", "refine.py")
        _load(f"{PACKAGE}.stream", "stream.py")
        _load(f"{PACKAGE}.engine_llm", "engine_llm.py")
        _load(f"{PACKAGE}.speech_snapshot", "speech_snapshot.py")
        _load(f"{PACKAGE}.speech_render", "speech_render.py")
        _load(f"{PACKAGE}.dispatch_media", "dispatch_media.py")
        dispatch = _load(f"{PACKAGE}.dispatch", "dispatch.py")
        sys.modules[f"{PACKAGE}.dispatch"] = dispatch
        executor = _load(f"{PACKAGE}.executor", "executor.py")
    return contracts, dispatch, executor


contracts, dispatch, executor = _load_stack()


def _item(name: str, **slots: object) -> dict[str, object]:
    return {"name": name, "slots": [{"name": key, "value": value} for key, value in slots.items()]}


class ExecutorTests(unittest.IsolatedAsyncioTestCase):
    async def test_partial_failure_is_first_class(self) -> None:
        results = [
            dispatch.IntentStepResult(True, speech="Wohnzimmer ist an."),
            dispatch.IntentStepResult(False, error="kitchen_failed"),
        ]

        async def _handle(*_args: object, **_kwargs: object) -> dispatch.IntentStepResult:
            return results.pop(0)

        with patch.object(executor, "handle_intent", side_effect=_handle):
            payload = await executor.execute_plan(
                SimpleNamespace(),
                SimpleNamespace(text="x", context=object(), language="de"),
                [_item("HassTurnOn", area="living"), _item("HassTurnOn", area="kitchen")],
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertEqual(payload["outcome"], "partial")
        self.assertEqual([step["status"] for step in payload["steps"]], ["success", "error"])
        self.assertIn("Wohnzimmer ist an.", payload["speech"])
        self.assertIn("fehlgeschlagen", payload["speech"])
        self.assertNotEqual(payload["speech"], "Wohnzimmer ist an.")

    async def test_all_failures_are_not_success(self) -> None:
        with patch.object(executor, "handle_intent", return_value=dispatch.IntentStepResult(False, error="boom")):
            payload = await executor.execute_plan(
                SimpleNamespace(),
                SimpleNamespace(text="x", context=object(), language="en"),
                [_item("HassTurnOn", area="living")],
                "en",
                None,
                lambda _entity_id: True,
            )
        self.assertEqual(payload["outcome"], "error")
        self.assertEqual(payload["steps"][0]["status"], "error")
        self.assertIn("did not work", payload["speech"])

    async def test_media_failure_speaks_a_useful_reason(self) -> None:
        with patch.object(
            executor,
            "handle_intent",
            return_value=dispatch.IntentStepResult(False, error="media_unavailable"),
        ):
            payload = await executor.execute_plan(
                SimpleNamespace(),
                SimpleNamespace(text="x", context=object(), language="de"),
                [_item("HassMediaSearchAndPlay", entity_id="media_player.e01", search_query="linkin park")],
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertEqual(payload["outcome"], "error")
        self.assertIn("nicht erreichbar", payload["speech"])
        self.assertNotEqual(payload["speech"], "Das hat nicht geklappt.")

    async def test_unavailable_tv_names_the_television(self) -> None:
        with patch.object(
            executor,
            "handle_intent",
            return_value=dispatch.IntentStepResult(False, error="media_unavailable"),
        ):
            payload = await executor.execute_plan(
                SimpleNamespace(),
                SimpleNamespace(text="x", context=object(), language="de"),
                [_item("HassTurnOn", entity_id="media_player.wohnzimmer_tv")],
                "de",
                None,
                lambda _entity_id: True,
            )
        self.assertEqual(payload["outcome"], "error")
        self.assertIn("Fernseher", payload["speech"])
        self.assertNotIn("Licht", payload["speech"])
        self.assertNotEqual(payload["speech"], "Das hat nicht geklappt.")

    def test_confirm_payload_never_yields_intents(self) -> None:
        payload = {
            "schema_version": "2.0",
            "text": "lock the door",
            "conversation_id": "c1",
            "decision": {"type": "confirm", "prompt": "Confirm?", "candidate_id": "selected-000"},
            "speech": "Confirm?",
            "confidence": 1.0,
            "margin": 1.0,
            "candidates": [],
            "evidence": [],
            "trace": {"stages": [], "discarded": []},
            "briefing": False,
        }
        checked = contracts.validate_v2_payload(payload)
        self.assertEqual(contracts.executable_intents(checked), [])


if __name__ == "__main__":
    unittest.main()
