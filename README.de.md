# Klar NLU

<p align="center">
  <img src="https://raw.githubusercontent.com/FABBricate-IT-Solutions/klar-ha-nlu/main/docs/klar-logo.png" alt="Klar — Rust-Krabbe mit Klar!" width="280">
</p>

[English](README.md) · [Deutsch](README.de.md)

[![CI](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/ci.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/ci.yml)
[![Security](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/security.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/security.yml)
[![Build](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/build.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/build.yml)
[![Validate](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/validate.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/validate.yml)
[![HACS Custom](https://img.shields.io/badge/HACS-Custom-orange.svg)](https://github.com/hacs/integration)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Deterministische Sprachsteuerung für Home Assistant. Klar zerlegt gesprochene Sätze in HA-Intents — ohne Cloud, ohne Modellgewichte.

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
- Optionaler LLM-Fallback in Home Assistant für Smalltalk, sobald Klar keinen Hausbefehl sieht

Klar steuert Geräte selbst. Ein LLM kommt nur zum Zug, wenn nichts zugeordnet wurde.

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

## Home Assistant

HACS installiert die Conversation-Integration. Die Klar-Engine (Binary, Docker oder Add-on) läuft weiter getrennt.

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. Badge klicken, oder in HACS → Integrationen → ⋮ → Benutzerdefinierte Repositories `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` als **Integration** hinzufügen
2. **Klar NLU** herunterladen und Home Assistant neu starten
3. Klar-Engine starten
4. [![Open your Home Assistant instance and start setting up a new integration.](https://my.home-assistant.io/badges/config_flow_start.svg)](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu) (URL-Standard `http://127.0.0.1:10520`)
5. Assist-Pipeline: Conversation-Engine = **Klar NLU**
6. Optional in den Optionen einen Conversation-Agent für Smalltalk wählen

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

**Actions → Release → Run workflow** erhöht die Version, schreibt das Changelog und taggt `vX.Y.Z`. Der Build-Workflow hängt danach linux-x86_64-, linux-aarch64- und linux-armv7-Tarballs an das GitHub Release. Ein manuelles `git tag v0.1.0 && git push origin v0.1.0` geht weiter.

Dependabot öffnet wöchentlich PRs für Crates und Actions; `cargo-audit` und `cargo-deny` laufen bei jeder Änderung.

## Tests

```bash
cargo test -- --test-threads=1
```

Wohnung DE/EN und beide Familiensuiten müssen 100 % halten. Details in [docs/testing.md](docs/testing.md).

## Lizenz

[MIT](LICENSE) — Copyright 2026 FABBricate IT Solutions
