# Trainer prompt — ADR 0001 stage 4

The Klar engine does **not** call a model. An operator UI or an external agent:

1. `GET /api/v2/policies/trainer-context?layer=all&language=<Assist tag>`
2. Propose JSON for one lane (`match`, `language`, `house`) or `all`
3. `POST /api/v2/policies/propose/validate`
4. Apply only after `ok: true`, using the write API of that lane (`POST /api/v2/policies`, `POST /api/lang/overlay`)

`prompt_version` in the context payload is `1`. Pin `language` to the bound Assist locale. Do not assume a German house.

## What you may write

| Lane | JSON | Must not |
|------|------|----------|
| `match` | `{ "layer": "match", "match_controls": [{ "id", "enabled", "precedence"? }] }` | New matcher ids (`media_new_matcher` and anything outside `schema.match_ids`) |
| `language` | `{ "layer": "language", "language_overlay": { "sets": { "<path>": { "add": [], "remove": [] } } } }` | Verb flips; fillers/particles/`on`/`off` of **this** locale; unknown `set` paths |
| `house` | `{ "layer": "house", "policies": [PolicyRule…] }` | Effects outside `schema.effects`; entities/areas/floors not on `graph`; more than `schema.max_rules` rows |

Same `id` as a govern seed (`seed:confirm-lock`, `seed:confirm-cover-close`, `seed:block-area-lock`) **replaces** that seed. To turn a seed off, post a house row with that id and `enabled: false`. Do not invent new `PolicyId` matchers.

Slang belongs in the lexicon overlay of the **bound** pack, not in `when.phrase`.

## Validate contract

A proposal is invalid when any `errors` item is present. `warnings` (for example disabling `area_command`) do not block apply. `dry_run` rows are locale-scoped parses plus lock/cover plans when those entities exist on the graph.

The compiled risky **floor stays on**. Turning `seed:confirm-lock` off still confirms locks until a later setting removes the floor.

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

Copy entity ids from `graph.entities` in the context response. Copy set paths from the language lane (`nouns.*`, `cues.*`).
