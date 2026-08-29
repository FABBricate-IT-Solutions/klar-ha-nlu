#!/usr/bin/env python3
"""Channel helpers for bundled-engine stable vs staging downloads."""

from __future__ import annotations

import importlib.util
import sys
import types
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]


def _load_const():
    languages = types.ModuleType("klar_channel_test.languages")
    languages.LANGUAGE_VARIANTS = {}
    languages.SUPPORTED_LANGUAGES = ("de", "en")
    package = types.ModuleType("klar_channel_test")
    package.__path__ = []
    with patch.dict(
        sys.modules,
        {
            "klar_channel_test": package,
            "klar_channel_test.languages": languages,
        },
    ):
        path = ROOT / "custom_components" / "klar_nlu" / "const.py"
        spec = importlib.util.spec_from_file_location("klar_channel_test.const", path)
        if spec is None or spec.loader is None:
            raise RuntimeError(f"cannot load {path}")
        module = importlib.util.module_from_spec(spec)
        sys.modules["klar_channel_test.const"] = module
        spec.loader.exec_module(module)
        return module


const = _load_const()


class EngineChannelTests(unittest.TestCase):
    def test_followup_session_key_and_keep(self) -> None:
        self.assertTrue(const.keeps_conversation("execute"))
        self.assertTrue(const.keeps_conversation("clarify"))
        self.assertTrue(const.keeps_conversation("chat"))
        self.assertEqual(const.engine_session_id("dev-1", None), "dev:dev-1")
        self.assertEqual(const.engine_session_id(None, "sat-1"), "dev:sat-1")
        self.assertEqual(const.engine_session_id(None, None), const.FOLLOWUP_SESSION)
        self.assertEqual(const.parse_session_id("assist-9", None, None), "assist-9")
        self.assertEqual(const.parse_session_id(None, None, None), const.FOLLOWUP_SESSION)

    def test_resolve_channel_defaults_stable(self) -> None:
        self.assertEqual(const.resolve_channel(None), const.CHANNEL_STABLE)
        self.assertEqual(const.resolve_channel("stable"), const.CHANNEL_STABLE)
        self.assertEqual(const.resolve_channel("nightly"), const.CHANNEL_STABLE)
        self.assertEqual(const.resolve_channel("staging"), const.CHANNEL_STAGING)

    def test_pick_staging_skips_latest_and_non_staging(self) -> None:
        releases = [
            {"tag_name": "2026.8.30", "prerelease": False, "name": "stable"},
            {"tag_name": "2026.8.30-rc.1", "prerelease": True, "name": "other"},
            {
                "tag_name": "2026.8.30-staging.abc1234",
                "prerelease": True,
                "name": "wanted",
            },
        ]
        chosen = const.pick_staging_release(releases)
        self.assertIsNotNone(chosen)
        self.assertEqual(chosen["name"], "wanted")

    def test_addon_url_follows_channel(self) -> None:
        self.assertEqual(
            const.addon_url_for_channel(const.CHANNEL_STABLE),
            const.DEFAULT_ADDON_URL,
        )
        self.assertEqual(
            const.addon_url_for_channel(const.CHANNEL_STAGING),
            const.DEFAULT_STAGING_ADDON_URL,
        )

    def test_resolve_engine_url_rewrites_managed_hosts(self) -> None:
        self.assertEqual(
            const.resolve_engine_url(
                mode=const.MODE_REMOTE,
                channel=const.CHANNEL_STAGING,
                url=const.DEFAULT_ADDON_URL,
            ),
            const.DEFAULT_STAGING_ADDON_URL,
        )
        self.assertEqual(
            const.resolve_engine_url(
                mode=const.MODE_REMOTE,
                channel=const.CHANNEL_STABLE,
                url=const.DEFAULT_STAGING_ADDON_URL,
            ),
            const.DEFAULT_ADDON_URL,
        )
        self.assertEqual(
            const.resolve_engine_url(
                mode=const.MODE_LOCAL,
                channel=const.CHANNEL_STAGING,
                url=const.DEFAULT_ADDON_URL,
            ),
            const.DEFAULT_URL,
        )
        self.assertEqual(
            const.resolve_engine_target(
                mode=const.MODE_LOCAL,
                channel=const.CHANNEL_STAGING,
                url=const.DEFAULT_URL,
                supervisor=True,
            ),
            (const.MODE_REMOTE, const.DEFAULT_STAGING_ADDON_URL),
        )
        self.assertEqual(
            const.resolve_engine_target(
                mode=const.MODE_LOCAL,
                channel=const.CHANNEL_STABLE,
                url=const.DEFAULT_STAGING_ADDON_URL,
                supervisor=True,
            ),
            (const.MODE_REMOTE, const.DEFAULT_ADDON_URL),
        )

    def test_resolve_engine_url_keeps_custom_host(self) -> None:
        custom = "http://192.168.1.40:10520"
        self.assertEqual(
            const.resolve_engine_url(
                mode=const.MODE_REMOTE,
                channel=const.CHANNEL_STAGING,
                url=custom,
            ),
            custom,
        )

    def test_resolve_engine_url_keeps_supervisor_prefix(self) -> None:
        prefixed = "http://xyz-klar-nlu:10520"
        fqdn = "http://8db2ab02-klar-nlu.local.hass.io:10520"
        for url in (prefixed, fqdn):
            self.assertTrue(const.is_managed_engine_url(url), url)
            self.assertEqual(
                const.resolve_engine_url(
                    mode=const.MODE_REMOTE,
                    channel=const.CHANNEL_STABLE,
                    url=url,
                    supervisor=True,
                ),
                url,
            )
            self.assertEqual(
                const.resolve_engine_target(
                    mode=const.MODE_REMOTE,
                    channel=const.CHANNEL_STABLE,
                    url=url,
                    supervisor=True,
                ),
                (const.MODE_REMOTE, url),
            )

    def test_resolve_engine_url_retargets_prefixed_channel(self) -> None:
        self.assertEqual(
            const.resolve_engine_url(
                mode=const.MODE_REMOTE,
                channel=const.CHANNEL_STAGING,
                url="http://xyz-klar-nlu:10520",
                supervisor=True,
            ),
            "http://xyz-klar-nlu-staging:10520",
        )
        self.assertEqual(
            const.resolve_engine_url(
                mode=const.MODE_REMOTE,
                channel=const.CHANNEL_STABLE,
                url="http://xyz-klar-nlu-staging.local.hass.io:10520",
                supervisor=True,
            ),
            "http://xyz-klar-nlu.local.hass.io:10520",
        )

    def test_hassio_discovery_slug_url_is_kept(self) -> None:
        discovered = "http://8db2ab02-klar-nlu:10520"
        self.assertEqual(
            const.resolve_engine_target(
                mode=const.MODE_REMOTE,
                channel=const.CHANNEL_STABLE,
                url=discovered,
                supervisor=True,
            ),
            (const.MODE_REMOTE, discovered),
        )

    def test_channel_for_addon_slug(self) -> None:
        self.assertEqual(
            const.channel_for_addon_slug("klar_nlu_staging"),
            const.CHANNEL_STAGING,
        )
        self.assertEqual(
            const.channel_for_addon_slug("klar-nlu-staging"),
            const.CHANNEL_STAGING,
        )
        self.assertEqual(
            const.channel_for_addon_slug("klar_nlu"),
            const.CHANNEL_STABLE,
        )

    def test_pick_staging_requires_prerelease_flag(self) -> None:
        self.assertIsNone(
            const.pick_staging_release(
                [{"tag_name": "2026.8.30-staging.deadbee", "prerelease": False}]
            )
        )
        self.assertIsNone(const.pick_staging_release("nope"))

    def test_addon_and_engine_drop_armv7(self) -> None:
        engine = (ROOT / "custom_components" / "klar_nlu" / "engine.py").read_text()
        self.assertIn("languages", engine)
        self.assertIn("ui_locale", engine)
        self.assertNotIn("armv7", engine)
        build = (ROOT / ".github" / "workflows" / "build.yml").read_text()
        self.assertNotIn("armv7", build)
        self.assertIn("x86_64-unknown-linux-musl", build)
        self.assertIn("aarch64-unknown-linux-musl", build)
        self.assertNotIn("unknown-linux-gnu", build)
        for rel in (
            "config.yaml",
            "addon/config.yaml",
            "addon-staging/config.yaml",
            "addon/build.yaml",
            "addon-staging/build.yaml",
        ):
            self.assertNotIn("armv7", (ROOT / rel).read_text(), rel)


if __name__ == "__main__":
    unittest.main()
