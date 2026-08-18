# Klar NLU

<div align="center">

![Klar](https://raw.githubusercontent.com/FABBricate-IT-Solutions/klar-ha-nlu/main/docs/klar-logo-sm.png)

<img src="docs/social-preview.png" alt="Klar NLU" width="640">

</div>

[English](README.md) · [Deutsch](README.de.md)

[![CI](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/ci.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/ci.yml)
[![Security](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/security.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/security.yml)
[![Build](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/build.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/build.yml)
[![Validate](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/validate.yml/badge.svg)](https://github.com/FABBricate-IT-Solutions/klar-ha-nlu/actions/workflows/validate.yml)
[![HACS Custom](https://img.shields.io/badge/HACS-Custom-orange.svg)](https://github.com/hacs/integration)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Deterministic voice control for Home Assistant. Klar NLU turns spoken sentences into HA intents — no cloud, no model weights.

**Household path:** [Getting started](docs/en/getting-started.md) — HACS → expose entities → Assist pipeline → five phrases → Mapping. If something misses: [troubleshooting](docs/en/troubleshooting.md).

**Breaking V2:** HTTP parse is only `POST /api/v2/parse` (`schema_version: "2.0"`). Upgrade the Rust engine and the Home Assistant integration together. `POST /api/parse` is gone. Existing `klar_nlu.json` overlays still load; `klar migrate import --from` dry-runs conflicts, orphans, and unsafe settings before a V2 save.

Every compiled Assist locale is first-class. German and English are hand-written reference packs and useful examples; generated packs use the same path. See [languages](docs/en/languages.md).

```
“Licht im Wohnzimmer an”     →  HassTurnOn   area=wohnzimmer  domain=light
“Set the garage door to 40%” →  HassSetPosition  cover.garage_door
“Spiel Queen” / “Play Queen” →  Music Assistant search-and-play
```

## What Klar NLU does

- Home commands: lights, climate, covers, locks, fans, media (including Music Assistant), timers, lists, scenes
- Multiple clauses (`living room and kitchen`, `turn the lights off and set heat to 21`)
- Clarification when a device is ambiguous
- Session: “turn it off” refers to the last target
- Personalities in Home Assistant (butler, grumpy, pirate, …) — Assist, the spoken cue, and the optional LLM rewrite all follow the same choice
- Optional LLM fallback in Home Assistant for chit-chat once Klar sees no home command
- Optional LLM refine of finished NLU replies (off by default; device control stays with Klar)

Klar NLU drives devices itself. An LLM only talks or rewrites speech — it does not run home intents.

## Home Assistant

HACS installs the conversation integration. On setup, pick **Start the bundled engine** — the integration downloads the matching GitHub Release and starts Klar NLU next to Home Assistant. HACS cannot start the Rust process itself.

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. Click the badge, or in HACS → Integrations → ⋮ → Custom repositories add `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` as **Integration**
2. Download **Klar NLU** and restart Home Assistant
3. [![Open your Home Assistant instance and start setting up a new integration.](https://my.home-assistant.io/badges/config_flow_start.svg)](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu) and keep **Start the bundled engine**
4. Expose the devices you want to talk to (Settings → Voice assistants → Expose)
5. Assist pipeline: conversation engine = **Klar NLU**
6. Optionally pick a conversation agent for chit-chat in the options. Same place: personality, and **Let the LLM refine NLU replies** if that agent should rewrite confirmations

If Klar NLU already runs, choose **Use an engine that is already running** and set the URL.

Step-by-step with example phrases: [docs/en/getting-started.md](docs/en/getting-started.md).

### Add-on (Home Assistant OS)

[![Open your Home Assistant instance and show the add add-on repository dialog with a specific repository URL pre-filled.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2FFABBricate-IT-Solutions%2Fklar-ha-nlu)

Settings → Add-ons → ⋮ → Repositories → `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` → install **Klar NLU**. Then add the integration with URL `http://klar-nlu:10520`.

Release candidates: install **Klar NLU (Staging)** from the same repository and use `http://klar-nlu-staging:10520`. Bundled engine: Configure → **Release channel** → Staging. See [releases](docs/en/releases.md).

### Docker

Pin the image tag to the engine CalVer (same as `Cargo.toml` / the GitHub Release). Current tree: **2026.8.30**.

```bash
docker run --rm --network host \
  -v /path/to/homeassistant/config:/config:ro \
  ghcr.io/fabbricate-it-solutions/klar-nlu:2026.8.30
```

Integration URL: `http://127.0.0.1:10520`. Images also exist per arch (`klar-nlu-amd64`, `klar-nlu-aarch64`, `klar-nlu-armv7`). RC images: tag `staging`.

Manual install: copy `custom_components/klar_nlu` to `config/custom_components/klar_nlu`, then restart.

Details: [docs/en/home-assistant.md](docs/en/home-assistant.md)

## Documentation

| Topic | English | Deutsch |
|-------|---------|---------|
| Getting started | [docs/en/getting-started.md](docs/en/getting-started.md) | [docs/getting-started.md](docs/getting-started.md) |
| Troubleshooting and privacy | [docs/en/troubleshooting.md](docs/en/troubleshooting.md) | [docs/troubleshooting.md](docs/troubleshooting.md) |
| Home Assistant | [docs/en/home-assistant.md](docs/en/home-assistant.md) | [docs/home-assistant.md](docs/home-assistant.md) |
| HTTP and Wyoming API | [docs/en/api.md](docs/en/api.md) | [docs/api.md](docs/api.md) |
| Languages | [docs/en/languages.md](docs/en/languages.md) | [docs/languages.md](docs/languages.md) |
| Architecture | [docs/en/architecture.md](docs/en/architecture.md) | [docs/architecture.md](docs/architecture.md) |
| Tests | [docs/en/testing.md](docs/en/testing.md) | [docs/testing.md](docs/testing.md) |
| Releases | [docs/en/releases.md](docs/en/releases.md) | [docs/releases.md](docs/releases.md) |
| Contributing | [CONTRIBUTING.md](CONTRIBUTING.md) | |

## How it is organized

The Rust engine is split into clear layers:

- `src/types/` defines intents, `ParseOutcome`, settings, and the home graph.
- `src/nlu/` ranks candidates and applies confidence, OOD, confirm, and multi-intent policy.
- `src/parse/` tokenizes, detects actions, resolves targets, and fills slots.
- `src/home/` loads Home Assistant registries, overlays, expose data, roles, and the built-in sample home.
- `src/eval/` and `src/migrate.rs` own held-out scorecards and the one-shot V1 overlay import.
- `src/io/` owns HTTP, Wyoming, shared runtime state, token handling, and reloads.

At runtime Klar NLU builds one effective home graph from the HA config and the writable data directory. The parse API and Wyoming server share that graph and the same session store, so follow-ups behave the same on both interfaces.

## Quick start (from source)

Rust 1.85+, then:

```bash
cargo run --release -- --config-dir /path/to/ha-config --http 0.0.0.0:10520 --wyoming 0.0.0.0:10500
```

Without an HA config Klar NLU uses a built-in sample home.

| Port  | Service                 |
|-------|-------------------------|
| 10520 | Web UI and parse API    |
| 10500 | Wyoming intent          |

UI: <http://127.0.0.1:10520> — React operator UI (Home, Conversations, Rules, House / Mapping, Lab, Settings).

Useful checks while developing:

```bash
cargo fmt --check
cargo check
cargo nextest run
cargo run --quiet -- lang validate packs
cargo run --quiet -- eval bench --repeat 8
cargo build --release
```

Do not run `scripts/lang_packs/generate.py` in pre-commit. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Releases

Changelogs come from [git-cliff](https://git-cliff.org/) and [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, …). See [CHANGELOG.md](CHANGELOG.md).

Every merge to `main` bumps the [Home Assistant CalVer](https://developers.home-assistant.io/docs/versioning/) (`YYYY.M.PATCH`), writes the changelog, and tags. The Build workflow then attaches linux-x86_64, linux-aarch64, and linux-armv7 tarballs to the GitHub Release. **Actions → Release → Run workflow** remains for a manual override. A manual `git tag 2026.8.0 && git push origin 2026.8.0` still works.

Dependabot opens weekly PRs for crates and Actions; `cargo-audit` and `cargo-deny` run on every change.

## Tests

```bash
cargo nextest run
```

The enforced voice-suite gates are documented in
[docs/en/testing.md](docs/en/testing.md); 100% remains the target, but the
current blocking thresholds are lower for the generated comparison suites.

## License

[MIT](LICENSE) — Copyright 2026 FABBricate IT Solutions
