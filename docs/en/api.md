# API

[Deutsch](../api.md) · [English](api.md)

Two interfaces: HTTP on port **10520**, Wyoming on **10500**.

## HTTP

### `POST /api/parse`

```json
{ "text": "Licht im Wohnzimmer an", "conversation_id": "optional-id" }
```

Response:

```json
{
  "text": "Licht im Wohnzimmer an",
  "intents": [
    {
      "name": "HassTurnOn",
      "slots": [
        { "name": "area", "value": "wohnzimmer" },
        { "name": "domain", "value": "light" }
      ]
    }
  ],
  "speech": "Schalte wohnzimmer ein.",
  "clarify": false,
  "conversation_id": "…"
}
```

`clarify: true` means: do not run intents, speak the question in `speech`, keep the same `conversation_id` for the answer.

Empty `intents` = no home command. The HA integration forwards that to the fallback agent if one is set.

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

### `GET /api/entities`

Loaded home graph (name, area, aliases, tags).

### `POST /api/entities`

```json
{ "entity_id": "light.wohn_decke", "tags": ["decke"] }
```

Tags help resolution when the display name is too generic.

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

The HA custom component uses HTTP, not Wyoming. Wyoming is for add-ons and satellites that expect an intent server.
