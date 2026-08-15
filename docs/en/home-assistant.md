# Home Assistant

[Deutsch](../home-assistant.md) · [English](home-assistant.md)

Klar attaches to Assist as a conversation entity. HACS cannot start the Rust engine. The integration can: it downloads the matching GitHub Release into `.storage/klar_nlu/` and runs it on `127.0.0.1:10520`.

## Integration

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. Click the badge, or HACS → Integrations → ⋮ → Custom repositories → `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` → category **Integration**.
2. Download **Klar NLU** and restart Home Assistant.
3. [![Open your Home Assistant instance and start setting up a new integration.](https://my.home-assistant.io/badges/config_flow_start.svg)](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu)  
   or Settings → Devices & services → Add integration → **Klar NLU**.
4. Keep **Start the bundled engine** (needs a GitHub Release with linux tarballs). Or pick **Use an engine that is already running** and set the URL.

Without HACS, copy `custom_components/klar_nlu` to `<config>/custom_components/klar_nlu` and restart.

One instance only. The URL stays in the first step; the chit-chat agent lives in the options.

## Assist pipeline

Settings → Voice assistants → edit the pipeline:

- **Conversation engine:** Klar NLU
- STT/TTS as you like (local or cloud)

Do not set the LLM agent as the engine. Assist would skip Klar and the LLM could control devices.

## LLM fallback

Settings → Devices & services → Klar NLU → Configure → **Conversation agent for chit-chat**.

Flow:

1. Klar parses.
2. Home intents run through `intent.async_handle`.
3. Clarifications (`clarify`) stay with Klar.
4. No intents → forward to the chosen agent.
5. Klar itself and an unreachable engine do not trigger fallback.

The agent is told not to control devices. If that agent still has HA tools in its own integration, they can still fire — turn tools off there if chit-chat should only talk.

## Personality

Settings → Devices & services → Klar NLU → Configure → **Personality**, or the **Personality** select entity on the Klar device.

The choice lives in this integration, not in the Klar app, so it survives reinstalling the engine. Assist, the spoken cue, and the LLM refine prompt all switch with it. Changing only the personality does not restart the engine.

| Id | Cue (DE) | Spoken tag |
|----|----------|------------|
| `default` | — | plain confirmation |
| `butler` | Sehr wohl. | …, wie gewünscht. |
| `locker` | Geht klar. | …, passt. |
| `fuersorglich` | Mache ich sofort. | …, alles gut. |
| `party` | Läuft! | …, super! |
| `grantig` | Schon gut. | …, na gut. |
| `sarkastisch` | Wie überraschend, wieder ein Befehl. | …, natürlich. |
| `pirat` | Aye. | …, Käpt'n. |
| `hippie` | Alles easy. | …, ganz ruhig. |
| `gollum` | Ja, mein Schatz. | …, ja. |

English uses the matching cues (`Very well.`, `Got it.`, `Aye.`, `Yes, my precious.`, …).

## LLM refine

Off by default. Settings → Devices & services → Klar NLU → Configure:

1. Set **Conversation agent for chit-chat** (OpenAI-compatible, local Gemma is fine).
2. Turn on **Let the LLM refine NLU replies**.
3. Keep Assist’s conversation engine = **Klar NLU**.
4. Turn Assist tools **off** on that LLM agent. If the agent can control the home, refine is skipped.

Flow after a home command:

1. Klar parses and HA runs the intents.
2. Klar prefixes the selected personality cue.
3. The fallback LLM rewrites that finished sentence after control and after a status query (not chit-chat, not news, not a clarify question).
4. If the model drops the cue, Klar puts it back.

The rewrite prompt is per personality (few-shots plus a fixed style tag). The **Refinement prompt** field is an optional extra line on top — it does not replace the voice.

Safety stays with Klar, not the model:

- no device control, no Home Assistant tools
- rooms, names, on/off/open/closed stay
- digits stay digits (`21` stays `21`, not twenty-one)
- no invented numbers (a temperature fragment without a value stays without a value)
- intent names such as `HassSetPosition` are rejected

On OpenAI-compatible agents Klar sends `chat_template_kwargs.enable_thinking=false` so Gemma 4 does not spend the turn in a thought channel. Prompt text cannot turn thinking off. If the agent has no direct chat client, Klar falls back to `conversation.async_converse` and keeps the NLU sentence only if the rewrite fails.

## Starting the engine

**Bundled (simplest):** HACS integration → **Start the bundled engine**. Downloads the GitHub Release into `.storage/klar_nlu/`.

**Add-on (HAOS):**

[![Open your Home Assistant instance and show the add add-on repository dialog with a specific repository URL pre-filled.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2FFABBricate-IT-Solutions%2Fklar-ha-nlu)

Add `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` as an add-on repository, install **Klar NLU**, then point the integration at `http://klar-nlu:10520`.

**Docker:**

```bash
docker run --rm --network host \
  -v /path/to/homeassistant/config:/config:ro \
  ghcr.io/fabbricate-it-solutions/klar-nlu:0.1.0
```

Integration URL: `http://127.0.0.1:10520`. From source: `docker build -t klar-nlu .` (root Dockerfile).

**Cargo:**

```bash
cargo run --release -- --config-dir /config
```

The engine reads `.storage/core.entity_registry` and `core.area_registry`. Keep aliases and areas in HA — Klar has no second device database.

## Intents

The integration executes the intent names Klar returns. Built-in Assist intents must be available in HA (they are). Custom sentences in Klar (`/api/custom`) can point at the same names or your own; custom names need a handler in HA.
