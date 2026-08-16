#!/usr/bin/env python3
"""Lightweight schema tests for the Klar options flow."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = "klar_config_flow_test"


class _Marker:
    def __init__(self, key: str, default: object = None) -> None:
        self.schema = key
        self.default = default


class _Schema:
    def __init__(self, schema: dict[object, object]) -> None:
        self.schema = schema


class _Selector:
    def __init__(self, config: object = None, **_kwargs: object) -> None:
        self.config = config


class _ConfigFlow:
    def __init_subclass__(cls, **_kwargs: object) -> None:
        super().__init_subclass__()


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


def _load_config_flow() -> types.ModuleType:
    vol = types.ModuleType("voluptuous")
    vol.Schema = _Schema
    vol.Optional = lambda key, default=None: _Marker(key, default)
    vol.Required = lambda key, default=None: _Marker(key, default)

    homeassistant = _module("homeassistant")
    config_entries = types.ModuleType("homeassistant.config_entries")
    config_entries.ConfigFlow = _ConfigFlow
    config_entries.ConfigEntry = object
    config_entries.OptionsFlow = object
    core = types.ModuleType("homeassistant.core")
    core.callback = lambda function: function
    data_entry_flow = types.ModuleType("homeassistant.data_entry_flow")
    data_entry_flow.FlowResult = dict
    helpers = _module("homeassistant.helpers")
    selector = types.ModuleType("homeassistant.helpers.selector")
    for name in (
        "BooleanSelector",
        "ConversationAgentSelector",
        "ConversationAgentSelectorConfig",
        "SelectSelector",
        "SelectSelectorConfig",
        "TextSelector",
        "TextSelectorConfig",
    ):
        setattr(selector, name, _Selector)
    selector.SelectSelectorMode = types.SimpleNamespace(
        DROPDOWN="dropdown",
        LIST="list",
    )
    helpers.selector = selector
    package = _module(PACKAGE)
    modules = {
        "voluptuous": vol,
        "homeassistant": homeassistant,
        "homeassistant.config_entries": config_entries,
        "homeassistant.core": core,
        "homeassistant.data_entry_flow": data_entry_flow,
        "homeassistant.helpers": helpers,
        "homeassistant.helpers.selector": selector,
        PACKAGE: package,
    }
    with patch.dict(sys.modules, modules):
        _load(f"{PACKAGE}.const", "const.py")
        return _load(f"{PACKAGE}.config_flow", "config_flow.py")


config_flow = _load_config_flow()


class ConfigFlowSchemaTests(unittest.TestCase):
    def test_assist_filter_is_always_in_options_schema(self) -> None:
        schema = config_flow._options_schema()
        keys = {marker.schema for marker in schema.schema}
        self.assertIn(config_flow.CONF_ASSIST_FILTER, keys)

    def test_options_schema_has_no_advanced_flag(self) -> None:
        with self.assertRaises(TypeError):
            config_flow._options_schema(False)


if __name__ == "__main__":
    unittest.main()
