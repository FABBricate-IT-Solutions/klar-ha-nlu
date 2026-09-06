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

Language and feature work lands on **`staging` first**, not `main`. `staging` and `main` are **protected**: open a PR/MR, do not push the branch directly.

Every merge to `staging` runs **Staging** (same quality + security jobs as Release, not language-parity / full_home). It then:

1. Tags `{CalVer}-staging.{sha7}` from `Cargo.toml` + the merge SHA (example: `2026.8.30-staging.a1b2c3d`)
2. Calls **Build** with `prerelease: true`
3. Publishes a GitHub **prerelease** (`prerelease: true`, `make_latest: false`)
4. Pushes Docker tags `{rc}` and **`staging`** — never `latest`

Stable cuts stay on **`main` + CalVer tags** as below.

You must create `staging` on GitHub once and protect it (Actions cannot always set this without admin):

- Require a pull request before merging (no direct push)
- Do not allow force-push or deletion
- Require the same PR checks as `main` (`test`, `clippy`, `rustfmt`, `web`, `release-gates`, `cargo-audit`, `cargo-deny`, `gitleaks`, `hassfest`, `hacs`)
- Optional: require one approval

Promote an RC by opening a PR **`staging` → `main`**. That merge cuts the next CalVer as today.

Home Assistant switch: [home-assistant.md](home-assistant.md) — Configure → **Release channel** (bundled GitHub download and default app URL).

## Cut a release

Every merge to `main` cuts the next `YYYY.M.PATCH` automatically. The Release workflow:

1. Computes the next CalVer
2. Writes it into `Cargo.toml`, `config.yaml`, `addon/config.yaml`, and the HA manifest
3. Regenerates `CHANGELOG.md`
4. Opens or updates `chore/release-YYYY.M.PATCH`, waits for the 10 required checks, merges with `gh pr merge` (no `--admin`), and tags
5. Calls **Build** in the same run (`workflow_call`)

If the push is already a `chore(release): prepare for YYYY.M.PATCH` land (or a merge of `chore/release-*`), the cut only tags. It never `git push`es a new commit to `main`.

**Actions → Release → Run workflow** remains for a manual override (empty = next version, or e.g. `2026.8.0`).

Build compiles linux-x86_64 and linux-aarch64 and attaches the tarballs to the GitHub Release. The release body is the latest git-cliff section.

A tag pushed from your machine still triggers Build on its own: `git tag 2026.8.0 && git push origin 2026.8.0`.

The cut uses only the job-scoped `GITHUB_TOKEN`. `main` and `staging` require the status checks (`test`, `clippy`, `rustfmt`, `web`, `release-gates`, `cargo-audit`, `cargo-deny`, `gitleaks`, `hassfest`, `hacs`) before any PR merge, including admins. The version bump lands through that same PR path so the required checks can go green first.

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
cargo nextest run
python3 -m unittest discover -s tests -p 'test_*.py'
cargo run --quiet -- lang validate packs
cargo run --quiet -- eval bench --repeat 8
cargo build --release
rg 'src/(parse\.rs|web\.rs|wyoming\.rs|lexicon\.rs|numbers\.rs)|parse_help|home_policy' docs README.md README.de.md
```

V2 cuts must ship the Rust engine and `custom_components/klar_nlu` together. `POST /api/parse` is gone.

The `rg` check keeps documentation and the module tree aligned. Matches are not automatically failures, but each one should be intentional and current.
