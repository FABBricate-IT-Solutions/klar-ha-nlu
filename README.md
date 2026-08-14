# Klar NLU

<p align="center">
  <img src="https://raw.githubusercontent.com/FABBricate-IT-Solutions/klar-ha-nlu/main/docs/klar-logo.png" alt="Klar — Rust crab saying Klar!" width="280">
</p>

[English](README.md) · [Deutsch](README.de.md)

[![CI](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/ci.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/ci.yml)
[![Security](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/security.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/security.yml)
[![Build](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/build.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/build.yml)
[![Validate](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/validate.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/validate.yml)
[![HACS Custom](https://img.shields.io/badge/HACS-Custom-orange.svg)](https://github.com/hacs/integration)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Deterministic voice control for Home Assistant. Klar turns spoken sentences into HA intents — no cloud, no model weights.

German and English ship in-tree and run side by side. Further languages are packs, not special cases in the engine.

```
“Licht im Wohnzimmer an”     →  HassTurnOn   area=wohnzimmer  domain=light
“Set the garage door to 40%” →  HassSetPosition  cover.garage_door
```

## What Klar does

- Home commands: lights, climate, covers, locks, fans, media, timers, lists, scenes
- Multiple clauses (`living room and kitchen`, `turn the lights off and set heat to 21`)
- Clarification when a device is ambiguous
- Session: “turn it off” refers to the last target
- Optional LLM fallback in Home Assistant for chit-chat once Klar sees no home command

Klar drives devices itself. An LLM only runs when nothing was matched.

## Quick start

Rust 1.85+, then:

```bash
cargo run --release -- --config-dir /path/to/ha-config --http 0.0.0.0:10520 --wyoming 0.0.0.0:10500
```

Without an HA config Klar uses a built-in sample home.

| Port  | Service                 |
|-------|-------------------------|
| 10520 | Web UI and parse API    |
| 10500 | Wyoming intent          |

UI: <http://127.0.0.1:10520>

## Home Assistant

HACS installs the conversation integration. The Klar engine (binary, Docker, or add-on) still has to run separately.

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. Click the badge, or in HACS → Integrations → ⋮ → Custom repositories add `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` as **Integration**
2. Download **Klar NLU** and restart Home Assistant
3. Start the Klar engine
4. [![Open your Home Assistant instance and start setting up a new integration.](https://my.home-assistant.io/badges/config_flow_start.svg)](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu) (URL default `http://127.0.0.1:10520`)
5. Assist pipeline: conversation engine = **Klar NLU**
6. Optionally pick a conversation agent for chit-chat in the options

Manual install: copy `custom_components/klar_nlu` to `config/custom_components/klar_nlu`, then restart.

Details: [docs/en/home-assistant.md](docs/en/home-assistant.md)

## Documentation

| Topic | English | Deutsch |
|-------|---------|---------|
| Architecture | [docs/en/architecture.md](docs/en/architecture.md) | [docs/architecture.md](docs/architecture.md) |
| HTTP and Wyoming API | [docs/en/api.md](docs/en/api.md) | [docs/api.md](docs/api.md) |
| Home Assistant | [docs/en/home-assistant.md](docs/en/home-assistant.md) | [docs/home-assistant.md](docs/home-assistant.md) |
| Adding languages | [docs/en/languages.md](docs/en/languages.md) | [docs/languages.md](docs/languages.md) |
| Tests | [docs/en/testing.md](docs/en/testing.md) | [docs/testing.md](docs/testing.md) |
| Releases | [docs/en/releases.md](docs/en/releases.md) | [docs/releases.md](docs/releases.md) |

## Releases

Changelogs come from [git-cliff](https://git-cliff.org/) and [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, …). See [CHANGELOG.md](CHANGELOG.md).

**Actions → Release → Run workflow** bumps the version, writes the changelog, and tags `vX.Y.Z`. The Build workflow then attaches linux-x86_64, linux-aarch64, and linux-armv7 tarballs to the GitHub Release. A manual `git tag v0.1.0 && git push origin v0.1.0` still works.

Dependabot opens weekly PRs for crates and Actions; `cargo-audit` and `cargo-deny` run on every change.

## Tests

```bash
cargo test -- --test-threads=1
```

The German/English apartment suites and both family-home suites must stay at 100%. See [docs/en/testing.md](docs/en/testing.md).

## License

[MIT](LICENSE) — Copyright 2026 FABBricate IT Solutions
