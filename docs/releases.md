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
| `feat!` / `BREAKING CHANGE:` | Breaking Changes (zuerst in den Notes) |
| `docs`, `refactor`, `test`, `ci`, `chore` | stehen drin, kein Versionswechsel |

`chore(deps*)` und `chore(release)` fehlen im Changelog.

## Staging (Release Candidates)

Sprach- und Feature-Arbeit landet zuerst auf **`staging`**, nicht auf `main`. Beide Branches sind durch GitHub-Rulesets geschützt, abgelegt in [`.github/protection/desired.yaml`](../.github/protection/desired.yaml). Der wöchentliche Workflow **protection-audit** (`scripts/audit_protection.py`) schlägt fehl, wenn die Live-Regeln von dieser Datei abweichen. Nicht direkt pushen; PR öffnen.

Soll-Regeln (sowohl `main` als auch `staging`):

- Pull Request vor dem Merge (kein Direkt-Push)
- Kein Force-Push, kein Löschen
- Pflicht-Checks (strict / Branch aktuell): `test`, `clippy`, `rustfmt`, `web`, `release-gates`, `cargo-audit`, `cargo-deny`, `gitleaks`, `hassfest`, `hacs`
- Approvals: **0** (menschliches Review ist optional; das Ruleset verlangt keines)

CalVer- und `v*`-Tags sind ebenfalls geschützt (kein Löschen / Force-Update). Immutable GitHub Releases bleiben an.

Jeder Merge auf `staging` startet **Staging** (Quality + Security mit nextest-Profil `ci-staging`: eigene Dataset-Suiten auf dem PR-Pfad; die volle Multi-Locale-Paritätsmatrix bleibt bei [language-parity.yml](../.github/workflows/language-parity.yml) und dem Profil `ci` auf `main`). Danach:

1. Tag `{CalVer}-staging.{sha7}` aus `Cargo.toml` plus Merge-SHA (Beispiel: `2026.11.3-staging.a1b2c3d`)
2. **Build** mit `prerelease: true`
3. GitHub-**Prerelease** (`prerelease: true`, `make_latest: false`)
4. Docker-Tags `{rc}` und **`staging`** — nie `latest`

Stabile Cuts bleiben **`main` + CalVer-Tags** wie unten.

RC nach Produktion: PR **`staging` → `main`**. Der Workflow **Promote** öffnet diesen PR wöchentlich (Montag 06:00 UTC) und per manueller Auslösung; er merget nicht. Der Merge schneidet danach die nächste CalVer wie bisher.

`workflow_dispatch` auf **Staging** veröffentlicht nur, wenn der Lauf auf dem Branch `staging` startet — andere Refs werden übersprungen.

Home-Assistant-Wechsel: [home-assistant.md](home-assistant.md) — Konfigurieren → **Release-Kanal** (GitHub-Download und Standard-App-URL).

## Release schneiden

Jeder Merge auf `main` mit fachlichem Inhalt schneidet automatisch die nächste `YYYY.M.PATCH`. Reine Abhängigkeits-Merges (Dependabot / `npm:` / `cargo:` / `docker:` / `ci:`) erzeugen keinen Release. Der Release-Workflow (`scripts/cut-release.sh`):

1. berechnet die nächste CalVer-Version
2. schreibt sie nach `Cargo.toml`, beide Add-on-Configs, HA-Manifest / `ENGINE_VERSION` und die Doc-Image-Pins auf dem Branch `chore/release-YYYY.M.PATCH`
3. erzeugt `CHANGELOG.md` neu
4. öffnet oder aktualisiert den Release-PR, wartet auf die 10 Required Checks und taggt die Branch-Spitze (noch kein Merge)
5. ruft **Build** auf: beide Architekturen, Container-Start, Stop per SIGTERM, Scan, danach Images, Manifest und GitHub Release
6. merget den Release-PR erst danach (`land`) — die Add-on-Version auf `main` erscheint nie vor ihren Images

Ist der Push schon ein Land von `chore(release): prepare for YYYY.M.PATCH` (oder ein Merge von `chore/release-*`), taggt der Cut nur, wenn der Tag noch fehlt. Er pusht keinen neuen Commit nach `main`.

**Actions → Release → Run workflow** bleibt für einen manuellen Override (Feld leer = nächste Version, oder z. B. `2026.8.0`).

Build erzeugt linux-x86_64 und linux-aarch64 und hängt die Tarballs an das GitHub-Release. Der Release-Text ist der letzte git-cliff-Abschnitt.

Ein Tag von deinem Rechner startet Build weiter selbst: `git tag 2026.8.0 && git push origin 2026.8.0`.

Der Cut öffnet den Release-PR mit einem **GitHub-App-Token** (`APP_CLIENT_ID` Variable, `APP_PRIVATE_KEY` Secret), damit die Required Checks ohne manuelle Actions-Freigabe laufen. Wenn `main` während des Wartens weiterläuft, wird das Changelog neu erzeugt, bevor getaggt wird. Ein Cut, dem nichts mehr zu veröffentlichen bleibt (HEAD schon getaggt, Version schon vergeben), endet mit Exit 0.

git-cliff ignoriert Staging-/RC-Tags (`ignore_tags`), damit jeder `feat`/`fix`-Commit zwischen zwei Stable-Tags im Changelog der Stable-Version steht.

## Lokales Changelog

```bash
python3 scripts/bump-version.py next
python3 scripts/bump-version.py --self-test
git cliff -o CHANGELOG.md
git cliff --unreleased
```

## Pre-Release-Checks

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

V2-Cuts müssen Rust-Engine und `custom_components/klar_nlu` zusammen ausliefern. `POST /api/parse` entfällt.

Der `rg`-Check hält Dokumentation und Modulbaum in Dec. Treffer sind nicht automatisch Fehler, aber jeder sollte absichtlich und aktuell sein.
