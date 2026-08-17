# Klar NLU

<div align="center">

![Klar](https://raw.githubusercontent.com/FABBricate-IT-Solutions/klar-ha-nlu/main/docs/klar-logo-sm.png)

</div>

[English](README.md) · [Deutsch](README.de.md)

[![CI](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/ci.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/ci.yml)
[![Security](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/security.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/security.yml)
[![Build](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/build.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/build.yml)
[![Validate](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/validate.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/validate.yml)
[![HACS Custom](https://img.shields.io/badge/HACS-Custom-orange.svg)](https://github.com/hacs/integration)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Deterministische Sprachsteuerung für Home Assistant. Klar zerlegt gesprochene Sätze in HA-Intents — ohne Cloud, ohne Modellgewichte.

**Breaking V2:** HTTP-Parse läuft nur noch über `POST /api/v2/parse` (`schema_version: "2.0"`). Rust-Engine und Home-Assistant-Integration zusammen aktualisieren. `POST /api/parse` entfällt. Vorhandene `klar_nlu.json`-Overlays laden weiter; `klar migrate import --from` prüft Konflikte, Waisen und unsichere Settings, bevor V2 geschrieben wird.

Deutsch und Englisch sind eingebaut und laufen parallel. Weitere Sprachen kommen als Paket dazu, nicht als Sonderfälle in der Engine.

```
„Licht im Wohnzimmer an“     →  HassTurnOn   area=wohnzimmer  domain=light
„Set the garage door to 40%“ →  HassSetPosition  cover.garage_door
```

## Was Klar macht

- Hausbefehle: Licht, Klima, Cover, Schloss, Lüfter, Medien, Timer, Listen, Szenen
- Mehrere Klauseln (`Wohnzimmer und Küche`, `mach das Licht aus und die Heizung auf 21`)
- Rückfragen, wenn ein Gerät nicht eindeutig ist
- Sitzung: „mach sie aus“ bezieht sich auf das letzte Ziel
- Persönlichkeiten in Home Assistant (Butler, Grantig, Pirat, …) — Assist, Sprechformel und optionale LLM-Umformulierung folgen derselben Auswahl
- Optionaler LLM-Fallback in Home Assistant für Smalltalk, sobald Klar keinen Hausbefehl sieht
- Optionale LLM-Verfeinerung fertiger NLU-Antworten (standardmäßig aus; Gerätesteuerung bleibt bei Klar)

Klar steuert Geräte selbst. Ein LLM redet oder formuliert um — Haus-Intents führt es nicht aus.

## Aufbau

Die Rust-Engine ist in klare Schichten geteilt:

- `src/types/` definiert Intents, `ParseOutcome`, Settings und den Home-Graph.
- `src/nlu/` rankt Kandidaten und wendet Confidence-, OOD-, Confirm- und Multi-Intent-Policy an.
- `src/parse/` tokenisiert, erkennt Aktionen, löst Ziele auf und füllt Slots.
- `src/home/` lädt Home-Assistant-Registries, Overlays, Expose-Daten, Rollen und die eingebaute Musterwohnung.
- `src/eval/` und `src/migrate/` liefern Held-out-Scorecards und den einmaligen V1-Overlay-Import.
- `src/io/` enthält HTTP, Wyoming, gemeinsamen Runtime-State, Token-Handling und Reloads.

Zur Laufzeit baut Klar einen effektiven Home-Graph aus HA-Config und beschreibbarem Datenverzeichnis. Parse-API und Wyoming-Server teilen diesen Graphen und denselben Session-Store, deshalb verhalten sich Follow-ups auf beiden Schnittstellen gleich.

## Schnellstart

Rust 1.85+, dann:

```bash
cargo run --release -- --config-dir /pfad/zur/ha-config --http 0.0.0.0:10520 --wyoming 0.0.0.0:10500
```

Ohne HA-Config nutzt Klar eine eingebaute Musterwohnung.

| Port  | Dienst              |
|-------|---------------------|
| 10520 | Web-UI und Parse-API |
| 10500 | Wyoming Intent      |

UI: <http://127.0.0.1:10520>

Nützliche Checks bei der Entwicklung:

```bash
cargo fmt --check
cargo check
cargo nextest run
cargo run --quiet -- lang validate packs
cargo run --quiet -- eval bench --repeat 8
cargo build --release
```

## Home Assistant

HACS installiert die Conversation-Integration. Beim Einrichten **Mitgelieferte Engine starten** wählen — die Integration lädt das passende GitHub-Release und startet Klar neben Home Assistant. HACS selbst kann den Rust-Prozess nicht starten.

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. Badge klicken, oder in HACS → Integrationen → ⋮ → Benutzerdefinierte Repositories `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` als **Integration** hinzufügen
2. **Klar NLU** herunterladen und Home Assistant neu starten
3. [![Open your Home Assistant instance and start setting up a new integration.](https://my.home-assistant.io/badges/config_flow_start.svg)](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu) und **Mitgelieferte Engine starten** behalten
4. Assist-Pipeline: Conversation-Engine = **Klar NLU**
5. Optional in den Optionen einen Conversation-Agent für Smalltalk wählen. Dort auch: Persönlichkeit, und **NLU-Antworten vom LLM verfeinern**, wenn der Agent Bestätigungen umformulieren soll

Läuft Klar schon, **Bereits laufende Engine verwenden** wählen und die URL setzen.

### Add-on (Home Assistant OS)

[![Open your Home Assistant instance and show the add add-on repository dialog with a specific repository URL pre-filled.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2FFABBricate-IT-Solutions%2Fklar-ha-nlu)

Einstellungen → Add-ons → ⋮ → Repositories → `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` → **Klar NLU** installieren. Integration mit URL `http://klar-nlu:10520`.

### Docker

```bash
docker run --rm --network host \
  -v /pfad/zur/homeassistant/config:/config:ro \
  ghcr.io/fabbricate-it-solutions/klar-nlu:0.1.0
```

Integrations-URL: `http://127.0.0.1:10520`. Es gibt auch Images pro Arch (`klar-nlu-amd64`, `klar-nlu-aarch64`, `klar-nlu-armv7`).

Manuell: `custom_components/klar_nlu` nach `config/custom_components/klar_nlu` kopieren, dann neu starten.

Ausführlich: [docs/home-assistant.md](docs/home-assistant.md) · [English](docs/en/home-assistant.md)

## Dokumentation

| Thema | English | Deutsch |
|-------|---------|---------|
| Architektur | [docs/en/architecture.md](docs/en/architecture.md) | [docs/architecture.md](docs/architecture.md) |
| HTTP- und Wyoming-API | [docs/en/api.md](docs/en/api.md) | [docs/api.md](docs/api.md) |
| Home Assistant | [docs/en/home-assistant.md](docs/en/home-assistant.md) | [docs/home-assistant.md](docs/home-assistant.md) |
| Sprachen erweitern | [docs/en/languages.md](docs/en/languages.md) | [docs/languages.md](docs/languages.md) |
| Tests | [docs/en/testing.md](docs/en/testing.md) | [docs/testing.md](docs/testing.md) |
| Releases | [docs/en/releases.md](docs/en/releases.md) | [docs/releases.md](docs/releases.md) |

## Releases

Changelogs kommen von [git-cliff](https://git-cliff.org/) und [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, …). Siehe [CHANGELOG.md](CHANGELOG.md).

Jeder Merge auf `main` erhöht die [Home-Assistant-CalVer](https://developers.home-assistant.io/docs/versioning/) (`YYYY.M.PATCH`), schreibt das Changelog und taggt. Der Build-Workflow hängt danach linux-x86_64-, linux-aarch64- und linux-armv7-Tarballs an das GitHub Release. **Actions → Release → Run workflow** bleibt für einen manuellen Override. Ein manuelles `git tag 2026.8.0 && git push origin 2026.8.0` geht weiter.

Dependabot öffnet wöchentlich PRs für Crates und Actions; `cargo-audit` und `cargo-deny` laufen bei jeder Änderung.

## Tests

```bash
cargo nextest run
```

Die verbindlichen Voice-Suite-Schwellen stehen in
[docs/testing.md](docs/testing.md). 100 % bleibt das Ziel, die aktuell
blockierenden Schwellen der generierten Vergleichssuiten liegen jedoch
darunter.

## Lizenz

[MIT](LICENSE) — Copyright 2026 FABBricate IT Solutions
