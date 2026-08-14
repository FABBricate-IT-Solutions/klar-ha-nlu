# Home Assistant

[Deutsch](../home-assistant.md) · [English](home-assistant.md)

Klar attaches to Assist as a conversation entity. The engine runs separately (binary, Docker, or add-on); the integration talks to it over HTTP.

## Integration

1. Copy `custom_components/klar_nlu` to `<config>/custom_components/klar_nlu`.
2. Restart Home Assistant.
3. Settings → Devices & services → Add integration → **Klar NLU**.
4. Engine URL, default `http://127.0.0.1:10520`.

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
