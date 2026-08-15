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
chore(release): prepare for v2026.8.0
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

**Actions → Release → Run workflow.** Leave the field empty for the next `YYYY.M.PATCH`, or set e.g. `2026.8.0`.

That job:

1. Computes the next CalVer (or uses the input)
2. Writes it into `Cargo.toml`, `config.yaml`, `addon/config.yaml`, and the HA manifest
3. Regenerates `CHANGELOG.md`
4. Commits `chore(release): prepare for vYYYY.M.PATCH` on `release/vYYYY.M.PATCH`
5. Opens a PR when the org allows Actions to do that — otherwise it waits for required checks and fast-forwards `main`
6. Tags `vYYYY.M.PATCH` and calls **Build** (`workflow_call`; a `GITHUB_TOKEN` tag push would not start another workflow)

Build compiles linux-x86_64, linux-aarch64, and linux-armv7 and attaches the tarballs to the GitHub Release. The release body is the latest git-cliff section.

A tag pushed from your machine still triggers Build on its own: `git tag v2026.8.0 && git push origin v2026.8.0`.

## Local changelog

```bash
python3 scripts/bump-version.py next
python3 scripts/bump-version.py --self-test
git cliff -o CHANGELOG.md
git cliff --unreleased
```
