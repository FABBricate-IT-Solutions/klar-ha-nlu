#!/usr/bin/env python3
"""Lightweight schema tests for the Klar options flow."""

from __future__ import annotations

import importlib.util
import json
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
        _load(f"{PACKAGE}.languages", "languages.py")
        _load(f"{PACKAGE}.const", "const.py")
        _load(f"{PACKAGE}.lang_select", "lang_select.py")
        return _load(f"{PACKAGE}.config_flow", "config_flow.py")


config_flow = _load_config_flow()


class ConfigFlowSchemaTests(unittest.TestCase):
    def test_assist_filter_is_always_in_options_schema(self) -> None:
        schema = config_flow._options_schema()
        keys = {marker.schema for marker in schema.schema}
        self.assertIn(config_flow.CONF_ASSIST_FILTER, keys)
        self.assertIn(config_flow.CONF_CHANNEL, keys)
        self.assertIn(config_flow.CONF_MODE, keys)
        self.assertNotIn(config_flow.CONF_FALLBACK_AGENT, keys)
        self.assertNotIn(config_flow.CONF_NLU_RAG, keys)
        self.assertNotIn(config_flow.CONF_QUIET_ACK, keys)
        self.assertNotIn(config_flow.CONF_CALENDAR_LLM, keys)
        self.assertNotIn(config_flow.CONF_ALLOW_LLM_TOOLS, keys)
        self.assertNotIn(config_flow.CONF_LANGUAGES, keys)
        self.assertNotIn(config_flow.CONF_PERSONALITY, keys)

    def test_user_schema_offers_release_channel(self) -> None:
        keys = {marker.schema for marker in config_flow.USER_SCHEMA.schema}
        self.assertIn(config_flow.CONF_CHANNEL, keys)
        self.assertIn(config_flow.CONF_MODE, keys)

    def test_options_schema_is_connection_glue_only(self) -> None:
        schema = config_flow._options_schema()
        keys = {marker.schema for marker in schema.schema}
        self.assertNotIn(config_flow.CONF_LANGUAGES, keys)
        self.assertNotIn("show_all_languages", keys)
        src = (ROOT / "custom_components" / "klar_nlu" / "config_flow.py").read_text(encoding="utf-8")
        self.assertNotIn('translation_key="nlu_language"', src)
        self.assertNotIn('translation_key="personality"', src)
        select = (ROOT / "custom_components" / "klar_nlu" / "select.py").read_text(encoding="utf-8")
        self.assertIn('_attr_translation_key = "personality"', select)
        strings = json.loads((ROOT / "custom_components" / "klar_nlu" / "strings.json").read_text(encoding="utf-8"))
        german = json.loads((ROOT / "custom_components" / "klar_nlu" / "translations" / "de.json").read_text(encoding="utf-8"))
        self.assertIn("operator UI", strings["options"]["step"]["init"]["description"])
        self.assertIn("Operator-UI", german["options"]["step"]["init"]["description"])
        self.assertEqual(strings["selector"]["nlu_language"]["options"]["system"], "System language")
        self.assertEqual(german["selector"]["nlu_language"]["options"]["system"], "Systemsprache")
        self.assertEqual(strings["entity"]["select"]["personality"]["name"], "Personality")
        self.assertEqual(german["entity"]["select"]["personality"]["name"], "Persönlichkeit")

    def test_hassio_discovery_uses_local_hass_io(self) -> None:
        src = (ROOT / "custom_components" / "klar_nlu" / "config_flow.py").read_text(encoding="utf-8")
        self.assertIn('f"http://{host}.local.hass.io:10520"', src)
        self.assertNotIn('f"http://{host}:10520"', src)


if __name__ == "__main__":
    unittest.main()
