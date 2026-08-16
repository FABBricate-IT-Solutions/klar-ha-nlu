# API

[Deutsch](../api.md) · [English](api.md)

Two interfaces: HTTP on port **10520**, Wyoming on **10500**.

**Breaking:** `POST /api/parse` is gone. Clients and the Home Assistant integration must use `POST /api/v2/parse` and `schema_version: "2.0"`. Upgrade the engine and the integration together.

## HTTP

### `POST /api/v2/parse`

```json
{ "text": "Turn on the living room light", "conversation_id": "optional-id", "language": "en", "personality": "butler" }
```

Response:

```json
{
  "schema_version": "2.0",
  "text": "Turn on the living room light",
  "conversation_id": "optional-id",
  "decision": { "type": "execute" },
  "speech": "Very well. Living room light is on.",
  "confidence": 0.96,
  "margin": 0.18,
  "selected_candidate_id": "selected-000",
  "plan": {
    "confidence": 0.96,
    "margin": 0.18,
    "evidence": [],
    "steps": [{
      "index": 0,
      "intent": {
        "name": "HassTurnOn",
        "slots": [
          { "name": "area", "value": "wohnzimmer" },
          { "name": "domain", "value": "light" }
        ]
      },
      "confidence": 0.96,
      "evidence": []
    }]
  },
  "candidates": [],
  "evidence": [],
  "trace": { "stages": [], "discarded": [] },
  "briefing": false
}
```

`text` is limited to 4096 characters. The HTTP body is limited to 16 KiB.

`language` is optional (`de`, `en`, or a BCP-47 tag such as `en-US`). When set, Klar binds only that pack for the request so Assist can switch between German and English. `speech` follows the pinned pack.

`personality` is optional and prefixes `speech` on this endpoint (`Sehr wohl.`, `Aye.`, …). Home Assistant stores the choice in the integration and sends it on every parse; the engine settings copy is only for the Klar UI. LLM refine in the HA integration then rewrites that sentence in the selected voice and does not stamp the cue back on.

`decision.type` is one of `execute`, `clarify`, `confirm`, `reject`, `chat`, or `error`. Only `execute` contains `plan`, complete `candidates`, and `selected_candidate_id`; clients must execute intents only for that decision. For every other decision, `candidates` is empty and no intent or slot data is serialized anywhere. `confirm` contains only a prompt and an opaque candidate ID. The proposal remains exclusively in the session until the same `conversation_id` answers affirmatively.

Confidence is evidence-backed and comparable across intents. The engine applies fixed bands: execute at confidence ≥ 0.80 and margin ≥ 0.05 when no competing complete plan is closer than 0.05; confirm for risky lock/unlock/cover-close or large plans when confidence ≥ 0.62; clarify when competing complete plans are closer than 0.05 or confidence is between 0.70 and the execute band; reject below 0.70 (below 0.62 when risky), for out-of-domain talk (weather, trivia, empty fillers), or when no grounded step remains. Fuzzy, session, and inferred evidence cannot outscore exact lexical+resolver evidence, and inferred actions never claim 1.0. Multi-clause plans keep only independently valid steps. After `nein` a pending confirm is dropped; after `ja` the stored plan is re-validated against the current home graph. `chat` stays limited to news, briefing follow-ups, and explicit LLM opt-in.

`candidates`, `evidence`, and `trace` explain ranking and discarded alternatives. All arrays and detail strings are server-capped.

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
  "languages": ["de", "en"],
  "support_bundle": false,
  "support_bundle_raw_text": false,
  "confirm_risky_actions": true,
  "semantic_adapters": false
}
```

| Field | Values |
|-------|--------|
| `personality` | `default`, `butler`, `locker`, `fuersorglich`, `party`, `grantig`, `sarkastisch`, `pirat`, `hippie`, `gollum` |
| `mode` | `full` (resolve devices) or `context_only` (areas only) |
| `languages` | Pack codes. Unknown codes are ignored. Empty falls back to `de`+`en`. |
| `support_bundle` | `true` writes parse traffic to `/data/support_bundle.jsonl` (max 2000 entries). Survives restarts. First boot also via `KLAR_SUPPORT_BUNDLE=1`. |
| `support_bundle_raw_text` | `true` keeps raw text and speech in downloads. Off by default. Conversation IDs are always hashed; entity and area names are always pseudonymized. |
| `confirm_risky_actions` | `true` requires confirmation before risky actions such as locking/unlocking and broad safety-relevant controls. |
| `semantic_adapters` | `true` consults local typed adapters after a ranking reject. Off by default. Proposals are revalidated; they never override Execute/Confirm/Clarify/Chat. |

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

### `GET /api/dashboard`

Operator dashboard for the React UI:

- `counts`: full graph, Assist-visible devices, rooms, open mappings, `high`/`medium`/`low`, bundle entries
- `coverage`: funnel `all` → `assist` → `high` plus `leftover`
- `rooms`: room readiness with open items per room
- `assignment`: devices with `confidence`, `suggested_area`, and reasons
- `traffic`: bundle aggregates, daily trend, recent sentences for replay

### `GET` / `POST /api/ui`

Persistent UI state under `/data/klar_nlu.json`:

```json
{
  "tab": "dashboard",
  "locale": "en",
  "dismissed": ["light.hue_play_1"],
  "last_apply": [],
  "graph": { "light.schlafzimmer": { "x": 120.0, "y": 40.0 } }
}
```

`POST` requires the write token like other write endpoints, except from loopback.

### `POST /api/assignment/apply`

Applies all non-dismissed room suggestions with score ≥ 3. The last batch is stored in `ui.last_apply`.

### `POST /api/assignment/undo`

Reverts the last auto-apply batch and clears `ui.last_apply`.

### `GET /api/bundle`

Support-bundle status: `enabled`, `count`, `bytes`.

### `GET /api/bundle/entries`

Stored recordings (newest first, max 400). Fields: `id`, `ts_ms`, `source`, `text`, `speech`, `intents`.

### `POST /api/bundle/entries`

Delete selected rows: `{ "ids": ["…"] }`. Response matches `GET /api/bundle/entries`.

### `GET /api/bundle/dataset`

Download as voice-suite YAML (`klar-assist-dataset.yaml`).

### `GET /api/bundle/protocol`

Redacted JSONL (`klar-support-bundle.jsonl`). Conversation IDs are hashed and entity/area names are pseudonymized. Without `support_bundle_raw_text` the utterance is the token replay string and speech is empty.

### `POST /api/bundle/clear` / `DELETE /api/bundle`

Clear the protocol.

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
