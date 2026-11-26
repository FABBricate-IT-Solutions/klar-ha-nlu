#!/usr/bin/env python3
"""Package 6.10: the stamped Klar-vs-Assist benchmark and the way CI produces it."""

from __future__ import annotations

import copy
import importlib.util
import json
import re
import tempfile
import unittest
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
SHA = "0123456789abcdef0123456789abcdef01234567"
PINNED = ("hassil", "home-assistant-intents", "gazetteer-matcher")


def load(name: str):
    spec = importlib.util.spec_from_file_location(f"klar_{name}", SCRIPTS / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def pins(path: Path) -> dict[str, str]:
    found = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        match = re.match(r"^([A-Za-z0-9_.-]+)==([^\s#]+)", line)
        if match:
            found[match.group(1).lower()] = match.group(2)
    return found


def rate(ok: int, n: int) -> dict:
    return {"n": n, "ok": ok, "fail": n - ok, "accuracy": round(100.0 * ok / n, 2)}


def scorecard(commit: str = SHA) -> dict:
    metrics = {
        "accuracy": 0.97,
        "in_domain_accuracy": 0.95,
        "intent_macro_f1": 1.0,
        "slot_micro_f1": 1.0,
        "reject_fpr": 0.01,
        "reject_fnr": 0.004,
        "reject_unsafe": 0.0,
        "typo_recovery": 0.78,
    }
    return {
        "schema_version": "m7.2",
        "commit": commit,
        "languages": [
            {"language": "en", "utterances": 1000, "in_domain": 560, "metrics": metrics},
            {"language": "de", "utterances": 1000, "in_domain": 560, "metrics": metrics},
        ],
    }


def assist(commit: str = SHA, languages: tuple[str, ...] = ("en", "de")) -> dict:
    def block(ok: int, only: int) -> dict:
        return {**rate(ok, 100), "hassil_only": rate(only, 100)}

    return {
        "commit": commit,
        "versions": {"hassil": "3.12.1"},
        "gazetteer_languages": ["en"],
        "languages": {
            code: {**block(80, 70), "in_domain": block(60, 50), "out_of_domain": block(100, 100)} for code in languages
        },
    }


class JoinTests(unittest.TestCase):
    def setUp(self) -> None:
        self.report = load("benchmark_report")

    def test_joins_both_runs_per_language_under_the_commit(self) -> None:
        joined = self.report.join(scorecard(), assist())
        self.assertEqual(joined["commit"], SHA)
        self.assertEqual(sorted(joined["languages"]), ["de", "en"])
        en = joined["languages"]["en"]
        self.assertEqual(en["in_domain"], 560)
        self.assertEqual(en["assist"]["in_domain_accuracy"], 0.6)
        self.assertEqual(en["assist"]["in_domain_hassil_only_accuracy"], 0.5)
        self.assertTrue(en["assist"]["gazetteer"])
        self.assertFalse(joined["languages"]["de"]["assist"]["gazetteer"])
        self.assertEqual(en["klar"]["reject_unsafe"], 0.0)

    def test_refuses_runs_of_different_commits(self) -> None:
        with self.assertRaisesRegex(self.report.ReportError, "scorecard is for"):
            self.report.join(scorecard(), assist("f" * 40))

    def test_refuses_a_run_without_a_full_sha(self) -> None:
        for bad in ("unknown", "", "abc123", None):
            with self.subTest(commit=bad):
                with self.assertRaisesRegex(self.report.ReportError, "full commit SHA"):
                    self.report.join(scorecard(bad), assist(bad))

    def test_refuses_when_assist_has_no_result_for_a_language(self) -> None:
        with self.assertRaisesRegex(self.report.ReportError, "no results for de"):
            self.report.join(scorecard(), assist(languages=("en",)))

    def test_the_report_has_no_confidence_figure(self) -> None:
        text = json.dumps(self.report.join(scorecard(), assist()))
        for banned in ("ece", "brier", "confidence"):
            self.assertNotIn(f'"{banned}"', text)

    def test_cli_writes_json_and_markdown(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "s.json").write_text(json.dumps(scorecard()), encoding="utf-8")
            (root / "a.json").write_text(json.dumps(assist()), encoding="utf-8")
            args = ["--scorecard", str(root / "s.json"), "--assist", str(root / "a.json"), "--out", str(root / "b.json")]
            self.assertEqual(self.report.main([*args, "--markdown", str(root / "b.md")]), 0)
            self.assertEqual(json.loads((root / "b.json").read_text(encoding="utf-8"))["commit"], SHA)
            self.assertIn(SHA[:12], (root / "b.md").read_text(encoding="utf-8"))
            bad = copy.deepcopy(assist())
            bad["commit"] = "unknown"
            (root / "a.json").write_text(json.dumps(bad), encoding="utf-8")
            self.assertEqual(self.report.main(args), 1)


class PinTests(unittest.TestCase):
    def test_assist_packages_are_pinned_with_the_home_assistant_tests(self) -> None:
        bench = pins(SCRIPTS / "requirements-assist-bench.txt")
        ha = pins(ROOT / "tests" / "ha_real" / "requirements.txt")
        for name in PINNED:
            self.assertIn(name, bench, f"{name} is not pinned for the benchmark")
            self.assertEqual(bench[name], ha[name], f"{name}: the benchmark and tests/ha_real disagree")


class WorkflowTests(unittest.TestCase):
    def setUp(self) -> None:
        workflow = yaml.safe_load((ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8"))
        self.job = workflow["jobs"]["benchmark"]
        self.run = "\n".join(step.get("run", "") for step in self.job["steps"])

    def test_job_stamps_the_commit_and_runs_both_engines(self) -> None:
        self.assertEqual(self.job["env"]["KLAR_COMMIT_SHA"], "${{ github.sha }}")
        self.assertIn("eval gate", self.run)
        self.assertIn("bench_assist.py --suite heldout_en --suite heldout_de", self.run)
        self.assertIn("benchmark_report.py", self.run)

    def test_job_publishes_the_joined_artifact_under_the_sha(self) -> None:
        uploads = [step for step in self.job["steps"] if str(step.get("uses", "")).startswith("actions/upload-artifact@")]
        self.assertEqual(len(uploads), 1)
        self.assertEqual(uploads[0]["with"]["name"], "benchmark-${{ github.sha }}")
        self.assertEqual(uploads[0]["with"]["if-no-files-found"], "error")

    def test_job_installs_the_pinned_requirements(self) -> None:
        self.assertIn("scripts/requirements-assist-bench.txt", self.run)


try:
    import gazetteer_matcher  # noqa: F401
    import hassil  # noqa: F401

    HAVE_ASSIST = True
except ImportError:
    HAVE_ASSIST = False


@unittest.skipUnless(HAVE_ASSIST, "hassil and gazetteer-matcher are installed by the benchmark job only")
class GazetteerAgentTests(unittest.TestCase):
    def setUp(self) -> None:
        self.module = load("assist_gazetteer")
        path = ROOT / "tests" / "datasets" / "family_home_en" / "home_config.yaml"
        self.home = yaml.safe_load(path.read_text(encoding="utf-8"))

    def test_home_carries_every_device_with_its_area(self) -> None:
        built = self.module.build_home(self.home)
        self.assertEqual(len(built["entities"]) - len(self.home["scenes"]) - len(self.home["scripts"]), len(self.home["devices"]))
        self.assertEqual(built["entities"]["light.entryway_light"]["area"], "entryway")

    def test_english_sentence_is_read_as_the_entity(self) -> None:
        intents = self.module.GazetteerAgent(self.home, "en").interpret("Turn on the Entryway Light")
        self.assertEqual([i["name"] for i in intents], ["HassTurnOn"])
        self.assertEqual(intents[0]["slots"]["entity_id"], "light.entryway_light")

    def test_other_languages_are_left_to_hassil_as_in_core(self) -> None:
        agent = self.module.GazetteerAgent(self.home, "de")
        self.assertFalse(agent.supported)
        self.assertEqual(agent.interpret("Turn on the Entryway Light"), [])


if __name__ == "__main__":
    unittest.main()
