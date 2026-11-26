#!/usr/bin/env python3
"""Package 5.4: scratch image, serde options, toolchain, deny, retention."""

from __future__ import annotations

import re
import tomllib
import unittest
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
WORKFLOWS = ROOT / ".github" / "workflows"


def load_workflow(name: str) -> dict:
    return yaml.safe_load((WORKFLOWS / name).read_text(encoding="utf-8"))


class EntrypointOptionsTests(unittest.TestCase):
    """A-14: entry script must not sed-parse tokens."""

    def test_entry_script_does_not_sed_options(self) -> None:
        script = (ROOT / "scripts" / "klar-entry.sh").read_text(encoding="utf-8")
        self.assertNotIn("options.json", script)
        self.assertNotIn("sed ", script)
        self.assertIn("exec /usr/local/bin/klar", script)


class DockerfileHardeningTests(unittest.TestCase):
    """V-13: scratch/non-root, labels, binary healthcheck."""

    def test_both_dockerfiles_are_scratch_non_root(self) -> None:
        for rel in ("Dockerfile", "docker/Dockerfile"):
            text = (ROOT / rel).read_text(encoding="utf-8")
            with self.subTest(file=rel):
                self.assertIsNotNone(re.search(r"(?m)^FROM scratch\s*$", text), text)
                self.assertIn("USER 65532:65532", text)
                self.assertIn("org.opencontainers.image.title", text)
                self.assertIn("HEALTHCHECK", text)
                self.assertIn('CMD ["/usr/local/bin/klar", "healthcheck"', text)
                final = text.rsplit("FROM scratch", 1)[-1]
                self.assertNotIn("debian:", final)
                self.assertNotIn("bash -c", final)


class ToolchainAndDenyTests(unittest.TestCase):
    """V-12 / V-14: raised MSRV, no blanket cargo ignores, unsound deny."""

    def test_msrv_is_at_least_1_91(self) -> None:
        msrv = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["package"]["rust-version"]
        major, minor, *_ = (int(part) for part in msrv.split("."))
        self.assertGreaterEqual((major, minor), (1, 91), msrv)
        channel = (ROOT / "rust-toolchain.toml").read_text(encoding="utf-8")
        self.assertIn(f'channel = "{msrv}"', channel)

    def test_cargo_toml_has_no_exact_msrv_pins(self) -> None:
        cargo = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
        self.assertNotIn("=2.5.9", cargo)
        self.assertNotIn("=1.1.0", cargo)
        self.assertNotIn("<1.27", cargo)
        self.assertIn('package = "serde_norway"', cargo)
        self.assertIn("default-features = false", cargo)

    def test_dependabot_drops_blanket_cargo_ignores_on_main(self) -> None:
        updates = yaml.safe_load((ROOT / ".github" / "dependabot.yml").read_text(encoding="utf-8"))["updates"]
        main_cargo = next(
            entry
            for entry in updates
            if entry["package-ecosystem"] == "cargo" and entry["target-branch"] == "main"
        )
        self.assertNotIn("ignore", main_cargo)

    def test_deny_refuses_unsound_crates(self) -> None:
        deny = tomllib.loads((ROOT / "deny.toml").read_text(encoding="utf-8"))
        self.assertEqual(deny["advisories"]["unsound"], "all")
        ignored = {item["id"] if isinstance(item, dict) else item for item in deny["advisories"].get("ignore", [])}
        self.assertNotIn("RUSTSEC-2024-0449", ignored)


class RetentionAndSbomTests(unittest.TestCase):
    """V-14 / V-15: image SBOM and retention job."""

    def test_release_sbom_targets_the_image(self) -> None:
        release = load_workflow("build.yml")["jobs"]["release"]
        sbom_steps = [
            step
            for step in release["steps"]
            if str(step.get("uses", "")).startswith("anchore/sbom-action@")
        ]
        self.assertEqual(len(sbom_steps), 1)
        self.assertIn("image", sbom_steps[0]["with"])
        self.assertNotIn("path", sbom_steps[0]["with"])

    def test_retention_workflow_prunes_prereleases_and_packages(self) -> None:
        jobs = load_workflow("retention.yml")["jobs"]
        self.assertIn("prune", jobs)
        runs = "\n".join(str(step.get("run", "")) for step in jobs["prune"]["steps"])
        self.assertIn("prerelease", runs)
        self.assertIn("packages/container", runs)


if __name__ == "__main__":
    unittest.main()
