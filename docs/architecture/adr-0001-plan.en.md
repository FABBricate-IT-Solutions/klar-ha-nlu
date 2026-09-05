# Implementation plan — ADR 0001

[Deutsch](adr-0001-plan.md) · [English](adr-0001-plan.en.md)

Frame: [ADR 0001](adr-0001-rules-and-trainer.en.md). Each stage is its own PR **against `staging`**. Defaults stay today’s Assist behavior until someone changes a lane. No calendar — order follows dependencies and risk.

## Delivery channel: staging, not a main release

This work is a **long staging cycle**. None of it is a stable / CalVer release until `staging` → `main` is explicitly approved.

| What | Decision |
|------|----------|
| Base of every implementation PR | `staging` (protected, merge via PR) |
| This plan/ADR PR | `staging` as well |
| After merge to `staging` | existing staging workflow: prerelease tag `{CalVer}-staging.{sha7}`, image tag `staging`, never `latest` |
| Testing | HA **Release channel = Staging** (`http://klar-nlu-staging:10520` / GitHub prerelease) |
| `staging` → `main` | **not** part of this plan. A separate promote PR, only after a long bake |
| CalVer / `latest` | untouched until that promote is deliberate |

Staging CI runs quality+security like Release, **not** the weekly full `parity_langs` matrix. The locale invariant still holds: `assist_langs` stays a PR gate; run `parity_langs` before every seed/parse merge locally or via `workflow_dispatch` / `language-parity.yml` — not “only on main”.

No `--admin`, no direct push to `staging` or `main`.

## Locale invariant

Everything applies to **every compiled Assist locale** in `GET /api/v2/languages` (67 today, including variants such as `de-CH`, `pt-BR`, `zh-CN`, `sr-Latn`). de/en are hand-written reference packs and oracle graphs, **not a support caste**. No `match LangId` in `src/parse/`. A feature that is green only on German/English is not done.

| Layer | How every locale ships |
|-------|------------------------|
| Match | language-agnostic (`PolicyId`). Catalog ids are stable; UI copy uses operator i18n keys, not hard-coded German. |
| Lexicon | each pack is that locale’s seed DB. Overlay `SetDelta` binds to the pinned catalog, not to `de`. |
| Govern safety | `when.domain=lock` is language-free — **one** seed bundle for every pack, not 67 translations. |
| Phrase seeds / household | only via a generator for **all** packs in the same PR, like `scripts/lang_packs`. Never de/en only. |
| Trainer validate | dry-run against representative + parity **of the bound locale**, not only `familienhaus_de`. |
| Operator chrome | new keys in `de.ts`/`en.ts`; other UI JSONs fall back to `en` (existing pattern). Assist quality does not hang on chrome. |

**Gate for every stage that touches parse or seeds:** `cargo nextest run --locked --test assist_langs --test parity_langs` (full matrix, no fail-fast), plus the oracle suites. Russian stays out (no pack).

## Locked decisions

| Topic | Decision |
|-------|----------|
| Match | compiled + overlay (`enabled`, `precedence`), no matching DSL |
| Lexicon | pack = seed DB; slang only `LanguageOverlay` `SetDelta` |
| Govern seed | one language-free safety bundle for all locales; phrase seeds only generated for the full matrix |
| Locales | every compiled LangId; de/en = oracles, not extra support |
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

`GET /api/v2/policies/catalog` — read-only match catalog from `PolicyId` (`id`, `precedence`, `summary_key`). No overlay, no locale-specific copy in the engine.

**Code:** `src/types/outcome.rs`, `src/nlu/draft.rs` `safety_decision`, `src/parse/policy.rs` (catalog rows), `web/src/types.ts`, `web/src/parseContract.ts`, `tests/contract.rs`.

**Gate:** `cargo nextest run --locked --test contract --test policy --test assist_langs`; web contract accepts the new optional keys. Full `parity_langs` matrix when the stage touches parse fields; otherwise contract is enough.

**Risk:** low. Old clients ignore unknown fields; confirm/clarify still never serialize a plan.

## Stage 1 — path visible, lanes rigid

Goal: the Rules tab shows three columns. Evaluate and Lab draw the same path. No match/seed toggles yet.

**UI**

- `web/src/components/PolicyPath.tsx` — three required nodes + band; `—` = checked, not hit; click sets active lane + row.
- `RulesPage`: three-column grid. Match from catalog (read-only). Language: lexicon deltas from `GET /api/lang/overlay` (read-only) + empty govern list. House: today’s editor.
- Lab: replace `.flow` / `processPath` with `PolicyPath` (read, do not write).
- Strings in `web/src/i18n/en.ts` and `de.ts`; other locales fall back to `en`.

**Gate:** browser pass on Rules + Lab, once with `de` and once with a generated locale (e.g. `ja` or `ar`). Evaluate with `language` pinned. Contract + `assist_langs` green.

**Risk:** low. No parse rewrite. On narrow screens stack the columns, one open.

## Stage 2 — match and lexicon controls

Goal: the operator can toggle Match and drag precedence; lexicon deltas write from the same lane. Defaults = today’s behavior.

**Overlay** in `klar_nlu.json`:

```json
"match_controls": [{ "id": "media", "enabled": false, "precedence": 3 }]
```

Unknown id → `400`. Missing row = engine default. Reset = delete the row.

**Code:** `src/home/overlay.rs`, `src/io/policies.rs` (bundle grows `match_controls`), `src/parse/clause.rs` (skip disabled, precedence from overlay), `src/parse/policy.rs`. Lexicon: call existing `POST /api/lang/overlay` from the Language lane; extend `src/lang/validate.rs` `set_field` with missing nouns/cues (not verbs).

**Tests:** empty overlay ≡ today’s candidate list; `media` off → no `PolicyId::Media` candidates; locale smokes green with default overlay. `tests/language.rs` overlay add on the bound pack (not a de-only token).

**Gate:** `assist_langs`, `parity_langs`, `policy`, `language`. Evaluate warns if a disable would hit a known smoke pattern (`area_command` / `all_lights`); save still allowed. An empty overlay must **not** shift any locale versus `main`.

**Risk:** medium. A bad disable breaks Assist in that house, not CI, as long as defaults stay tested.

## Stage 3 — govern seed for every locale, same behavior

Goal: `risky_intent` / `allow_permitted` as visible seed rules on **every** bound locale. `compiled_risky` stays the floor.

**Data:** one language-free bundle, e.g. `src/lang/govern_safety.json`, bound with every pack — not 67 copies, not `de`/`en` only. Ids `seed:confirm-lock`, `seed:confirm-cover-close`, `seed:block-area-lock`. `when` is intent/domain/area only, no phrase. Merge: house in front, same id replaces seed, seeds do not count toward 64.

Phrase or household seeds do **not** belong here. If they come later: the generator writes them for `LangId::all()` in the same PR.

**Code:** `src/types/policy.rs`, `src/nlu/draft.rs`, `src/nlu/policy_route.rs`, bind in `src/nlu/context.rs` independent of `LangId`. A toggle writes a house override `enabled: false` with `replaces`.

**Tests:** lock/cover matrix in `tests/policy.rs` (de) **and** the same intents through `assist_langs` / representative per locale when the set has a lock or cover. Band identical to `main`. `policy_trace.seed` set when the floor is not the only hit.

**Gate:** `policy`, `assist_langs`, `parity_langs`; bit-identical confirm/reject oracles `familienhaus_de` / `family_home_en` stay the graph reference, not the only locales.

**Risk:** high. A small `when` mistake changes lock confirm in every language at once. Keep the floor on until the full matrix is green.

## Stage 4 — trainer (context + validate)

Goal: the LLM sets the house up; the engine stays without a net. Context and validate are **locale-scoped** (`language` on the request, else the Assist pin).

1. `GET /api/v2/policies/trainer-context?layer=&language=` — graph, gaps, catalog, seed, overlays, schema. Lexicon proposals only against the bound pack.
2. UI or HA agent produces JSON (versioned prompt, `docs/architecture/trainer-prompt.md` in this PR). The prompt names the locale, not “German house”.
3. `POST /api/v2/policies/propose/validate` — sanitize, grounding, dry-run against representative + parity **of that locale** plus house smokes.
4. Drawer: diff; apply calls that lane’s write API.

**Tests:** fixtures without an LLM for at least one reference (`familienhaus_de`) **and** one generated locale (e.g. `tests/datasets/full_home/ja` or a parity graph). `media_new_matcher` rejected; lexicon-add of a particle of the **bound** locale rejected (not hard-coded `an` for every language).

**Gate:** unit + contract; `assist_langs` unchanged. No live model in CI.

**Risk:** medium (prompt drift). Validate is the edge.

## Stage 5 — later, not v1

- Household phrases into the seed: **generator for every pack**, contract like `src/nlu/household.rs`, gate `assist_langs` + `parity_langs`.
- Trainer as an Assist conversation (pipeline language = pack).
- `compiled_risky` behind a setting once stage 3 holds on the full matrix.
- Path chip on a conversation row.

## Order and stop rules

```
0 contract → 1 UI path → 2 match/lexicon overlay → 3 seed safety (every locale)
                                                   → 4 trainer (locale-scoped)
```

Do not parallelize with stage 3: seed merge changes `safety_decision`. Stage 1 may land on stage 0 as soon as catalog+trace exist.

Stop and revert if: any locale in `assist_langs` / `parity_langs` goes red, lock confirm drifts, the catalog merges locales, the trainer saves without validate, a stage lands de/en-only, or someone opens `staging` → `main` without an explicit go-ahead.

## Explicitly out

- A `staging` → `main` promote without an explicit go-ahead
- A stage or seed that only ships de/en
- New `PolicyId` from the UI or the model
- Replacing pack files at runtime
- Morphology, `NumberStyle`, tokenizer as overlay
- Merging every lexicon into one catalog
- LLM inside `nlu::parse`
- Assist tools for the trainer
