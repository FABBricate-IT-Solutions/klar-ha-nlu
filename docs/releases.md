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

## Staging (Release Candidates)

Sprach- und Feature-Arbeit landet zuerst auf **`staging`**, nicht auf `main`. `staging` und `main` sind **geschützt**: per PR/MR mergen, nicht direkt pushen.

Jeder Merge auf `staging` startet **Staging** (dieselben Quality- und Security-Jobs wie Release, nicht language-parity / full_home). Danach:

1. Tag `{CalVer}-staging.{sha7}` aus `Cargo.toml` plus Merge-SHA (Beispiel: `2026.8.30-staging.a1b2c3d`)
2. **Build** mit `prerelease: true`
3. GitHub-**Prerelease** (`prerelease: true`, `make_latest: false`)
4. Docker-Tags `{rc}` und **`staging`** — nie `latest`

Stabile Cuts bleiben **`main` + CalVer-Tags** wie unten.

`staging` einmal auf GitHub anlegen und schützen (Actions kann das oft nicht ohne Admin):

- Pull Request vor dem Merge (kein Direkt-Push)
- Kein Force-Push, kein Löschen
- Dieselben Required Checks wie `main` (`test`, `clippy`, `rustfmt`, `web`, `release-gates`, `cargo-audit`, `cargo-deny`, `gitleaks`, `hassfest`, `hacs`)
- Optional: eine Approval

RC nach Produktion: PR **`staging` → `main`**. Dieser Merge schneidet die nächste CalVer wie bisher.

Home-Assistant-Wechsel: [home-assistant.md](home-assistant.md) (mitgelieferter **Release-Kanal** oder Add-on **Klar NLU (Staging)**).

## Release schneiden

Jeder Merge auf `main` schneidet automatisch die nächste `YYYY.M.PATCH`. Der Release-Workflow:

1. berechnet die nächste CalVer-Version
2. schreibt sie nach `Cargo.toml`, `config.yaml`, `addon/config.yaml` und das HA-Manifest
3. erzeugt `CHANGELOG.md` neu
4. committet `chore(release): prepare for YYYY.M.PATCH` auf `main` und taggt
5. ruft **Build** im selben Lauf auf (`workflow_call`)

**Actions → Release → Run workflow** bleibt für einen manuellen Override (Feld leer = nächste Version, oder z. B. `2026.8.0`).

Build erzeugt linux-x86_64, linux-aarch64 und linux-armv7 und hängt die Tarballs an das GitHub-Release. Der Release-Text ist der letzte git-cliff-Abschnitt.

Ein Tag von deinem Rechner startet Build weiter selbst: `git tag 2026.8.0 && git push origin 2026.8.0`.

Der Cut nutzt nur den kurzlebigen `GITHUB_TOKEN` des Jobs. `main` verbietet Löschen und Force-Push; Required Checks gelten nicht für den Version-Commit (sonst bräuchte es ein dauerhaftes Admin-PAT). PR-CI bleibt der Qualitätsfilter vor dem Merge.

## Changelog lokal

```bash
python3 scripts/bump-version.py next
python3 scripts/bump-version.py --self-test
git cliff -o CHANGELOG.md
git cliff --unreleased
```

## Vor dem Release prüfen

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

V2-Cuts müssen Rust-Engine und `custom_components/klar_nlu` zusammen ausliefern. `POST /api/parse` entfällt.

Der `rg`-Check hält Dokumentation und Modulbaum synchron. Treffer sind nicht automatisch Fehler, müssen aber bewusst aktuell sein.
