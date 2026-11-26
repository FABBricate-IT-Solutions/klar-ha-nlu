# API

[Deutsch](../api.md) · [English](api.md)

Two interfaces: HTTP on port **10520**, Wyoming on **10500**.

**Breaking:** `POST /api/parse` is gone. Clients and the Home Assistant integration must use `POST /api/v2/parse` and `schema_version: "2.0"`. Upgrade the engine and the integration together.

## HTTP

Route index generated from the Axum router (`python3 scripts/gen_api_routes.py`). Auth is the capability from `src/io/auth.rs`, or `public` when the route sits outside the token middleware (health and `/`).

<!-- BEGIN GENERATED ROUTES -->

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| `GET` | `/` | public | React operator UI (or redirect into `web/dist`). |
| `POST` | `/api/assignment/apply` | Write | Apply room-assignment suggestions. |
| `POST` | `/api/assignment/undo` | Write | Undo the last assignment batch. |
| `DELETE` | `/api/bundle` | Write | Support-bundle status; DELETE clears it. |
| `GET` | `/api/bundle` | Read | Support-bundle status; DELETE clears it. |
| `POST` | `/api/bundle/clear` | Write | Clear the support-bundle protocol (POST alias). |
| `GET` | `/api/bundle/dataset` | Read | Download marked-wrong cases as nlu_expect YAML with redacted home snippets. |
| `GET` | `/api/bundle/entries` | Read | List or delete recorded turns. |
| `POST` | `/api/bundle/entries` | Write | List or delete recorded turns. |
| `GET` | `/api/bundle/protocol` | Read | Download the redacted JSONL protocol. |
| `GET` | `/api/custom` | Read | Custom phrase list. |
| `POST` | `/api/custom` | Write | Custom phrase list. |
| `GET` | `/api/dashboard` | Read | Operator dashboard payload for the React UI. |
| `GET` | `/api/entities` | Read | Home graph listing; POST tags / aliases / preferred / area. |
| `POST` | `/api/entities` | PersistParse | Home graph listing; POST tags / aliases / preferred / area. |
| `GET` | `/api/gaps` | Read | Calibration leftovers and rooms. |
| `POST` | `/api/lang/explain` | Read | Preview plus stages / evidence / matched custom. |
| `GET` | `/api/lang/overlay` | Read | User language overlay + custom sentences. |
| `POST` | `/api/lang/overlay` | Write | User language overlay + custom sentences. |
| `POST` | `/api/lang/preview` | Read | Parse with unsaved overlay; does not install. |
| `POST` | `/api/lang/rollback` | Write | Restore a previous overlay history row. |
| `GET` | `/api/settings` | Read | Engine settings store (GET/POST full, PATCH partial). |
| `PATCH` | `/api/settings` | Write | Engine settings store (GET/POST full, PATCH partial). |
| `POST` | `/api/settings` | Write | Engine settings store (GET/POST full, PATCH partial). |
| `GET` | `/api/ui` | Read | Persistent UI chrome state under the overlay. |
| `POST` | `/api/ui` | Write | Persistent UI chrome state under the overlay. |
| `GET` | `/api/v2/conversations` | Read | Conversation journal (last 200 turns / 24 h). |
| `GET` | `/api/v2/conversations/live` | Read | SSE stream of journal turns (snapshot, then each new turn). |
| `POST` | `/api/v2/conversations/mark-wrong` | Write | Mark a journal turn with the intended intent/target; export an nlu_expect case. |
| `GET` | `/api/v2/conversations/{id}` | Read | Turns for one conversation id. |
| `GET` | `/api/v2/health` | public | Public build identity, home age, overlay, LLM, Wyoming, ops counters (no token). |
| `POST` | `/api/v2/home` | Write | Live home-graph push from the Home Assistant integration. |
| `GET` | `/api/v2/intents` | Read | Known intent name list. |
| `GET` | `/api/v2/languages` | Read | Locales that can be picked (tier + offered). |
| `GET` | `/api/v2/last-turn` | Read | Newest journal turn, or null. |
| `POST` | `/api/v2/llm/assist` | Write | Engine-owned Yarn/chat/RAG/calendar/news assist. |
| `POST` | `/api/v2/llm/chat` | Write | Raw chat / SSE against the engine LLM. |
| `POST` | `/api/v2/llm/custom-voice` | Write | Generate a custom voice block via the endpoint LLM. |
| `GET` | `/api/v2/llm/endpoint` | Read | OpenAI-compatible upstream config (never returns the key). |
| `POST` | `/api/v2/llm/endpoint` | Write | OpenAI-compatible upstream config (never returns the key). |
| `POST` | `/api/v2/llm/models` | Write | List models from a caller-named upstream (write). |
| `POST` | `/api/v2/llm/refine` | Write | Engine-owned refine of finished NLU speech. |
| `GET` | `/api/v2/llm/voice` | Read | Personality preview (flavor + refine prompt). |
| `POST` | `/api/v2/parse` | Read | V2 parse (`schema_version: "2.0"`). |
| `GET` | `/api/v2/policies` | Read | Rules tab policy bundle. |
| `POST` | `/api/v2/policies` | Write | Rules tab policy bundle. |
| `GET` | `/api/v2/policies/catalog` | Read | Built-in match catalog and safety seeds. |
| `POST` | `/api/v2/policies/evaluate` | Read | Dry-run parse against optional policies. |
| `GET` | `/api/v2/policies/history` | Read | List prior house-rule bundle snapshots. |
| `POST` | `/api/v2/policies/preview-turns` | Read | Count how many recent journal turns a draft rule would match. |
| `POST` | `/api/v2/policies/propose/validate` | Read | Validate a trainer proposal without applying. |
| `POST` | `/api/v2/policies/rollback` | Write | Restore a prior house-rule bundle from history. |
| `GET` | `/api/v2/policies/trainer-context` | Read | Trainer context stub for the Rules LLM. |
| `POST` | `/api/v2/policies/trainer/chat` | Write | Trainer chat SSE (write token). |
| `POST` | `/api/v2/policies/trainer/consent` | Write | Approve or deny a pending trainer write tool. |
| `GET` | `/api/v2/result-metrics/export` | Read | Redacted weekly result metrics (decision mix, top rejections, matcher hit rates). |
| `POST` | `/api/v2/session/undo-priors` | Write | Stamp prior brightness/position/temperature/volume onto the last execute so undo can restore them. |
| `GET` | `/api/v2/settings/backup` | Write | Download allowlisted settings tar.gz. |
| `GET` | `/api/v2/settings/history` | Read | List prior engine settings snapshots. |
| `POST` | `/api/v2/settings/restore` | Write | Restore from a settings backup archive. |
| `POST` | `/api/v2/settings/rollback` | Write | Restore prior engine settings from history. |
| `GET` | `/api/v2/setup/token` | Read | Supervisor/loopback claim of the auto-minted write token (fresh App install). |
| `GET` | `/api/v2/speech/asr_boost` | Read | Ranked Whisper bias terms for external STT. |
| `POST` | `/api/v2/speech/render` | Read | Post-execute speech from an HA state snapshot. |
| `GET` | `/api/v2/wrong-cases` | Read | List operator-marked regression cases. |
| `GET` | `/api/v2/wrong-cases/dataset` | Read | Download marked-wrong cases as nlu_expect YAML. |
| `GET` | `/api/v2/wrong-cases/dry-run` | Read | Dry-run saved per-home regression cases against the live engine. |
| `GET` | `/api/v2/wrong-cases/update-dialog` | Read | After an engine update: how many saved cases changed, and which sentences. |
| `POST` | `/api/v2/wrong-cases/update-dialog` | Write | After an engine update: how many saved cases changed, and which sentences. |

<!-- END GENERATED ROUTES -->

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

`text` is limited to 4096 characters. The HTTP body is limited to 16 KiB. `news_topic` appears next to `briefing` only for a news briefing that named a topic, and is omitted otherwise.

`language` is optional (`de`, `en`, `fr`, or a BCP-47 tag such as `en-US`). When set, Klar binds only that pack for the request. `speech` follows the pinned pack.

`personality` is optional and prefixes `speech` on this endpoint (`Sehr wohl.`, `Aye.`, …). The engine settings store is the source of truth (`GET`/`POST /api/settings`); the operator UI Settings page is the editor. Home Assistant omits personality on parse when it already has stored engine settings, and only falls back to leftover integration options if that cache is empty. LLM refine then rewrites that sentence in the selected voice and does not stamp the cue back on.

`decision.type` is one of `execute`, `clarify`, `confirm`, `reject`, `chat`, or `error`. Only `execute` contains `plan`, complete `candidates`, and `selected_candidate_id`; clients must execute intents only for that decision. For every other decision, `candidates` is empty and no plan, intent or slot is serialized. `confirm` contains a prompt that names action and target ("Should I really unlock Front Door?") and an opaque candidate ID; `evidence` and `trace` still say what was heard (for example `action:off`, `target_entity:lock.front_door`) so the decision can be explained, but they are not executable. The proposal remains exclusively in the session until the same `conversation_id` answers affirmatively. An open `confirm` or `clarify` question lasts 60 seconds, is answered once, and is replaced by a complete new command. A `conversation_id` that is empty, longer than 128 characters or contains control characters never names a shared session: `/api/parse` treats an empty id as absent and answers `400` for the other two.

Confidence is evidence-backed and comparable across intents. The engine applies fixed bands: execute at confidence ≥ 0.80 and margin ≥ 0.05 when no competing complete plan is closer than 0.05; confirm for risky lock/unlock/cover-close, for any step that moves a garage door, gate or door (by device class, else by name; opening and toggling included), or plans of eight or more steps that are not only lights when confidence ≥ 0.62; clarify when competing complete plans are closer than 0.05 or confidence is between 0.70 and the execute band; reject below 0.70 (below 0.62 when risky), for out-of-domain talk (weather, trivia, empty fillers), or when no grounded step remains. Fuzzy, session, and inferred evidence cannot outscore exact lexical+resolver evidence, and inferred actions never claim 1.0. Multi-clause plans keep only independently valid steps. A pending question is answered only by the active language's own yes, no and cancel words, checked on the words as spoken: after a no or cancel word it is dropped; after a yes word the stored plan is re-validated against the current home graph. An either-or `clarify` does not accept a yes; it takes one of its options, named word for word. `chat` stays limited to news, briefing follow-ups, and explicit LLM opt-in. General questions that are not home commands reject with reason `out_of_domain`; Home Assistant forwards only that reason to the configured LLM and still speaks the reject line when no model answers. A failed device command stays a normal reject.

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

### `GET` / `POST` / `PATCH /api/settings`

`GET` returns the full settings object. `POST` replaces it. `PATCH` merges a partial object (unknown keys → `422`).

```json
{
  "personality": "default",
  "mode": "full",
  "languages": ["de", "en"],
  "support_bundle": false,
  "support_bundle_raw_text": false,
  "confirm_risky_actions": true,
  "semantic_adapters": false,
  "web_lookup": false,
  "news_briefing": false,
  "nlu_rag": false
}
```

| Field | Values |
|-------|--------|
| `personality` | `default`, `butler`, `locker`, `fuersorglich`, `party`, `grantig`, `sarkastisch`, `pirat`, `hippie`, `gollum` |
| `mode` | `full` (resolve devices) or `context_only` (areas only) |
| `languages` | Pack codes. Unknown codes are ignored. Empty means every compiled locale may be pinned; a request without a language then binds `default_language`. Do not merge all lexicons — token collisions break Assist. Adding an experimental locale while `experimental_languages` is off is refused with `422`; one that is configured already may stay. |
| `default_language` | The pack a request binds when `languages` is empty and the request names no language. Empty means English. A code no pack answers is refused with `422`. |
| `experimental_languages` | `true` offers experimental locales for a new selection (see [languages](languages.md#tiers)). Off by default. |
| `support_bundle` | `true` writes parse traffic to `/data/support_bundle.jsonl` (max 2000 entries). Survives restarts. First boot also via `KLAR_SUPPORT_BUNDLE=1`. |
| `support_bundle_raw_text` | `true` keeps raw text, speech, and tokens in downloads and the journal. Off by default. Conversation IDs are always hashed; entity and area names are always pseudonymized. |
| `confirm_risky_actions` | `true` requires confirmation before risky actions such as locking/unlocking and broad safety-relevant controls. |
| `semantic_adapters` | `true` consults local typed adapters after a ranking reject. Off by default. Proposals are revalidated; they never override Execute/Confirm/Clarify/Chat. |
| `nlu_rag` | `true` attaches a matched-slice retrieval on `chat` and `reject` only. Off by default. Never Assist tools; the HA fallback may recover a command only through Klar tools. `POST /api/v2/parse` can set `nlu_rag` per request. |
| `web_lookup` | `true` allows Wikipedia/Wikidata on out-of-domain chat. Off by default. |
| `news_briefing` | `true` allows RSS fetches for news briefings. Off by default. |
| `refine_speech` | `true` rewrites finished NLU speech with the engine LLM. Off by default. |
| `extra_prompt` | House rule as a user message. Empty uses the pack voice only. Does not replace personality. |
| `quiet_ack` | `true` chimes instead of TTS on simple on/off. Off by default. |
| `satellite_followup` | `true` keeps the satellite microphone open after a finished chat reply. Off by default, so a shared mic and speaker does not hear its own speech. Clarify and confirm stay open either way. Typed Assist has no satellite id and is unchanged. |
| `calendar_llm` | `true` lets the engine LLM speak calendar events. Off by default. |
| `news_feeds` | List of `{ "pack", "url" }`. One `http` or `https` URL per pack code, no userinfo, at most eight entries. Empty uses the built-in feeds — fetch only when `news_briefing` is on. |
| `allow_llm_tools` | `true` lets the engine chat model call Home Assistant Assist tools after Klar parse (2026.9 prefixed names). Off by default. |

### `GET` / `POST /api/v2/llm/endpoint`

OpenAI-compatible upstream for the engine. `GET` returns `{ "configured", "base_url", "model" }` — never the API key. `POST` sets `base_url`, `api_key`, `model` in `data_dir/llm_endpoint.json` (not the overlay). Empty `api_key` keeps the stored key. `configured: false` deletes the file. `KLAR_LLM_*` env wins on boot. Configure it in the operator UI; Assist does not need another Home Assistant LLM integration. `nlu::parse` does not use it.

### `POST /api/v2/llm/chat`

```json
{ "messages": [{ "role": "user", "content": "…" }], "stream": true, "temperature": 0.2, "max_tokens": 2048 }
```

Write token required (same as overlay). `stream: true` (default) emits SSE `data: {"type":"delta"|"done"|"error",…}`. `stream: false` returns JSON `{"type":"done","text":"…"}`. `503` when no endpoint is configured.

### `POST /api/v2/llm/refine`

```json
{ "speech": "Wohnzimmer Licht ist an.", "language": "de", "personality": "butler", "extra_prompt": "", "stream": false }
```

The engine builds the refine system prompt (pack + voice only) and sends extra as a user message, runs the model (`temperature` 0.65, `max_tokens` scales with the original length, min 192, max 4096), and applies `accept_refined`. JSON `{"type":"done","text":"…","accepted":true}`. If accept rejects, `text` is the original and `accepted` is false. Caps: `speech` ≤ 4096, `extra_prompt` ≤ 2048. Write token required. `503` when no endpoint. Do not send a Python-built system prompt.

### `POST /api/v2/llm/assist`

```json
{
  "text": "erzähl einen Witz",
  "language": "de",
  "personality": "butler",
  "kind": "auto",
  "allow_tools": false,
  "nlu_rag": false,
  "retrieval": null,
  "facts": null,
  "history": [["user", "…"], ["assistant", "…"]],
  "extra_system": null,
  "stream": true
}
```

Engine owns Yarn/chat/RAG/calendar/news prompts and `yarn_canned` / `yarn_nudge`. `kind`: `auto` | `yarn` | `chat` | `rag` | `calendar` | `news` | `news_follow`. `auto` uses `yarn_request` / the RAG flag. `facts` is a headline list or calendar readback gathered by HA. Personality is applied here — do not also prepend `refine_prompt`. SSE adds `{"type":"tool","tool":"klar.parse","text":"licht an"}` / `klar.act` so TTS never speaks `KLAR_PARSE:`. Write token required. `503` when no endpoint. Missing route fails closed — Home Assistant does not rebuild the Python prompt.

### `GET /api/v2/speech/asr_boost`

Ranked bias list for external Whisper (`wyoming-faster-whisper --initial-prompt`). Klar does **not** host STT/TTS or Whisper/Piper. Query: `language` (pack code, default from settings), `max_tokens` (default 200, cap 223). Auth like other read APIs (Supervisor/loopback without a token, otherwise a token).

Hard names only: fuzzy-prone entity names/aliases (short, umlauts, cover/curtain class, hyphen/digits) and custom-phrase anchors. Not included: common rooms, pack verbs/device words (`light`, `on`, …), full Assist sentences, live state, secrets, infra, unexposed entities.

```json
{
  "schema_version": "1",
  "language": "de",
  "max_tokens": 200,
  "prompt": "Kugel Vorhang Rollo Studio-Vorhang Papiertonne …",
  "terms": [{"text": "Vorhang", "tier": 3, "entity_id": "cover.studio_vorhang"}],
  "dropped": 42,
  "updated_at": "a1b2c3…"
}
```

`updated_at` is a content fingerprint (not wall-clock). The integration writes the prompt after home-graph sync to `config/klar_nlu_asr_boost.txt` and via service `klar_nlu.export_asr_boost`. Setup: [Home Assistant — ASR boost](home-assistant.md#asr-boost-whisper).

### `POST /api/v2/speech/render`

Post-execute snapshot from Home Assistant. The engine interpolates pack templates into a factual sentence (`source: "post_execute"`). Personality prefix is applied later at Assist finish. Assist calls this after execute; missing route fails closed (no Python `from_handled`). Same auth as parse: Supervisor network and loopback without a token, otherwise a token.

```json
{
  "schema_version": "1",
  "language": "de",
  "personality": "default",
  "now": "2026-09-05T19:22:00+02:00",
  "intent": {
    "name": "HassTurnOn",
    "slots": [{"name": "area", "value": "wohnzimmer"}]
  },
  "outcome": "success",
  "entities": [
    {
      "entity_id": "light.wohnzimmer",
      "name": "Wohnzimmer",
      "domain": "light",
      "state": "on",
      "area": "wohnzimmer",
      "area_name": "Wohnzimmer",
      "device_class": null,
      "attributes": {
        "current_temperature": null,
        "temperature_unit": null,
        "unit_of_measurement": null,
        "hvac_action": null,
        "hvac_mode": null,
        "volume_level": null,
        "is_volume_muted": null,
        "media_title": null,
        "media_artist": null,
        "media_album_name": null
      }
    }
  ],
  "calendar_events": [],
  "media_queue": []
}
```

Response: `{ "speech": "The living room light is on.", "quiet_ack": false, "source": "post_execute" }`. `outcome` is `success` | `partial` | `error`. `now` is required (clock). Unknown attribute keys are dropped. Missing `schema_version` → `400`. Caps: 32 entities, 16 calendar rows, 8 queue titles, attribute values ≤ 256 characters. Missing climate/MASS attributes use the unavailable line — never invent a value.

### `POST /api/v2/policies/trainer/chat`

```json
{ "message": "Treat Funzel as a light noun", "layer": "language", "language": "de", "history": [] }
```

The engine loads trainer context, streams the model, extracts JSON, and appends `proposal` plus `validate` to the stream. Apply remains a human write call. See [trainer-prompt.md](../architecture/trainer-prompt.md).

### `POST /api/v2/home`

Live home-graph snapshot from the Home Assistant integration. The complete request has an 8 MiB byte ceiling, separate from the 16 KiB budget for ordinary JSON routes. Collection and field caps still apply; unauthorized requests are rejected before the body is read. `schema_version` must be `"1"`. Caps and error codes: [home-assistant.md](home-assistant.md#registry-sync-ha-is-the-source-of-truth). After a valid push, HA is the live source; the `.storage` watcher no longer overwrites that graph.

### `GET /api/v2/languages`

The locales that can be picked now, with `code`, `native_name`, `script`, `variants`, `tier` (`reference`, `beta`, or `experimental`) and `offered`. Reference and Beta locales are always listed; experimental ones when `experimental_languages` is on or when they are configured already. `?all=true` lists every compiled locale; `offered` is `false` for the ones the switch keeps out of a new selection.

### `GET` / `POST /api/lang/overlay`

User language overlay plus custom sentences, persisted under `/data/klar_nlu.json`.

```json
{
  "custom": [{ "phrase": "filmabend", "intent": "HassTurnOn", "slots": { "entity_id": "scene.film" } }],
  "language": { "sets": {} },
  "label": "save"
}
```

Response includes `history` (`hash`, `label`, `saved_at`). Omitting `language` on POST keeps the stored set deltas, omitting `custom` keeps the stored phrases; `"custom": []` clears them. Invalid custom/overlay → `422`. Write token required off-loopback.

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

Calibration view for entities with weak names, missing areas, or no helpful aliases/tags. The response contains `leftover` and `rooms` (including each room’s `floor_id`). It does not include the overlay.

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

`locale` is operator chrome only. A saved Settings pick (`locale_set: true`) wins. Until then `GET` uses `KLAR_UI_LOCALE`, then `en`. This is not the Assist pin and not `Accept-Language`.

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

Redacted JSONL (`klar-support-bundle.jsonl`). Conversation IDs are hashed and entity/area names are pseudonymized. Without `support_bundle_raw_text` text, speech, and tokens are empty.

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
