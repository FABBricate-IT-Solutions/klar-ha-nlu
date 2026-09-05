# Home Assistant

[Deutsch](../home-assistant.md) · [English](home-assistant.md)

Household zero-to-Assist: [getting started](getting-started.md). Misses, token, bundle: [troubleshooting](troubleshooting.md).

Klar NLU attaches to Assist as a conversation entity. That is the **HACS integration**. The **engine** that parses speech is separate: the Klar NLU App (add-on), the bundled child the integration starts, or Docker. Same parser either way — the App does not make Assist smarter.

Install both on Home Assistant OS if you want Assist **and** Mapping/Lab. HACS alone is enough for Assist without the App UI. The App alone does not hook Assist.

V2 talks `POST /api/v2/parse` only. Update the integration and the engine in the same release; a mixed pair will fail. Every compiled Assist locale is first-class; Assist pins one pack per request.

## Integration

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. Click the badge, or HACS → Integrations → ⋮ → Custom repositories → `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` → category **Integration**.
2. Download **Klar NLU** and restart Home Assistant.
3. [![Open your Home Assistant instance and start setting up a new integration.](https://my.home-assistant.io/badges/config_flow_start.svg)](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu)  
   or Settings → Devices & services → Add integration → **Klar NLU**.
4. With the App running, pick **Use the Klar NLU App or Docker**. Without it, pick **Start the bundled engine (HACS only)** (needs a GitHub Release with linux tarballs).
5. Bundled engine: **Release channel** = Stable (CalVer) or Staging (latest GitHub prerelease). You can change this later under Configure.

Without HACS, copy `custom_components/klar_nlu` to `<config>/custom_components/klar_nlu` and restart.

One instance only. The URL stays in the first step; the chit-chat agent lives in the options.

## Assist pipeline

Settings → Voice assistants → edit the pipeline:

- **Conversation engine:** Klar NLU
- STT/TTS as you like (local or cloud)

Do not set the LLM agent as the engine. Assist would skip Klar and the LLM could control devices.

## LLM fallback

Settings → Devices & services → Klar NLU → Configure → **Conversation agent for chit-chat**.

There is no user phrase such as “Ask the LLM”. If a fallback agent is set, unhandled speech (reject or empty intents) goes to that agent in the Assist language pack. Refine is only tone on an already-spoken NLU line.

Flow:

1. Klar parses.
2. Home intents run through `intent.async_handle`.
3. Clarifications (`clarify`) stay with Klar.
4. No intents, including reject → forward to the chosen agent. NLU-RAG is not required.
5. Klar itself and an unreachable engine do not trigger fallback.

The agent is told not to control devices and to answer in the user’s Assist language. If that agent still has HA tools in its own integration, Klar skips it unless you turn on **Allow Assist tools on the chit-chat agent**. That option is off by default. When it is on, the model may turn lights and run scripts.

## Personality

Klar operator UI → Settings → **Voice**, or the **Personality** select entity on the Klar device (it patches the engine).

The choice lives in engine settings (`GET`/`POST /api/settings`), not in the Home Assistant options form. Assist reads the engine cache each turn and falls back to leftover integration options only if that cache is empty. Changing personality does not restart the engine.

| Id | Voice |
|----|-------|
| `default` | plain, friendly |
| `butler` | polite butler, formal and discreet |
| `locker` | casual, buddy-like |
| `fuersorglich` | warm, reassuring |
| `party` | hyped, celebratory |
| `grantig` | grumpy, reluctant |
| `sarkastisch` | dryly sarcastic |
| `pirat` | pirate-like, still clear |
| `hippie` | chill, soft |
| `gollum` | hissy, still clear |

With LLM refine the voice lives in the sentence — not in a stamp such as “Very well.” Without refine, a short spoken variant rotates in (sometimes none). The fallback is not a fixed stamp. Refine follows the language of the input line; it does not silently wrap other locales in German.

## LLM refine

Off by default. Klar operator UI → Settings:

1. Set the LLM endpoint (OpenAI-compatible; local Gemma is fine).
2. Turn on **LLM refine**.
3. Keep Assist’s conversation engine = **Klar NLU**.
4. Keep Assist tools **off** on chat unless you also turn on **Allow Assist tools on chat**. If the model can control the home and that option is off, Klar skips that agent.

Flow after a home command:

1. Klar parses and HA runs the intents.
2. The fallback LLM rewrites the finished NLU reply in the selected personality — one or two spoken sentences, after control and after a status query (not chit-chat, not news, not a clarify question).
3. That rewrite stands. Klar does not stamp a cue back on.
4. If refine fails, the short fallback cue remains.

The rewrite prompt is per personality (voice plus few-shots). **Extra prompt** on Settings is an optional extra line on top — it does not replace the voice.

Safety stays with Klar, not the model:

- no device control, no Home Assistant tools
- rooms, names, on/off/open/closed stay
- digits stay digits (`21` stays `21`, not twenty-one)
- no invented numbers (a temperature fragment without a value stays without a value)
- intent names such as `HassSetPosition` are rejected

On OpenAI-compatible agents Klar sends `chat_template_kwargs.enable_thinking=false` so Gemma 4 does not spend the turn in a thought channel. Prompt text cannot turn thinking off. If the agent has no direct chat client, Klar falls back to `conversation.async_converse` and keeps the NLU sentence only if the rewrite fails.

## Starting the engine

Pick **one** host. Reloading the integration onto the App URL stops the bundled child.

**Home Assistant OS (recommended):** HACS integration **plus** the App. The App is the engine host and the Mapping/Lab UI (sidebar **Klar NLU**).

[![Open your Home Assistant instance and show the add add-on repository dialog with a specific repository URL pre-filled.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2FFABBricate-IT-Solutions%2Fklar-ha-nlu)

Add `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` as an App repository, install **Klar NLU** or **Klar NLU (Staging)**, start it, then in the integration choose **Use the Klar NLU App or Docker** (`http://klar-nlu:10520`).

**Bundled (no App):** HACS integration → **Start the bundled engine (HACS only)**. Downloads the GitHub Release into `/config/klar_nlu/`. Assist works. Mapping/Lab bind `127.0.0.1` inside Core, so a phone cannot open them. Lovelace **Klar** is only the last Assist turn.

**Stable vs staging:** one switch. Settings → Devices & services → Klar NLU → Configure → **Release channel**. Stable points at the Klar NLU app (`http://klar-nlu:10520`) or the CalVer GitHub release. Staging points at Klar NLU (Staging) (`http://klar-nlu-staging:10520`) or the latest prerelease. A custom URL is left alone. The integration reloads after the switch. Do not edit `.storage`.

The Supervisor reads the add-on repo from `main`. **Klar NLU (Staging)** shows in the store only after `addon-staging/` is on that branch. After a merge to `staging`, rebuild the staging app (version stays `staging`).

**Docker:**

```bash
docker run --rm --network host \
  -v /path/to/homeassistant/config:/config:ro \
  ghcr.io/fabbricate-it-solutions/klar-nlu:2026.8.30
```

Use the CalVer that matches the engine (`Cargo.toml` / GitHub Release), not an old `0.1.x` tag. For a release candidate: `ghcr.io/fabbricate-it-solutions/klar-nlu:staging`.

Integration URL: `http://127.0.0.1:10520`. From source: `docker build -t klar-nlu .` (root Dockerfile).

**Cargo:**

```bash
cargo run --release -- --config-dir /config
```

The engine can read `.storage/core.entity_registry`, `core.device_registry`, `core.area_registry`, `core.floor_registry`, `core.label_registry`, and the Assist expose list. That path is the fallback for Wyoming-only or when the integration does not push. Keep aliases, areas, floors, and Assist exposure in HA — Klar has no second device database.

## Registry sync (HA is the source of truth)

The integration is the official sync path. After setup and on registry/expose updates (`entity`, `device`, `area`, `floor`, `label`, `exposed_entities`) it posts a versioned snapshot to `POST /api/v2/home`.

```json
{
  "schema_version": "1",
  "entities": [
    {
      "entity_id": "light.living",
      "name": "Living ceiling",
      "original_name": "Ceiling",
      "has_entity_name": true,
      "area_id": "living",
      "device_id": "dev1",
      "platform": "hue",
      "aliases": ["ceiling"],
      "labels": ["Light"],
      "disabled": false
    }
  ],
  "devices": [{"id": "dev1", "name": "Hue", "name_by_user": null, "area_id": "living"}],
  "areas": [{"id": "living", "name": "Living room", "aliases": ["living"], "floor_id": "upper"}],
  "floors": [{"floor_id": "upper", "name": "Upper floor", "aliases": ["upstairs"], "level": 1}],
  "labels": [{"label_id": "lbl_1", "name": "Light"}],
  "assist": ["light.living"]
}
```

`schema_version` must be `"1"`. The engine validates the snapshot at the API boundary: unknown fields and invalid JSON are rejected with `422`, empty IDs, control characters, and schema errors with `400`, and oversized bodies or collections with `413`. None of these crash the process. Caps: 4096 entities, 2048 devices, 256 areas, 64 floors, 256 labels, 4096 Assist IDs, 32 aliases per item. `assist: null` means no expose filter; an array limits visible IDs.

After a valid push, HA is the live source. The `.storage` file watcher no longer overwrites that graph. Overlay calibration (aliases, manual areas, preferred, infra) is still applied on top.

## Registry, Overlay, and Reload

Klar builds one effective home graph:

1. Use the live snapshot from the integration (`POST /api/v2/home`) when present.
2. Otherwise read HA registries from `--config-dir`: entities, devices, areas, floors, labels, and the expose list.
3. If the registries are missing, use the built-in sample home.
4. Apply the overlay from `--config-dir`.
5. If `--data-dir` exists and differs from `--config-dir`, apply the overlay from `--data-dir` on top.

The overlay contains Klar UI calibration: aliases, manual areas, preferred devices, infrastructure filters, timer hints, settings, and custom sentences. In the add-on, `/config` is meant to be read-only and `/data` writable; with Cargo/Docker both can point at the same directory.

Without live sync, Klar watches the HA registry files and reloads the home graph when they change. Settings and custom sentences are re-read from the overlays during that reload. Changes from the Klar UI are saved immediately and applied to the running `HomeStore`.

## Access and Token

Loopback may read and write. The Supervisor network may read; writes from there or from the LAN require a token. Set it with `--token`, `KLAR_TOKEN`, or `--token-file`.

```bash
cargo run --release -- --config-dir /config --data-dir /data --token-file /data/klar.token
```

HTTP accepts the token as `x-klar-token` or `Authorization: Bearer ...`. Wyoming is limited to loopback and the Supervisor network.

## Intents

The integration executes only `decision.type == execute`. Confirm, clarify, and reject never call services. An execute plan runs in plan order through `intent.async_handle`. Each step reports success or error; partial failure is a first-class outcome (speech plus structured errors), not silent total success. Direct service calls are used only where HA has no native intent (Music Assistant, relative volume, mute). Custom sentences in Klar NLU (`/api/custom`) can point at the same names or your own; custom names need a handler in HA.

## Floor and room status

`What's the status of the apartment` stays **one** `HassGetState` with `floor` — Klar does not expand that into eight room intents (that would trip confirm). The integration collects the areas on the floor and speaks a fixed order: room, then each light and outlet by name, presence, temperature, lux, then the other devices by name. Switches without `device_class: outlet` and climate entities are named devices, not an anonymous socket or a temperature number alone. The same clauses are used for a single-room status. Every Assist speech pack uses that order. An empty floor or room says a localized “No devices”, not Home Assistant’s mixed short report. Areas without a floor stay out.

## Media and Music Assistant

Pause, next, previous, mute, and volume use the named `media_player` or the one in that room. The player must be Assist-exposed and either a Music Assistant player or tagged for music. Without that player the miss is setup, not a missing English lexicon.

| Phrase | Notes |
|--------|--------|
| Pause the living room TV | Native media pause |
| Play Queen | Search-and-play on a Music Assistant player |
| Play the playlist Chill in the living room | Keeps playlist class and the area |
| Spiel Queen / Musik an | Same path in German; `Musik an` resumes the MA player |

Klar NLU does not invent a library. Unavailable players are skipped. See [getting started](getting-started.md) and [troubleshooting](troubleshooting.md).
