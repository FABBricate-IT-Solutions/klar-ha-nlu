# API

[Deutsch](../api.md) · [English](api.md)

Two interfaces: HTTP on port **10520**, Wyoming on **10500**.

## HTTP

### `POST /api/parse`

```json
{ "text": "Turn on the living room light", "conversation_id": "optional-id", "language": "en", "personality": "butler" }
```

Response:

```json
{
  "text": "Turn on the living room light",
  "intents": [
    {
      "name": "HassTurnOn",
      "slots": [
        { "name": "area", "value": "wohnzimmer" },
        { "name": "domain", "value": "light" }
      ]
    }
  ],
  "speech": "Very well. Living room light is on.",
  "clarify": false,
  "conversation_id": "optional-id",
  "chat": false,
  "briefing": false,
  "personality": "butler"
}
```

`text` is limited to 4096 characters. The HTTP body is limited to 16 KiB.

`language` is optional (`de`, `en`, or a BCP-47 tag such as `en-US`). When set, Klar binds only that pack for the request so Assist can switch between German and English. `speech` follows the pinned pack.

`personality` is optional and prefixes `speech` on this endpoint (`Sehr wohl.`, `Aye.`, …). Home Assistant stores the choice in the integration and sends it on every parse; the engine settings copy is only for the Klar UI. LLM refine in the HA integration then rewrites that sentence in the selected voice and does not stamp the cue back on.

`clarify: true` means: do not run intents, speak the question in `speech`, keep the same `conversation_id` for the answer.

Empty `intents` = no home command. The HA integration forwards that to the fallback agent if one is set.

`chat: true` means Klar intentionally emitted no home intent and the sentence may go to the fallback agent. `briefing: true` marks news/briefing dialog so follow-ups do not accidentally control devices.

### Auth and Errors

Read-only HTTP endpoints are allowed from loopback and the Home Assistant Supervisor network. Write endpoints are allowed without a token only from loopback. Other hosts must send a token:

```http
x-klar-token: secret
Authorization: Bearer secret
```

The token comes from `--token`, `KLAR_TOKEN`, or `--token-file`.

| Status | Meaning |
|--------|---------|
| `400` | invalid custom sentence or entity ID |
| `401` | token is missing or does not match |
| `404` | entity not found |
| `413` | parse text too long |

### `GET` / `POST /api/settings`

```json
{
  "personality": "default",
  "mode": "full",
  "languages": ["de", "en"]
}
```

| Field | Values |
|-------|--------|
| `personality` | `default`, `butler`, `locker`, `fuersorglich`, `party`, `grantig`, `sarkastisch`, `pirat`, `hippie`, `gollum` |
| `mode` | `full` (resolve devices) or `context_only` (areas only) |
| `languages` | Pack codes. Unknown codes are ignored. Empty falls back to `de`+`en`. |

### `GET` / `POST /api/custom`

Custom phrases, exact or fuzzy match:

```json
[{ "phrase": "filmabend", "intent": "HassTurnOn", "slots": { "entity_id": "scene.film" } }]
```

At most 64 entries. Each phrase must have at least four characters and at most 200 bytes; `intent` must be a known intent name.

### `GET /api/entities`

Loaded home graph (name, area, aliases, tags).

### `POST /api/entities`

```json
{ "entity_id": "light.wohn_decke", "tags": ["decke"] }
```

Tags help resolution when the display name is too generic.

`aliases`, `preferred`, and optional `area` are persisted in the overlay and applied immediately to the running `HomeStore`.

### `GET /api/gaps`

Calibration view for entities with weak names, missing areas, or no helpful aliases/tags. The response contains `leftover`, `rooms`, and the current overlay.

### `GET /`

Local test UI (`web/index.html`).

## Wyoming

Klar speaks the Wyoming intent protocol (one JSON line per event).

`describe` → `info` with name, version, and `languages` from settings.

`recognize` with `data.text`:

| Result | Event |
|--------|--------|
| nothing recognized | `not-recognized` |
| one intent | `intent` |
| several | `intents-start` … `intent` … `intents-stop` |

Each intent carries `name`, `entities` (`name`/`value`), and `text` (confirmation).

Wyoming accepts only loopback and the Supervisor network. It uses the same `HomeStore`, sessions, and settings as HTTP. The HA custom component uses HTTP, not Wyoming. Wyoming is for add-ons and satellites that expect an intent server.
