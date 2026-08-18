#!/usr/bin/env python3
"""Klar sidebar dashboard config must include LovelaceStorage's id."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
PACKAGE = "klar_panel_test"


def _load(name: str, rel: str) -> types.ModuleType:
    path = ROOT / "custom_components" / "klar_nlu" / rel
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def _load_panel() -> types.ModuleType:
    package = types.ModuleType(PACKAGE)
    package.__path__ = []
    homeassistant = types.ModuleType("homeassistant")
    components = types.ModuleType("homeassistant.components")
    frontend = types.ModuleType("homeassistant.components.frontend")
    frontend.add_extra_js_url = lambda *_args, **_kwargs: None
    core = types.ModuleType("homeassistant.core")
    core.HomeAssistant = object
    homeassistant.components = components
    components.frontend = frontend
    with patch.dict(
        sys.modules,
        {
            PACKAGE: package,
            "homeassistant": homeassistant,
            "homeassistant.components": components,
            "homeassistant.components.frontend": frontend,
            "homeassistant.core": core,
        },
    ):
        _load(f"{PACKAGE}.languages", "languages.py")
        _load(f"{PACKAGE}.const", "const.py")
        return _load(f"{PACKAGE}.panel", "panel.py")


panel = _load_panel()


class PanelDashboardTests(unittest.TestCase):
    def test_dashboard_config_has_storage_id(self) -> None:
        config = panel._dashboard_config("klar-nlu")
        self.assertEqual(config["id"], "klar-nlu")
        self.assertEqual(config["url_path"], "klar-nlu")
        self.assertEqual(config["mode"], "storage")


if __name__ == "__main__":
    unittest.main()
