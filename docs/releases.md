# Releases

[Deutsch](releases.md) · [English](en/releases.md)

Changelogs und GitHub Releases kommen von [git-cliff](https://git-cliff.org/) und [Conventional Commits](https://www.conventionalcommits.org/).

## Commit-Format

```
<type>(optionaler scope): <Beschreibung>

feat: französische Zahlwörter
fix(lock): „mach sie an“ bleibt einschalten
docs: Parse-API beschreiben
ci: rustc pro Target cachen
chore(release): prepare for v0.2.0
```

| Typ | Changelog-Gruppe | Semver |
|-----|------------------|--------|
| `feat` | Features | minor |
| `fix` | Bug Fixes | patch |
| `perf` | Performance | patch |
| `feat!` / `BREAKING CHANGE:` | Features + breaking | major |
| `docs`, `refactor`, `test`, `ci`, `chore` | stehen drin, kein Bump | — |

`chore(deps*)` und `chore(release)` fehlen im Changelog.

## Release schneiden

**Actions → Release → Run workflow.** `auto` (aus Commits seit dem letzten Tag) oder `patch` / `minor` / `major`.

Der Job:

1. holt die nächste Version von git-cliff
2. schreibt sie nach `Cargo.toml`, `config.yaml`, `addon/config.yaml` und das HA-Manifest
3. erzeugt `CHANGELOG.md` neu
4. committet `chore(release): prepare for vX.Y.Z` und pusht Tag `vX.Y.Z`

Danach ruft der Job **Build** auf (ein Tag-Push mit `GITHUB_TOKEN` startet keinen zweiten Workflow). Build erzeugt linux-x86_64, linux-aarch64 und linux-armv7 und hängt die Tarballs an. Der Release-Text ist der letzte git-cliff-Abschnitt.

Ein Tag von deinem Rechner startet Build weiter selbst: `git tag v0.2.0 && git push origin v0.2.0`.

## Changelog lokal

```bash
git cliff -o CHANGELOG.md
git cliff --unreleased
git cliff --bumped-version
```
