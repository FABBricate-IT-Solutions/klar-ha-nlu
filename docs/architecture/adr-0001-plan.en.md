# Implementation plan — ADR 0001

[Deutsch](adr-0001-plan.md) · [English](adr-0001-plan.en.md)

Frame: [ADR 0001](adr-0001-rules-and-trainer.en.md). Each stage is its own PR. Defaults stay today’s Assist behavior until someone changes a lane. No calendar — order follows dependencies and risk.

## Locked decisions

| Topic | Decision |
|-------|----------|
| Match | compiled + overlay (`enabled`, `precedence`), no matching DSL |
| Lexicon | pack = seed DB; slang only `LanguageOverlay` `SetDelta` |
| Govern seed | `PolicyRule[]` per locale; house replaces by id |
| Trainer | operator UI only; engine has no net; propose = context + validate |
| `compiled_risky` | floor on until seed parity is bit-identical |
| `origin: trainer` | stays visible |
| Precedence | save allowed, evaluate warns, reset is one click |
| Apply | confirmed per lane |
| Lexicon tokens | preview required; apply only after a green dry-run or an explicit override |
| Overlay paths | extend nouns/cues; verbs only as new tokens |
| Household → seed | not in v1 |

## Stage 0 — contract, no behavior change

Goal: the data the UI will draw already exists in JSON. Parse outcome and scorecard unchanged.

**API (additive, `schema_version` stays `2.0`)**

Extend `PolicyTrace` with optional fields (`skip_serializing_if`):

```json
{
  "match": { "id": "area_command", "score": 0.93, "origin": "engine" },
  "seed": null,
  "house": { "id": "prefer-ceiling", "hit": "prefer_entity", "origin": "operator" },
  "band": "execute",
  "compiled_risky": false,
  "discarded": [{ "id": "grounded_entities", "score": 0.88, "reason": "lower_score" }]
}
```

`GET /api/v2/policies/catalog` — read-only match catalog from `PolicyId` (id, precedence, summary). No overlay.

**Code:** `src/types/outcome.rs`, `src/nlu/draft.rs` `safety_decision`, `src/parse/policy.rs` (catalog rows), `web/src/types.ts`, `web/src/parseContract.ts`, `tests/contract.rs`.

**Gate:** `cargo nextest run --locked --test contract --test policy`; web contract accepts the new optional keys. DE/EN voice suites unchanged.

**Risk:** low. Old clients ignore unknown fields; confirm/clarify still never serialize a plan.

## Stage 1 — path visible, lanes rigid

Goal: the Rules tab shows three columns. Evaluate and Lab draw the same path. No match/seed toggles yet.

**UI**

- `web/src/components/PolicyPath.tsx` — three required nodes + band; `—` = checked, not hit; click sets active lane + row.
- `RulesPage`: three-column grid. Match from catalog (read-only). Language: lexicon deltas from `GET /api/lang/overlay` (read-only) + empty govern list. House: today’s editor.
- Lab: replace `.flow` / `processPath` with `PolicyPath` (read, do not write).
- Strings in `web/src/i18n/en.ts` and `de.ts`; other locales fall back to `en`.

**Gate:** browser pass on Rules + Lab with “turn on the living room lights” and a lock sentence. Contract still green.

**Risk:** low. No parse rewrite. On narrow screens stack the columns, one open.

## Stage 2 — match and lexicon controls

Goal: the operator can toggle Match and drag precedence; lexicon deltas write from the same lane. Defaults = today’s behavior.

**Overlay** in `klar_nlu.json`:

```json
"match_controls": [{ "id": "media", "enabled": false, "precedence": 3 }]
```

Unknown id → `400`. Missing row = engine default. Reset = delete the row.

**Code:** `src/home/overlay.rs`, `src/io/policies.rs` (bundle grows `match_controls`), `src/parse/clause.rs` (skip disabled, precedence from overlay), `src/parse/policy.rs`. Lexicon: call existing `POST /api/lang/overlay` from the Language lane; extend `src/lang/validate.rs` `set_field` with missing nouns/cues (not verbs).

**Tests:** empty overlay ≡ today’s candidate list; `media` off → no `PolicyId::Media` candidates; locale smokes green with default overlay. `tests/language.rs` overlay add `funzel`.

**Gate:** `assist_langs`, `parity_langs`, `policy`, `language`. Evaluate warns if a disable would hit a known smoke pattern (`area_command` / `all_lights`); save still allowed.

**Risk:** medium. A bad disable breaks Assist in that house, not CI, as long as defaults stay tested.

## Stage 3 — govern seed de/en, same behavior

Goal: `risky_intent` / `allow_permitted` as visible seed rules. `compiled_risky` stays the floor.

**Data:** `src/lang/packs/de/govern.json`, `en/govern.json` — ids `seed:confirm-lock`, `seed:confirm-cover-close`, `seed:block-area-lock`. Merge: house rules in front, same id replaces seed, seeds do not count toward 64.

**Code:** `src/types/policy.rs` (optional origin/replaces), `src/nlu/draft.rs`, `src/nlu/policy_route.rs`. A seed toggle writes a house override `enabled: false` with `replaces`; it does not delete the pack.

**Tests:** the same lock/cover cases in `src/nlu/validation.rs` and `tests/policy.rs` — band identical to today, `policy_trace.seed` set when the floor is not the only hit.

**Gate:** bit-identical confirm/reject matrix vs `main` on `familienhaus_de` / `family_home_en` plus `tests/policy.rs`.

**Risk:** high. A small `when` mistake changes lock confirm. Keep the floor on until this stage is green.

## Stage 4 — seeds for the remaining locales

Goal: every compiled locale has the same safety-universal set (language-agnostic). Phrase seeds only where the pack has household phrases — not in this stage.

Generator like `scripts/lang_packs`: copy the `de`/`en` reference. Freshness check like packs.

**Gate:** `parity_langs`; confirm-lock smoke per locale when the representative set has a lock, otherwise skip.

**Risk:** low if stage 3 holds. Thin seeds beat mistranslated phrase seeds.

## Stage 5 — trainer (context + validate)

Goal: the LLM sets the house up; the engine stays without a net.

1. `GET /api/v2/policies/trainer-context?layer=` — visible graph, gaps, catalog, seed, current overlays, lane schema. No raw journal unless settings allow it.
2. UI or HA agent produces JSON (versioned prompt in-repo as `docs/architecture/trainer-prompt.md`, added in this PR).
3. `POST /api/v2/policies/propose/validate` — `sanitize_*`, grounding (entity/area/`prefer` on the graph, match id in the catalog, lexicon path in `set_field`), dry-run against house smokes + locale smokes of the bound language.
4. Drawer: diff, checkboxes; apply calls that lane’s existing write API.

**Schemas:** as in the ADR table (match / lexicon / seed / house). Unknown match id and off-graph entity → row `rejected`, not HTTP 500.

**Tests:** fixtures without an LLM: graph `familienhaus_de` → expected climate/lock proposals; `media_new_matcher` rejected; lexicon-add of particle `an` rejected.

**Gate:** unit + contract for context/validate. No live model in CI.

**Risk:** medium (prompt drift). Validate is the edge; the model may only propose.

## Stage 6 — later, not v1

- Household phrases (clock, weather, undo, explain) into the govern seed, only if the contract matches `src/nlu/household.rs`.
- Trainer as an Assist conversation.
- `compiled_risky` behind a setting once stages 3/4 stay green.
- Path chip on a conversation row.

## Order and stop rules

```
0 contract → 1 UI path → 2 match/lexicon overlay → 3 seed safety de/en
                                                    → 4 locale seeds
                                                    → 5 trainer
```

Do not parallelize with stage 3: seed merge changes `safety_decision`. Stage 1 may land on stage 0 as soon as catalog+trace exist.

Stop and revert if: DE/EN oracle suites go red, lock confirm drifts, the catalog merges locales, or the trainer saves without validate.

## Explicitly out

- New `PolicyId` from the UI or the model
- Replacing pack files at runtime
- Morphology, `NumberStyle`, tokenizer as overlay
- Merging every lexicon into one catalog
- LLM inside `nlu::parse`
- Assist tools for the trainer
