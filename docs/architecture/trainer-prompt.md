# Trainer prompt — ADR 0001 stage 4

The Klar engine does **not** parse with a model. Chat completions live in Rust (`src/llm/`). An operator UI:

1. `GET /api/v2/llm/endpoint` — `configured` must be true (operator UI Settings → LLM, or `KLAR_LLM_BASE_URL` / `KLAR_LLM_API_KEY` / `KLAR_LLM_MODEL`). No Home Assistant conversation integration is required.
2. `POST /api/v2/policies/trainer/chat` with `{ "message", "layer"?, "language"?, "history"? }` — SSE events `delta`, `consent`, `session`, `done`
3. Writes wait for chat consent: **Allow once** (this call), **Allow** (this tool name for the session), **YOLO** (all trainer writes this session), or **Deny**. `POST /api/v2/policies/trainer/consent` `{ "call_id", "decision": "allow_once"|"allow"|"yolo"|"deny"|"ask_again" }`
4. The server `validate()`s every write, including under YOLO. Overlays **merge**, they do not replace.

Read tools run immediately (`list_languages`, `search_house`, `get_entity`, `list_lexicon_paths`, `get_lexicon`, `list_matchers`, `list_policies`, `list_gaps`, `validate_proposal`). Write tools: `apply_lexicon`, `apply_match`, `apply_house`, `apply_aliases`.

`prompt_version` in the chat stub is `2`. The stub lists `settings.languages`, schema ids, and gap counts. Details come from tools. Do not assume a German house.

Personality / refine voice never enters the trainer system prompt. Extra prompt is a user message on Assist/refine, not a voice replacement.

Manual loop (debug): `GET /api/v2/policies/trainer-context` → JSON → `POST /api/v2/policies/propose/validate`.

## What you may write

| Lane | Tool | Must not |
|------|------|----------|
| `match` | `apply_match` with known `schema.match_ids` | New matcher ids |
| `language` | `apply_lexicon` add/remove on known SET_KEYS paths | Verb flips; fillers/particles/`on`/`off` of **this** locale |
| `house` | `apply_house` PolicyRule upsert by `id` | Effects outside `schema.effects`; entities/areas/floors not on the graph |
| aliases | `apply_aliases` | Entities not on the graph |

Same `id` as a govern seed (`seed:confirm-lock`, `seed:confirm-cover-close`, `seed:block-area-lock`) **replaces** that seed. To turn a seed off, post a house row with that id and `enabled: false`. Do not invent new `PolicyId` matchers.

Slang belongs in the lexicon overlay of the **bound** pack, not in `when.phrase`.

## Validate contract

A write is rejected when any `errors` item is present. `warnings` do not block apply. `dry_run` rows are locale-scoped parses plus lock/cover plans when those entities exist on the graph.

The compiled risky **floor stays on**. Turning `seed:confirm-lock` off still confirms locks until a later setting removes the floor.

## Assist tools (HA 2026.9)

When `allow_llm_tools` is on, Klar's conversation entity calls `chat_log.async_provide_llm_data` and forwards `chat_log.llm_api.tools` with Core names (`intent__HassTurnOn`, `homeassistant__GetLiveContext`). Never hardcode the old unprefixed names. Tools run only on chat/reject fallback after Klar parse, not in parallel with execute. `nlu_rag` and HA tools are exclusive in one round.

## Example (house prefer)

```json
{
  "layer": "house",
  "language": "de",
  "policies": [
    {
      "id": "prefer-decke",
      "enabled": true,
      "label": "Wohnzimmer ceiling",
      "when": { "domain": "light", "area": "wohnzimmer" },
      "effect": "prefer_entity",
      "prefer": "light.wohnzimmer_decke"
    }
  ]
}
```

Copy entity ids from `search_house` / `get_entity`. Copy set paths from `list_lexicon_paths`.
