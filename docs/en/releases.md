# Releases

[Deutsch](../releases.md) · [English](releases.md)

Changelogs and GitHub Releases come from [git-cliff](https://git-cliff.org/) and [Conventional Commits](https://www.conventionalcommits.org/).

## Commit format

```
<type>(optional scope): <description>

feat: add French number words
fix(lock): keep "mach sie an" as turn-on
docs: describe the parse API
ci: cache rustc by target
chore(release): prepare for v0.2.0
```

| Type | Changelog group | Semver |
|------|-----------------|--------|
| `feat` | Features | minor |
| `fix` | Bug Fixes | patch |
| `perf` | Performance | patch |
| `feat!` / `BREAKING CHANGE:` | Features + breaking | major |
| `docs`, `refactor`, `test`, `ci`, `chore` | listed, no bump by default | — |

`chore(deps*)` and `chore(release)` are omitted from the changelog.

## Cut a release

**Actions → Release → Run workflow.** Pick `auto` (from commits since the last tag) or force `patch` / `minor` / `major`.

That job:

1. Asks git-cliff for the next version
2. Writes it into `Cargo.toml`, `config.yaml`, `addon/config.yaml`, and the HA manifest
3. Regenerates `CHANGELOG.md`
4. Commits `chore(release): prepare for vX.Y.Z` and pushes tag `vX.Y.Z`

That job then calls **Build** (a `GITHUB_TOKEN` tag push would not start another workflow). Build compiles linux-x86_64, linux-aarch64, and linux-armv7 and attaches the tarballs. The release body is the latest git-cliff section.

A tag pushed from your machine still triggers Build on its own: `git tag v0.2.0 && git push origin v0.2.0`.

## Local changelog

```bash
git cliff -o CHANGELOG.md
git cliff --unreleased
git cliff --bumped-version
```
