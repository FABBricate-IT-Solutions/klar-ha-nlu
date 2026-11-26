#!/usr/bin/env python3
"""Package 5.2: cliff ignore staging tags; empty cut exits 0; staging dispatch guard."""

from __future__ import annotations

import re
import shutil
import subprocess
import unittest
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github" / "workflows"
CLIFF = ROOT / "cliff.toml"


def load(name: str) -> dict:
    return yaml.safe_load((WORKFLOWS / name).read_text(encoding="utf-8"))


class CliffIgnoreStagingTagsTests(unittest.TestCase):
    """V-03: staging/RC tags must not swallow feat/fix commits between stables."""

    def test_cliff_toml_ignores_staging_and_rc_tags(self) -> None:
        text = CLIFF.read_text(encoding="utf-8")
        match = re.search(r'^ignore_tags\s*=\s*"(.*)"\s*$', text, re.M)
        self.assertIsNotNone(match)
        assert match is not None
        # Raw TOML source escapes the regex dot as \\. → one \ for git-cliff.
        self.assertEqual(match.group(1), r".*-(staging|rc)\\..*")
        self.assertIn("staging|rc", match.group(1))

    def test_feat_fix_between_stables_appear_when_staging_tags_ignored(self) -> None:
        if shutil.which("git-cliff") is None:
            self.skipTest("git-cliff not installed")

        # Synthetic next tag on current tip: previous compare must be a stable
        # CalVer (no -staging.), proving ignore_tags is active in cliff.toml.
        out = subprocess.check_output(
            ["git-cliff", "--tag", "2099.1.0"],
            cwd=ROOT,
            text=True,
        )
        heading = next(line for line in out.splitlines() if line.startswith("## [2099.1.0]"))
        self.assertNotIn("-staging.", heading)
        self.assertRegex(heading, r"compare/20\d{2}\.\d+\.\d+\.\.\.2099\.1\.0")

        section = out.split("## [2099.1.0]", 1)[1].split("\n## [", 1)[0]
        self.assertTrue(
            re.search(r"^- .*\(\[([0-9a-f]{7})", section, re.M),
            "expected commit bullets under the synthetic stable section",
        )


class EmptyReleaseExitTests(unittest.TestCase):
    """V-04: nothing to publish must exit 0 (dense-merge race)."""

    def test_cut_release_self_test(self) -> None:
        proc = subprocess.run(
            ["bash", "scripts/cut-release.sh", "--self-test"],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr + proc.stdout)
        self.assertIn("ok", proc.stdout)

    def test_nothing_to_release_helper_exits_zero(self) -> None:
        script = (ROOT / "scripts" / "cut-release.sh").read_text(encoding="utf-8")
        self.assertIn("nothing_to_release", script)
        self.assertIn("Tag ${tag} already exists — nothing to release", script)
        self.assertIn("Only dependency commits since the last stable tag", script)
        # The old hard-fail path must be gone.
        self.assertNotRegex(
            script,
            r'already exists — nothing to release" >&2\n\s*exit 1',
        )


class StagingDispatchGuardTests(unittest.TestCase):
    """V-10: workflow_dispatch must not publish a non-staging ref as :staging."""

    def test_every_staging_job_requires_staging_ref(self) -> None:
        jobs = load("staging.yml")["jobs"]
        for job_id, job in jobs.items():
            with self.subTest(job=job_id):
                condition = job.get("if", "")
                self.assertIn("github.ref == 'refs/heads/staging'", condition)

    def test_promote_is_scheduled_and_does_not_merge(self) -> None:
        promote = load("promote.yml")
        # PyYAML parses the workflow key `on` as boolean True.
        triggers = promote.get("on") or promote.get(True) or {}
        self.assertIn("schedule", triggers)
        cron = triggers["schedule"][0]["cron"]
        self.assertTrue(cron)
        proc = subprocess.run(
            ["bash", "scripts/promote-staging.sh", "--self-test"],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr + proc.stdout)

    def test_release_cut_uses_github_app_token(self) -> None:
        cut = load("release.yml")["jobs"]["cut"]
        uses = [step.get("uses", "") for step in cut["steps"]]
        self.assertTrue(
            any(u.startswith("actions/create-github-app-token@") for u in uses),
            uses,
        )
        env = next(
            step["env"]
            for step in cut["steps"]
            if step.get("name") == "Cut or reuse release"
        )
        self.assertIn("steps.app-token.outputs.token", str(env["GITHUB_TOKEN"]))


class ChangelogRefreshTests(unittest.TestCase):
    def test_cut_regenerates_changelog_when_main_moves(self) -> None:
        script = (ROOT / "scripts" / "cut-release.sh").read_text(encoding="utf-8")
        self.assertIn("sync_release_branch_with_main", script)
        self.assertIn("regenerating changelog", script)
        # BEHIND is no longer accepted as ready-to-tag.
        wait = script.split("wait_required_checks()")[1].split("\n}")[0]
        self.assertNotIn("BEHIND | UNSTABLE", wait)
        self.assertIn("BEHIND)", wait)
        self.assertIn("return 2", wait)


if __name__ == "__main__":
    unittest.main()
