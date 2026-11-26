#!/usr/bin/env python3
"""Supply chain and token scope of the workflows: pins, permissions, the MSRV toolchain."""

from __future__ import annotations

import re
import tomllib
import unittest
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
GITHUB = ROOT / ".github"
WORKFLOWS = sorted((GITHUB / "workflows").glob("*.yml"))

USES = re.compile(r"^\s*(?:- )?uses:\s*(\S+)(?:\s+#\s*(\S+))?\s*$")
PINNED = re.compile(r"^[\w.-]+/[\w./-]+@[0-9a-f]{40}$")
DIGEST = re.compile(r"^[\w./-]+:[\w.-]+@sha256:[0-9a-f]{64}$")
READ_ONLY = {"read", "none"}


def load(path: Path) -> dict:
    return yaml.safe_load(path.read_text(encoding="utf-8"))


def uses_lines() -> list[tuple[str, str, str | None]]:
    """(where, target, trailing comment) of every `uses:` in the workflows."""
    found: list[tuple[str, str, str | None]] = []
    for path in WORKFLOWS:
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            match = USES.match(line)
            if match:
                found.append((f"{path.name}:{number}", match.group(1), match.group(2)))
    return found


def steps() -> list[tuple[str, dict]]:
    return [
        (f"{path.name}:{job_id}", step)
        for path in WORKFLOWS
        for job_id, job in load(path)["jobs"].items()
        for step in job.get("steps", [])
    ]


class PinTests(unittest.TestCase):
    def test_every_action_is_pinned_by_commit_with_its_version_noted(self) -> None:
        remote = [item for item in uses_lines() if not item[1].startswith("./")]
        self.assertGreater(len(remote), 40)
        unpinned = [f"{where}: {target}" for where, target, _ in remote if not PINNED.match(target)]
        self.assertEqual(unpinned, [])
        # Dependabot reads the comment to tell which release the commit is.
        bare = [f"{where}: {target}" for where, target, comment in remote if not comment]
        self.assertEqual(bare, [])

    def test_one_action_has_one_commit_everywhere(self) -> None:
        commits: dict[str, set[str]] = {}
        for _, target, _ in uses_lines():
            if not target.startswith("./"):
                action, _, commit = target.partition("@")
                commits.setdefault("/".join(action.split("/")[:2]), set()).add(commit)
        self.assertEqual({action: sorted(found) for action, found in commits.items() if len(found) > 1}, {})

    def test_container_images_are_pinned_by_digest(self) -> None:
        images = [
            (f"{path.name}:{job_id}", job["container"]["image"] if isinstance(job["container"], dict) else job["container"])
            for path in WORKFLOWS
            for job_id, job in load(path)["jobs"].items()
            if "container" in job
        ]
        self.assertTrue(images)
        self.assertEqual([item for item in images if not DIGEST.match(item[1])], [])

    def test_the_toolchain_is_the_msrv_wherever_rust_is_installed(self) -> None:
        msrv = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"]["rust-version"]
        installs = [(where, step) for where, step in steps() if str(step.get("uses", "")).startswith("dtolnay/rust-toolchain@")]
        self.assertGreater(len(installs), 5)
        for where, step in installs:
            # The commit is one of the action's master branch, which takes the version as input.
            self.assertEqual((step.get("with") or {}).get("toolchain"), msrv, where)

    def test_dependabot_may_update_every_action(self) -> None:
        entries = [
            entry
            for entry in load(GITHUB / "dependabot.yml")["updates"]
            if entry["package-ecosystem"] == "github-actions"
        ]
        self.assertEqual(sorted(entry["target-branch"] for entry in entries), ["main", "staging"])
        for entry in entries:
            self.assertNotIn("ignore", entry, entry["target-branch"])


class PermissionTests(unittest.TestCase):
    def test_no_workflow_grants_write_access_to_all_its_jobs(self) -> None:
        for path in WORKFLOWS:
            with self.subTest(workflow=path.name):
                granted = load(path).get("permissions")
                self.assertIsNotNone(granted, "without a permissions block the token takes the repository default")
                self.assertEqual({scope: level for scope, level in granted.items() if level not in READ_ONLY}, {})

    def test_the_build_job_cannot_write_to_the_repository(self) -> None:
        jobs = load(GITHUB / "workflows" / "build.yml")["jobs"]
        # build only loads/scans locally; docker-publish is the sole packages writer.
        self.assertEqual(jobs["build"]["permissions"], {"contents": "read", "packages": "read"})
        self.assertEqual(jobs["docker-publish"]["permissions"], {"contents": "read", "packages": "write"})
        writers = sorted(job_id for job_id, job in jobs.items() if job["permissions"].get("contents") == "write")
        self.assertEqual(writers, ["release"])

    def test_checkouts_keep_no_credentials_unless_the_job_pushes(self) -> None:
        keeps = sorted(
            where
            for where, step in steps()
            if str(step.get("uses", "")).startswith("actions/checkout@")
            and (step.get("with") or {}).get("persist-credentials") is not False
        )
        self.assertEqual(
            keeps,
            ["build.yml:release", "release.yml:cut", "release.yml:land", "staging.yml:tag"],
        )

    def test_no_run_block_expands_a_context_that_a_caller_or_branch_name_controls(self) -> None:
        risky = re.compile(r"\$\{\{\s*(inputs\.nextest_profile|github\.(base_ref|head_ref)|github\.event\.[\w.]*(title|body|message))")
        found = [where for where, step in steps() if risky.search(str(step.get("run", "")))]
        self.assertEqual(found, [])


if __name__ == "__main__":
    unittest.main()
