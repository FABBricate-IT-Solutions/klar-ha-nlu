# Releases

[Deutsch](releases.md) · [English](en/releases.md)

Versionen folgen [Home Assistant CalVer](https://developers.home-assistant.io/docs/versioning/): `YYYY.M.PATCH` (Monat ohne führende Null). Changelogs kommen von [git-cliff](https://git-cliff.org/) und [Conventional Commits](https://www.conventionalcommits.org/).

Beispiele: `2026.8.0` ist die erste August-2026-Version, `2026.8.1` der nächste Schnitt im selben Monat, `2026.9.0` der erste im September.

## Commit-Format

```
<type>(optionaler scope): <Beschreibung>

feat: französische Zahlwörter
fix(lock): „mach sie an“ bleibt einschalten
docs: Parse-API beschreiben
ci: rustc pro Target cachen
chore(release): prepare for v2026.8.0
```

| Typ | Changelog-Gruppe |
|-----|------------------|
| `feat` | Features |
| `fix` | Bug Fixes |
| `perf` | Performance |
| `feat!` / `BREAKING CHANGE:` | Features + breaking |
| `docs`, `refactor`, `test`, `ci`, `chore` | stehen drin, kein Versionswechsel |

`chore(deps*)` und `chore(release)` fehlen im Changelog.

## Release schneiden

**Actions → Release → Run workflow.** Feld leer lassen für die nächste `YYYY.M.PATCH`, oder z. B. `2026.8.0` setzen.

Der Job:

1. berechnet die nächste CalVer-Version (oder nimmt die Eingabe)
2. schreibt sie nach `Cargo.toml`, `config.yaml`, `addon/config.yaml` und das HA-Manifest
3. erzeugt `CHANGELOG.md` neu
4. committet `chore(release): prepare for vYYYY.M.PATCH` und taggt `vYYYY.M.PATCH`

Danach ruft der Job **Build** auf (ein Tag-Push mit `GITHUB_TOKEN` startet keinen zweiten Workflow). Build erzeugt linux-x86_64, linux-aarch64 und linux-armv7 und hängt die Tarballs an. Der Release-Text ist der letzte git-cliff-Abschnitt.

Ein Tag von deinem Rechner startet Build weiter selbst: `git tag v2026.8.0 && git push origin v2026.8.0`.

## Changelog lokal

```bash
python3 scripts/bump-version.py next
python3 scripts/bump-version.py --self-test
git cliff -o CHANGELOG.md
git cliff --unreleased
```
