#!/usr/bin/env python3
"""The aggregate gate `release-gates` and the workflow wiring around it."""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import os
import re
import unittest
from pathlib import Path
from unittest import mock

import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github" / "workflows"


def _load():
    path = ROOT / "scripts" / "release_gates.py"
    spec = importlib.util.spec_from_file_location("klar_release_gates", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


gates = _load()


def workflow(name: str) -> dict:
    return yaml.safe_load((WORKFLOWS / name).read_text(encoding="utf-8"))


def run_gate(needs: object) -> tuple[int, str]:
    raw = needs if isinstance(needs, str) else json.dumps(needs)
    out = io.StringIO()
    with mock.patch.dict(os.environ, {"NEEDS": raw}), contextlib.redirect_stdout(out), contextlib.redirect_stderr(out):
        code = gates.main()
    return code, out.getvalue()


class ReleaseGatesScriptTests(unittest.TestCase):
    def test_passes_when_every_needed_job_succeeded(self) -> None:
        code, out = run_gate({"test": {"result": "success", "outputs": {}}, "web": {"result": "success", "outputs": {}}})
        self.assertEqual(code, 0, out)
        self.assertIn("all 2 needed jobs succeeded", out)

    def test_fails_and_names_every_job_that_did_not_succeed(self) -> None:
        for result in ("failure", "cancelled", "skipped"):
            with self.subTest(result=result):
                code, out = run_gate({"test": {"result": "success"}, "datasets": {"result": result}})
                self.assertEqual(code, 1, out)
                self.assertIn("not successful: datasets", out)
                self.assertNotIn("not successful: datasets, test", out)

    def test_fails_without_a_usable_needs_context(self) -> None:
        for raw in ("", "{}", "[]", "not json", '{"test": "success"}', '{"test": {}}'):
            with self.subTest(raw=raw):
                code, out = run_gate(raw)
                self.assertEqual(code, 1, out)
                self.assertIn("cannot read the needs context", out)


class CiWorkflowTests(unittest.TestCase):
    def setUp(self) -> None:
        self.jobs = workflow("ci.yml")["jobs"]
        self.gate = self.jobs["release-gates"]

    def test_release_gates_needs_every_other_job(self) -> None:
        others = sorted(job for job in self.jobs if job != "release-gates")
        self.assertEqual(sorted(self.gate["needs"]), others, "a ci.yml job is not listed under release-gates.needs")

    def test_release_gates_runs_after_failures_and_checks_results(self) -> None:
        self.assertEqual(self.gate["if"], "always()")
        steps = [step for step in self.gate["steps"] if "run" in step]
        self.assertEqual(len(steps), 1)
        self.assertEqual(steps[0]["run"].strip(), "python3 scripts/release_gates.py")
        self.assertEqual(steps[0]["env"], {"NEEDS": "${{ toJSON(needs) }}"})

    def test_no_job_can_be_skipped_by_a_condition(self) -> None:
        for job_id, job in self.jobs.items():
            if job_id != "release-gates":
                self.assertNotIn("if", job, f"{job_id}: a skipped job fails release-gates")

    def test_dataset_job_runs_the_ignored_full_tier_explicitly(self) -> None:
        job = self.jobs["datasets"]
        self.assertEqual(job["name"], "dataset-suites")
        run = " ".join(step.get("run", "") for step in job["steps"])
        self.assertIn("--profile ci-datasets", run)
        self.assertIn("--run-ignored all", run)

    def test_rust_jobs_do_not_share_a_cache_key(self) -> None:
        keys: dict[str, str] = {}
        for name in ("ci.yml", "language-parity.yml"):
            for job_id, job in workflow(name)["jobs"].items():
                for step in job.get("steps", []):
                    if str(step.get("uses", "")).startswith("Swatinem/rust-cache@"):
                        keys[f"{name}:{job_id}"] = step["with"]["shared-key"]
        self.assertGreaterEqual(len(keys), 4)
        self.assertEqual(len(set(keys.values())), len(keys), keys)


class RequiredCheckNameTests(unittest.TestCase):
    def test_cut_release_checks_are_job_names(self) -> None:
        script = (ROOT / "scripts" / "cut-release.sh").read_text(encoding="utf-8")
        block = re.search(r"^REQUIRED_CHECKS=\(\n(.*?)^\)", script, flags=re.M | re.S)
        self.assertIsNotNone(block)
        required = block.group(1).split()
        self.assertEqual(len(required), 10)
        names = {
            job.get("name", job_id)
            for file in ("ci.yml", "security.yml", "validate.yml")
            for job_id, job in workflow(file)["jobs"].items()
        }
        self.assertEqual([name for name in required if name not in names], [])


if __name__ == "__main__":
    unittest.main()
