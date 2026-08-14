# Home Assistant

[Deutsch](../home-assistant.md) · [English](home-assistant.md)

Klar attaches to Assist as a conversation entity. The engine runs separately (binary, Docker, or add-on); the integration talks to it over HTTP.

## Integration

HACS installs only the conversation integration. The engine (binary, Docker, or add-on) runs separately.

[![Open your Home Assistant instance and open a repository inside the Home Assistant Community Store.](https://my.home-assistant.io/badges/hacs_repository.svg)](https://my.home-assistant.io/redirect/hacs_repository/?owner=FABBricate-IT-Solutions&repository=klar-ha-nlu&category=integration)

1. Click the badge, or HACS → Integrations → ⋮ → Custom repositories → `https://github.com/FABBricate-IT-Solutions/klar-ha-nlu` → category **Integration**.
2. Download **Klar NLU** and restart Home Assistant.
3. Start the Klar engine.
4. [![Open your Home Assistant instance and start setting up a new integration.](https://my.home-assistant.io/badges/config_flow_start.svg)](https://my.home-assistant.io/redirect/config_flow_start/?domain=klar_nlu)  
   or Settings → Devices & services → Add integration → **Klar NLU**.
5. Engine URL, default `http://127.0.0.1:10520`.

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

## Starting the engine

Locally, HA config read-only:

```bash
cargo run --release -- --config-dir /config
```

Or Docker (root Dockerfile):

```bash
docker build -t klar-nlu .
docker run --network host -v /config:/config:ro klar-nlu
```

Add-on metadata is under `addon/` (`host_network`, ports 10520/10500, config read-only).

The engine reads `.storage/core.entity_registry` and `core.area_registry`. Keep aliases and areas in HA — Klar has no second device database.

## Intents

The integration executes the intent names Klar returns. Built-in Assist intents must be available in HA (they are). Custom sentences in Klar (`/api/custom`) can point at the same names or your own; custom names need a handler in HA.
