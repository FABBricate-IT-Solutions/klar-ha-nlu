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
| `feat!` / `BREAKING CHANGE:` | Features + breaking |
| `docs`, `refactor`, `test`, `ci`, `chore` | listed, no version change |

`chore(deps*)` and `chore(release)` are omitted from the changelog.

## Cut a release

Every merge to `main` cuts the next `YYYY.M.PATCH` automatically. The Release workflow:

1. Computes the next CalVer
2. Writes it into `Cargo.toml`, `config.yaml`, `addon/config.yaml`, and the HA manifest
3. Regenerates `CHANGELOG.md`
4. Commits `chore(release): prepare for YYYY.M.PATCH` on `main` and tags
5. Calls **Build** in the same run (`workflow_call`)

**Actions → Release → Run workflow** remains for a manual override (empty = next version, or e.g. `2026.8.0`).

Build compiles linux-x86_64, linux-aarch64, and linux-armv7 and attaches the tarballs to the GitHub Release. The release body is the latest git-cliff section.

A tag pushed from your machine still triggers Build on its own: `git tag 2026.8.0 && git push origin 2026.8.0`.

`github-actions[bot]` (or a `RELEASE_TOKEN`) needs a required-pull-request bypass on `main`, or the version commit fails.

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
