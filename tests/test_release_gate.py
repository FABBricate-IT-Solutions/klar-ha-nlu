#!/usr/bin/env python3
"""Package 5.1: images and scan before the add-on version; release targets on every PR."""

from __future__ import annotations

import re
import unittest
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github" / "workflows"


def load(name: str) -> dict:
    return yaml.safe_load((WORKFLOWS / name).read_text(encoding="utf-8"))


def run_steps(job: dict) -> str:
    return "\n".join(str(step.get("run", "")) for step in job.get("steps", []))


class AddonVersionAfterImagesTests(unittest.TestCase):
    """V-02: Supervisor must not see a new add-on version before its GHCR images exist."""

    def test_release_workflow_lands_addon_only_after_publish(self) -> None:
        jobs = load("release.yml")["jobs"]
        self.assertIn("land", jobs)
        self.assertEqual(jobs["publish"]["needs"], "cut")
        land_needs = jobs["land"]["needs"]
        self.assertIn("publish", land_needs)
        self.assertIn("cut", land_needs)
        self.assertIn("needs.publish.result == 'success'", jobs["land"].get("if", ""))

    def test_cut_script_tags_before_merge_and_lands_afterwards(self) -> None:
        script = (ROOT / "scripts" / "cut-release.sh").read_text(encoding="utf-8")
        self.assertIn("prepare_via_pr", script)
        self.assertIn("land_addon", script)
        self.assertIn("--land-addon", script)
        # The prepare path must tag without merging the release PR.
        prepare = script.split("prepare_via_pr()")[1].split("\n}")[0]
        self.assertIn("tag_and_exit", prepare)
        self.assertNotIn("gh pr merge", prepare)
        self.assertIn("gh pr merge", script.split("land_addon()")[1].split("\n}")[0])
        self.assertIn('prepare_via_pr "$tag"', script)
        self.assertNotIn('land_via_pr "$tag"', script)


class NoPartialPublishTests(unittest.TestCase):
    """V-05: do not publish a GitHub Release or push images until both arches and the scan succeed."""

    def test_build_matrix_fails_fast(self) -> None:
        strategy = load("build.yml")["jobs"]["build"]["strategy"]
        self.assertTrue(strategy["fail-fast"])

    def test_matrix_builds_and_smokes_before_any_push(self) -> None:
        build = load("build.yml")["jobs"]["build"]
        names = [step.get("name", "") for step in build["steps"]]
        self.assertIn("Build container", names)
        self.assertIn("Smoke SIGTERM", names)
        self.assertIn("Scan local image", names)
        self.assertNotIn("Push container", names)
        runs = run_steps(build)
        self.assertIn("docker-smoke.sh", runs)
        self.assertIn("--load", runs)
        self.assertNotIn("--push", runs)

    def test_images_are_pushed_only_after_every_arch_succeeds(self) -> None:
        jobs = load("build.yml")["jobs"]
        self.assertIn("docker-publish", jobs)
        self.assertEqual(jobs["docker-publish"]["needs"], "build")
        publish_runs = run_steps(jobs["docker-publish"])
        self.assertIn("docker load", publish_runs)
        self.assertIn("docker push", publish_runs)
        self.assertIn("imagetools create", publish_runs)

    def test_github_release_waits_for_images_and_scan(self) -> None:
        jobs = load("build.yml")["jobs"]
        needs = jobs["release"]["needs"]
        self.assertIn("docker-publish", needs)
        self.assertIn("trivy", needs)
        self.assertEqual(jobs["trivy"]["needs"], "docker-publish")


class ReleaseTargetsOnEveryPrTests(unittest.TestCase):
    """V-09: musl release targets and both Dockerfiles build on every PR."""

    def test_ci_builds_both_musl_targets(self) -> None:
        job = load("ci.yml")["jobs"]["release-targets"]
        targets = {row["target"] for row in job["strategy"]["matrix"]["include"]}
        self.assertEqual(
            targets,
            {"x86_64-unknown-linux-musl", "aarch64-unknown-linux-musl"},
        )
        runs = run_steps(job)
        self.assertIn("cargo build --release --locked --target", runs)
        self.assertIn("docker/Dockerfile", runs)
        self.assertIn("docker-smoke.sh", runs)

    def test_ci_builds_the_root_dockerfile(self) -> None:
        job = load("ci.yml")["jobs"]["dockerfile-root"]
        runs = run_steps(job)
        self.assertIn("-f Dockerfile", runs)
        self.assertIn("docker-smoke.sh", runs)

    def test_release_gates_include_the_ship_jobs(self) -> None:
        gate = load("ci.yml")["jobs"]["release-gates"]
        self.assertIn("release-targets", gate["needs"])
        self.assertIn("dockerfile-root", gate["needs"])


class SmokeScriptTests(unittest.TestCase):
    def test_smoke_script_stops_with_sigterm(self) -> None:
        script = (ROOT / "scripts" / "docker-smoke.sh").read_text(encoding="utf-8")
        self.assertTrue(script.startswith("#!/"))
        self.assertIn("docker stop", script)
        self.assertIn("SIGTERM", script)
        self.assertRegex(script, re.compile(r"\b143\b"))


if __name__ == "__main__":
    unittest.main()
