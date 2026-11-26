#!/usr/bin/env python3
"""Package 5.3: desired protection schema and live-vs-desired comparison (V-06)."""

from __future__ import annotations

import copy
import importlib.util
import unittest
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
DESIRED = ROOT / ".github" / "protection" / "desired.yaml"
SCRIPT = ROOT / "scripts" / "audit_protection.py"
WORKFLOW = ROOT / ".github" / "workflows" / "protection-audit.yml"


def load_audit():
    spec = importlib.util.spec_from_file_location("audit_protection", SCRIPT)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


audit = load_audit()


class DesiredSchemaTests(unittest.TestCase):
    def test_checked_in_desired_config_is_valid(self) -> None:
        data = audit.load_desired(DESIRED)
        expanded = audit.expand_desired(data)
        self.assertTrue(expanded["immutable_releases"]["enabled"])
        names = {item["name"] for item in expanded["rulesets"]}
        self.assertEqual(names, {"Protect main", "Protect staging", "Protect version tags"})

    def test_calver_tag_pattern_20_star_is_required(self) -> None:
        data = yaml.safe_load(DESIRED.read_text(encoding="utf-8"))
        tag = next(item for item in data["rulesets"] if item["target"] == "tag")
        self.assertIn("refs/tags/20*", tag["include"])
        broken = copy.deepcopy(data)
        for item in broken["rulesets"]:
            if item["target"] == "tag":
                item["include"] = ["refs/tags/v*"]
        with self.assertRaises(audit.DesiredConfigError):
            audit.validate_desired(broken)

    def test_branch_rulesets_require_pr_and_strict_checks(self) -> None:
        data = audit.load_desired(DESIRED)
        expanded = audit.expand_desired(data)
        for name in ("Protect main", "Protect staging"):
            ruleset = next(item for item in expanded["rulesets"] if item["name"] == name)
            types = {rule["type"]: rule for rule in ruleset["rules"]}
            self.assertIn("pull_request", types)
            status = types["required_status_checks"]["parameters"]
            self.assertTrue(status["strict_required_status_checks_policy"])
            contexts = [item["context"] for item in status["required_status_checks"]]
            self.assertEqual(contexts, sorted(data["required_status_checks"]))

    def test_reject_duplicate_ruleset_names(self) -> None:
        data = yaml.safe_load(DESIRED.read_text(encoding="utf-8"))
        data["rulesets"].append(copy.deepcopy(data["rulesets"][0]))
        with self.assertRaises(audit.DesiredConfigError):
            audit.validate_desired(data)


class CompareTests(unittest.TestCase):
    def setUp(self) -> None:
        self.desired = audit.expand_desired(audit.load_desired(DESIRED))

    def test_identity_matches(self) -> None:
        self.assertEqual(audit.compare_protection(self.desired, copy.deepcopy(self.desired)), [])

    def test_immutable_releases_drift(self) -> None:
        live = copy.deepcopy(self.desired)
        live["immutable_releases"]["enabled"] = False
        drifts = audit.compare_protection(self.desired, live)
        self.assertTrue(any("immutable_releases" in line for line in drifts))

    def test_missing_calver_include_drifts(self) -> None:
        live = copy.deepcopy(self.desired)
        tag = next(item for item in live["rulesets"] if item["name"] == "Protect version tags")
        tag["include"] = ["refs/tags/v*"]
        drifts = audit.compare_protection(self.desired, live)
        self.assertTrue(any("Protect version tags" in line for line in drifts))

    def test_normalize_live_strips_noise_fields(self) -> None:
        live_rulesets = [
            {
                "name": "Protect version tags",
                "target": "tag",
                "enforcement": "active",
                "conditions": {"ref_name": {"include": ["refs/tags/20*", "refs/tags/v*"], "exclude": []}},
                "rules": [
                    {"type": "update"},
                    {"type": "deletion"},
                    {"type": "non_fast_forward"},
                ],
                "bypass_actors": [],
            }
        ]
        normalized = audit.normalize_live(live_rulesets, True)
        tag = normalized["rulesets"][0]
        self.assertEqual(tag["include"], ["refs/tags/20*", "refs/tags/v*"])
        self.assertEqual(
            [rule["type"] for rule in tag["rules"]],
            ["deletion", "non_fast_forward", "update"],
        )


class WorkflowTests(unittest.TestCase):
    def test_weekly_audit_workflow_exists_and_is_read_only(self) -> None:
        workflow = yaml.safe_load(WORKFLOW.read_text(encoding="utf-8"))
        # PyYAML turns `on` into True.
        triggers = workflow[True]
        self.assertIn("schedule", triggers)
        self.assertIn("workflow_dispatch", triggers)
        self.assertEqual(workflow["permissions"], {"contents": "read"})
        job = workflow["jobs"]["audit"]
        run = " ".join(step.get("run", "") for step in job["steps"])
        self.assertIn("scripts/audit_protection.py", run)
        self.assertIn("--require-live", run)


if __name__ == "__main__":
    unittest.main()
