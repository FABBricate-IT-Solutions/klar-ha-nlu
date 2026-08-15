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
chore(release): prepare for 2026.8.0
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
4. committet `chore(release): prepare for YYYY.M.PATCH` auf `release/YYYY.M.PATCH`
5. öffnet einen PR, wenn die Organisation Actions das erlaubt — sonst startet er CI selbst (ein `GITHUB_TOKEN`-Push löst keine Workflows aus), wartet auf die Pflicht-Checks, fast-forwardet `main` und taggt im selben Lauf
6. ruft **Build** auf (`workflow_call`)

Build erzeugt linux-x86_64, linux-aarch64 und linux-armv7 und hängt die Tarballs an das GitHub-Release. Der Release-Text ist der letzte git-cliff-Abschnitt.

Ein Tag von deinem Rechner startet Build weiter selbst: `git tag 2026.8.0 && git push origin 2026.8.0`.

## Changelog lokal

```bash
python3 scripts/bump-version.py next
python3 scripts/bump-version.py --self-test
git cliff -o CHANGELOG.md
git cliff --unreleased
```
