"""Package 4.4: pack_lint catches known defects before packs are fixed."""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from lang_packs.pack_lint import lint_packs  # noqa: E402


def _write_pack(root: Path, name: str, *, speech: str, body: str, verbs: str) -> None:
    pack = root / name
    pack.mkdir(parents=True)
    (pack / "speech.rs").write_text(f"Speech {{\n{speech}\n}}\n", encoding="utf-8")
    (pack / "pack.rs").write_text(body, encoding="utf-8")
    (pack / "verbs.rs").write_text(verbs, encoding="utf-8")


class PackLintCatchTests(unittest.TestCase):
    def test_catches_bare_identical_actions_and_english_colors(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            packs = root / "packs"
            # Minimal en reference for placeholder parity.
            _write_pack(
                packs,
                "en",
                speech='lock: "Locked {target}.",\nunlock: "Unlocked {target}.",\n'
                'cover_open: "Opened {target}.",\ncover_close: "Closed {target}.",\n'
                'done: "Done: {name}.",\nturn_on: "{target} is on.",\nturn_off: "{target} is off.",',
                body='colors: &[("red", "red")],\nweekday: &["what day"],\nclock: &["what time"],\n'
                'weather: &["weather"],\ngreet: &["hello"],\nfeeling: &["mood"],\n',
                verbs='("turn", VerbKind::On),\n',
            )
            _write_pack(
                packs,
                "ar",
                speech='lock: "{target}.",\nunlock: "{target}.",\n'
                'cover_open: "{target}.",\ncover_close: "{target}.",\n'
                'done: "{name}",\nturn_on: "{target}.",\nturn_off: "{target}.",',
                body='colors: &[("red", "red"), ("blue", "blue")],\n'
                'weekday: &[],\nclock: &["ايش موقت"],\nweather: &["ايش حراره"],\n'
                'explain: &["ايش اطفي حط"],\ngreet: &["شغل"],\nfeeling: &["mood"],\n'
                'tell: &["tell"],\nadvice: &["idea"],\n'
                'fillers: &["und"],\nlight_nouns: &["ضوء"],\n',
                verbs='("شغل", VerbKind::On),\n("اطفي", VerbKind::Off),\n("حط", VerbKind::Set),\n'
                '("ايش", VerbKind::Query),\n("موقت", VerbKind::On),\n("حراره", VerbKind::On),\n',
            )
            registry = root / "registry.rs"
            registry.write_text(
                'LangMeta { code: "ar", native_name: "Hindi", script: "Arab", variants: &["ar"], parent: None, tier: Tier::Beta },\n',
                encoding="utf-8",
            )
            errors = lint_packs(packs, registry)
            blob = "\n".join(errors)
            self.assertIn("placeholder-only", blob)
            self.assertIn("identical", blob)
            self.assertIn("English color", blob)
            self.assertIn("weekday is empty", blob)
            self.assertIn("EN stubs", blob)
            self.assertIn("German und", blob)
            self.assertIn("native_name is English", blob)

    def test_staging_packs_are_green(self) -> None:
        packs = ROOT / "src" / "lang" / "packs"
        registry = ROOT / "src" / "lang" / "registry.rs"
        if not packs.is_dir():
            self.skipTest("packs missing")
        errors = lint_packs(packs, registry)
        self.assertEqual(errors, [], msg="\n".join(errors[:40]))


if __name__ == "__main__":
    unittest.main()
