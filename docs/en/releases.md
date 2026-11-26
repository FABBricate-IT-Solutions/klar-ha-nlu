# Releases

[Deutsch](../releases.md) · [English](releases.md)

Versions follow [Home Assistant CalVer](https://developers.home-assistant.io/docs/versioning/): `YYYY.M.PATCH` (month not zero-padded). Changelogs come from [git-cliff](https://git-cliff.org/) and [Conventional Commits](https://www.conventionalcommits.org/).

Examples: `2026.8.0` is the first August 2026 cut, `2026.8.1` the next cut that month, `2026.9.0` the first in September.

## Commit format

```
<type>(optional scope): <description>

feat: add French number words
fix(lock): keep "mach sie an" as turn-on
docs: describe the parse API
ci: cache rustc by target
chore(release): prepare for 2026.8.0
```

| Type | Changelog group |
|------|-----------------|
| `feat` | Features |
| `fix` | Bug Fixes |
| `perf` | Performance |
| `feat!` / `BREAKING CHANGE:` | Breaking Changes (first in the notes) |
| `docs`, `refactor`, `test`, `ci`, `chore` | listed, no version change |

`chore(deps*)` and `chore(release)` are omitted from the changelog.

## Staging (release candidates)

Language and feature work lands on **`staging` first**, not `main`. Both branches are protected by GitHub rulesets checked in as [`.github/protection/desired.yaml`](../../.github/protection/desired.yaml). The weekly **protection-audit** workflow (`scripts/audit_protection.py`) fails when live rules drift from that file. Do not push either branch directly; open a PR.

Desired rules (both `main` and `staging`):

- Pull request before merge (no direct push)
- No force-push, no deletion
- Required status checks (strict / branch up to date): `test`, `clippy`, `rustfmt`, `web`, `release-gates`, `cargo-audit`, `cargo-deny`, `gitleaks`, `hassfest`, `hacs`
- Approvals: **0** (optional human review is fine; the ruleset does not require one)

CalVer and `v*` tags are also protected (no delete / force-update). Immutable GitHub Releases stay enabled.

Every merge to `staging` runs **Staging** (quality + security with nextest profile `ci-staging`: Klar’s own dataset suites on the PR path; the full multi-locale parity matrix stays in [language-parity.yml](../../.github/workflows/language-parity.yml) and the `ci` profile on `main`). It then:

1. Tags `{CalVer}-staging.{sha7}` from `Cargo.toml` + the merge SHA (example: `2026.11.3-staging.a1b2c3d`)
2. Calls **Build** with `prerelease: true`
3. Publishes a GitHub **prerelease** (`prerelease: true`, `make_latest: false`)
4. Pushes Docker tags `{rc}` and **`staging`** — never `latest`

Stable cuts stay on **`main` + CalVer tags** as below.

Promote an RC with a PR **`staging` → `main`**. The **Promote** workflow opens that PR weekly (Monday 06:00 UTC) and on manual dispatch; it does not merge. Merging then cuts the next CalVer as today.

`workflow_dispatch` on **Staging** only publishes when the run starts on the `staging` branch — other refs are skipped.

Home Assistant switch: [home-assistant.md](home-assistant.md) — Configure → **Release channel** (bundled GitHub download and default app URL).

## Cut a release

Every merge to `main` that carries product changes cuts the next `YYYY.M.PATCH` automatically. Pure dependency merges (Dependabot / `npm:` / `cargo:` / `docker:` / `ci:`) do not cut a release. The Release workflow (`scripts/cut-release.sh`):

1. Computes the next CalVer
2. Writes it into `Cargo.toml`, both add-on configs, the HA manifest / `ENGINE_VERSION`, and the doc image pins on branch `chore/release-YYYY.M.PATCH`
3. Regenerates `CHANGELOG.md`
4. Opens or updates the release PR, waits for the 10 required checks, and tags that branch tip (no merge yet)
5. Calls **Build**: both architectures, start each container, stop with SIGTERM, scan, then push images, manifest, and the GitHub Release
6. Merges the release PR only afterwards (`land`) — the add-on version on `main` never appears before its images

If the push is already a `chore(release): prepare for YYYY.M.PATCH` land (or a merge of `chore/release-*`), the cut only tags when the tag is still missing. It never `git push`es a new commit to `main`.

**Actions → Release → Run workflow** remains for a manual override (empty = next version, or e.g. `2026.8.0`).

Build compiles linux-x86_64 and linux-aarch64 and attaches the tarballs to the GitHub Release. The release body is the latest git-cliff section.

A tag pushed from your machine still triggers Build on its own: `git tag 2026.8.0 && git push origin 2026.8.0`.

The cut opens the release PR with a **GitHub App token** (`APP_CLIENT_ID` variable, `APP_PRIVATE_KEY` secret) so required checks run without a manual Actions approval. If `main` moves while the cut waits, the changelog is regenerated before tagging. A cut with nothing left to publish (HEAD already tagged, version already taken) exits 0.

git-cliff ignores staging/RC tags (`ignore_tags`) so every `feat`/`fix` commit between two stable tags appears in that stable release's changelog.

## Local changelog

```bash
python3 scripts/bump-version.py next
python3 scripts/bump-version.py --self-test
git cliff -o CHANGELOG.md
git cliff --unreleased
```

## Pre-Release Checks

```bash
cargo fmt --check
cargo check
cargo nextest run --locked
python3 -m unittest discover -s tests -p 'test_*.py'
cargo run --quiet -- lang validate packs
cargo run --quiet -- eval bench --repeat 8
cargo build --release
rg 'src/(parse\.rs|web\.rs|wyoming\.rs|lexicon\.rs|numbers\.rs)|parse_help|home_policy' docs README.md README.de.md
```

V2 cuts must ship the Rust engine and `custom_components/klar_nlu` together. `POST /api/parse` is gone.

The `rg` check keeps documentation and the module tree aligned. Matches are not automatically failures, but each one should be intentional and current.
