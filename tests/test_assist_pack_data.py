"""Package 6.9: assist stubs are one-file corrections; generator never overwrites them."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from lang_packs.emit_assist import ensure_assist_file, stub_yaml  # noqa: E402


class AssistPackDataTests(unittest.TestCase):
    def test_generator_does_not_overwrite_assist_correction(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            assist = Path(tmp) / "packs" / "assist"
            assist.mkdir(parents=True)
            path = assist / "fr.yaml"
            path.write_text(
                stub_yaml({"code": "fr", "mod": "fr", "variants": ["fr", "fr-FR"], "parent": None})
                + "verbs:\n  ampoule: On\n",
                encoding="utf-8",
            )
            before = path.read_text(encoding="utf-8")
            # Point emitter at the temp tree.
            import lang_packs.emit_assist as emit_assist

            old = emit_assist.ASSIST
            emit_assist.ASSIST = assist
            try:
                ensure_assist_file({"code": "fr", "mod": "fr", "variants": ["fr", "fr-FR"], "parent": None})
            finally:
                emit_assist.ASSIST = old
            self.assertEqual(path.read_text(encoding="utf-8"), before)
            self.assertIn("ampoule: On", before)

    def test_stub_is_single_locale_data_file(self) -> None:
        text = stub_yaml({"code": "nl", "mod": "nl", "variants": ["nl", "nl-NL"], "parent": None})
        self.assertIn("id: nl", text)
        self.assertIn("klar_lang_pack:", text)
        self.assertIn("intents: []", text)


if __name__ == "__main__":
    unittest.main()
