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

`language` is optional (`de`, `en`, `fr`, or a BCP-47 tag such as `en-US`). When set, Klar binds only that pack for the request. `speech` follows the pinned pack.

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
  "semantic_adapters": false,
  "nlu_rag": false
}
```

| Field | Values |
|-------|--------|
| `personality` | `default`, `butler`, `locker`, `fuersorglich`, `party`, `grantig`, `sarkastisch`, `pirat`, `hippie`, `gollum` |
| `mode` | `full` (resolve devices) or `context_only` (areas only) |
| `languages` | Pack codes. Unknown codes are ignored. Empty means every compiled locale is enabled; the catalog still binds per request. Do not merge all lexicons — token collisions break Assist. |
| `support_bundle` | `true` writes parse traffic to `/data/support_bundle.jsonl` (max 2000 entries). Survives restarts. First boot also via `KLAR_SUPPORT_BUNDLE=1`. |
| `support_bundle_raw_text` | `true` keeps raw text and speech in downloads. Off by default. Conversation IDs are always hashed; entity and area names are always pseudonymized. |
| `confirm_risky_actions` | `true` requires confirmation before risky actions such as locking/unlocking and broad safety-relevant controls. |
| `semantic_adapters` | `true` consults local typed adapters after a ranking reject. Off by default. Proposals are revalidated; they never override Execute/Confirm/Clarify/Chat. |
| `nlu_rag` | `true` attaches a matched-slice retrieval on `chat` and `reject` only. Off by default. Never Assist tools; the HA fallback may recover a command only through Klar tools. `POST /api/v2/parse` can set `nlu_rag` per request. |

### `POST /api/v2/home`

Live home-graph snapshot from the Home Assistant integration. `schema_version` must be `"1"`. Caps and error codes: [home-assistant.md](home-assistant.md#registry-sync-ha-is-the-source-of-truth). After a valid push, HA is the live source; the `.storage` watcher no longer overwrites that graph.

### `GET /api/v2/languages`

Compiled pack metadata (`code`, `native_name`, `script`, `variants`). This is the first-class Assist locale list in the binary.

### `GET` / `POST /api/lang/overlay`

User language overlay plus custom sentences, persisted under `/data/klar_nlu.json`.

```json
{
  "custom": [{ "phrase": "filmabend", "intent": "HassTurnOn", "slots": { "entity_id": "scene.film" } }],
  "language": { "sets": {} },
  "label": "save"
}
```

Response includes `history` (`hash`, `label`, `saved_at`). Omitting `language` on POST keeps the stored set deltas. Invalid custom/overlay → `422`. Write token required off-loopback.

### `POST /api/lang/preview`

Parse with optional unsaved `custom` and `language_overlay`. Does not install the overlay. Same text limit as parse (`413` if too long). Optional `language` pins one pack (`422` if unknown).

### `POST /api/lang/explain`

Same body as preview. Returns `decision`, `confidence`, `speech`, `stages`, `evidence`, and `matched_custom` — no live overlay install.

### `POST /api/lang/rollback`

`{ "hash": "…" }` restores that history row. Omit `hash` to roll back to the latest stored revision. `404` if nothing to restore.

### `GET` / `POST /api/v2/policies`

Policy bundle for the **Rules** tab: `{ "policies": […], "speech_bank": { "entries": […] } }`.

Each rule: `id`, `enabled`, `label`, `when` (optional `intent` / `domain` / `area` / `entity_id` / `floor` / `name` / `phrase`), `effect`, optional `prefer` / `payload`. Effects: `confirm`, `block`, `allow`, `prefer_entity`, `prefer_area`, `reply`, `script`, `template`, `llm`. At most 64 rules. Invalid body → `400`. POST persists into the overlay.

### `POST /api/v2/policies/evaluate`

Dry-run parse with optional `policies` (else the stored set). Body: `{ "text", "language?", "policies?" }`. Response: `outcome`, `compiled_risky`, `matched_rule`, `hit`, `speech_variant`.

### `GET /api/v2/conversations`

Conversation journal: last **200** turns, **24 hours**, file `/data/conversations.jsonl`. Fields: `conversation_id`, `ts_ms`, optional `text`, `decision`, `speech`, `confidence`, `briefing`, `evidence_kinds`, `last_names`, optional `confirm_prompt` / `candidate_id`. Raw `text` is stored only when `support_bundle_raw_text` is on. Confirm/clarify never include a plan.

### `GET /api/v2/conversations/{id}`

Turns for one `conversation_id` (`400` if the id is longer than 128 characters).

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

React operator UI built from `web/` (Home, Conversations, Rules, House / Mapping, Lab, Settings). Served from `/usr/share/klar/ui` in the image or `web/dist` after `npm run build`. `web/index.html` is the Vite mount, not a standalone test page.

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
