# Security

Report vulnerabilities **privately** via
[GitHub Security Advisories](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/security/advisories/new)
on this repository, or to the maintainers of
[FABBricate IT Solutions](https://github.com/FABBricate-IT-Solutions).

There is no public security email. Do not open a public issue for an unfixed vulnerability.

## Supported versions

The current CalVer on `main` and the latest GitHub Release (this tree: **2026.8.30**) are supported. Pre-CalVer tags such as `0.1.0` are V1-only and must not be used with the V2 integration (`POST /api/v2/parse`).

Household token and bundle notes: [docs/en/troubleshooting.md](docs/en/troubleshooting.md).

This repository runs:

- `cargo-audit` on every pull request and weekly
- `cargo-deny` for advisories, licenses, and crate sources
- `gitleaks` on every pull request (HA tokens / `KLAR_TOKEN` must not land in packs or tests)
- Semgrep custom rules that forbid restoring a de+en-only default catalog
- Weekly CodeQL (Rust, generated packs ignored) and Trivy on the GHCR `:latest` image
- CycloneDX SBOM attached to GitHub Releases
- Dependabot version updates (Cargo, npm, Docker, and GitHub Actions)
- GitHub Dependabot security updates (enabled on the repo)
- Release on `main` waits for CI (nextest, clippy, pack freshness) and these security jobs
