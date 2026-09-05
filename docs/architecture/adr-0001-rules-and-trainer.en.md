# ADR 0001 — Visible rules, language seeds, and an LLM trainer

[Deutsch](adr-0001-rules-and-trainer.md) · [English](adr-0001-rules-and-trainer.en.md)

Status: **proposed** (no implementation in this change)

Klar stays a deterministic, local NLU. An LLM may **set up** the house; it must not **drive** the parse path.

## The idea, as we understand it

Behavior lives in two places that both say “policy” and do different jobs:

1. **Match** — compiled clause strategies in Rust (`PolicyId` in `src/parse/policy.rs` and `src/parse/clause.rs`): *how* a sentence becomes an intent (`area_command`, `grounded_entities`, `media`, …).
2. **Govern** — overlay rules (`PolicyRule` in `src/types/policy.rs`, **Rules** tab): *whether* a plan already recognized is executed, confirmed, blocked, preferred, or turned into a reply/script.

On top of that, **safety** (`requires_confirmation` for locks/covers, infra filters, confidence bands) and **household phrases** (clock, weather, explain, undo) are compiled plus language-pack strings.

Desired direction:

- An LLM acts as a **trainer**: it looks at the home graph (rooms, devices, tags, gaps) and proposes **govern rules** — the same `PolicyRule` objects the Rules tab already stores.
- Everything that today is **hard-coded** and changes behavior should be **visible**: same list, same evaluate, same why-trace. Otherwise “why confirm?” stays a secret behind `compiled_risky`.
- Behavior should be **house-precise**, especially so the trainer has something it is allowed to write without rewriting the engine.

Open strategy question: lift *everything* into the rules engine (plus pre-seeded rule databases per language) — or keep Match compiled and make only Govern / safety / household defaults data-driven.

## Current state (short)

```
Text
  → tokenize / bound language catalog
  → PolicyId matching (compiled) → candidates
  → ranking + evidence
  → overlay PolicyRule (govern) + compiled_risky
  → band: execute / confirm / clarify / reject / chat
```

Already there and reusable:

| Piece | Where | Gap |
|-------|--------|-----|
| Overlay `PolicyRule` (max 64) | `klar_nlu.json`, `GET/POST /api/v2/policies` | empty at start, no seed, no origin |
| Evaluate | `POST /api/v2/policies/evaluate` | overlay hit + `compiled_risky`; not *which* compiled match policy or *why* risky |
| `PolicyTrace` | `ParseOutcome` | overlay rule + flag only |
| `IntentCandidate.policy` | ranking | PolicyId string, not in the Rules UI |
| Language packs | `src/lang/packs/{code}/` | lexicon and household phrases, no govern seeds |
| Household route | `src/nlu/household.rs` | phrase→action in code |
| Safety | `src/nlu/validation.rs` `risky_intent` | invisible, not overridable via rules |
| Infra | tags + `infra_needles.txt` | only partly overlay (`infra_id` / tags) |
| Trainer | — | missing. The HA LLM talks or rewrites; it does not write rules |

Two different things are named policy. The trainer may write **govern** only. **Match** is an algorithm.

## Three strategies

### A — UI/trace only, engine unchanged

Catalog of PolicyIds plus a richer why-trace. Overlay rules stay hand-made.

- Cheap, no benchmark risk.
- The trainer has no standard rules to clone or override. Safety stays invisible.

### B — Everything in the rules engine (Match as data)

`area_command`, resolver, session replay, media-vs-lights as a DSL in pre-seeded per-language databases. The trainer writes arbitrary matching rules.

- One model, maximum flexibility.
- Today’s PolicyIds are **functions** (session, compounds, media claim, ranking caps), not `when`/`effect` rows. A DSL that can express them is a second language plus an interpreter. The 9,922-sentence DE/EN gate and locale parity would hang on generated matching. An LLM that invents match rules can quietly break Assist. That fights “no net in the engine.”

**Not the path** while Klar should stay local, deterministic, and benchmark-stable.

### C — Hybrid (recommendation)

Three layers with explicit write rights:

```
Match (compiled, read-only in the catalog)
  PolicyId + resolver + ranking + thresholds

Govern (data-driven, visible, overridable)
  language seed  →  house overlay (operator / trainer)
  confirm / block / allow / prefer / reply / script / template / llm

Invariants (compiled, rare, always in the trace)
  plan validation, expose filter, schema, optional safety floor
```

The LLM sees the house and writes **house overlay rules only** (plus optional aliases, tags, custom sentences). It does not change PolicyIds or pack word lists.

Pre-seeded “databases per language” then exist in **two** forms, and they stay separate:

1. **Lexicon pack** (already there): verbs, nouns, household phrases.
2. **Govern seed** (new): default rules for that language, as real `PolicyRule[]`.

## Why C, not B

- The trainer needs a **tight schema**. `PolicyRule` already has it (`when` + `effect` + `prefer`/`payload`). `sanitize_rules` and evaluate already exist.
- Visibility is not “the same interpreter.” Match policies can appear as catalog rows (`id`, label, precedence, “what it does”) without turning their Rust function into an editable rule.
- Safety that today lives in `risky_intent` *can* appear as a seed rule (`when.domain = lock` → `confirm`). Then you see it, you can override it, and evaluate shows `matched_rule` instead of only `compiled_risky`.
- Language seeds give the trainer templates: “this house has `lock.front_door` and `cover.living_shade` — instantiate the seed rules onto those entity ids.”

## Layer contract

### 1. Match catalog (engine, read-only)

Each `PolicyId` becomes a catalog row, for example:

```json
{
  "id": "area_command",
  "layer": "match",
  "origin": "engine",
  "editable": false,
  "precedence": 8,
  "summary": "Area + domain without a device name → area intent"
}
```

The parse trace always includes the chosen match policy (already `candidates[].policy`) plus losers and margin. The Rules UI shows these rows as **Engine**, not as an editable list.

Not in this catalog as editable rules: tokenizer, fuzzy, session memory, compound split. That stays code. The trace only says *that* they fired.

### 2. Govern seed per language (new)

Shipped with the lexicon pack, e.g. `src/lang/packs/de/govern.json` (or generated like packs). Contents are ordinary `PolicyRule`s with stable ids:

| Example id | when | effect | Role |
|------------|------|--------|------|
| `seed:confirm-lock` | domain `lock` | `confirm` | visible form of `risky_intent` for locks |
| `seed:confirm-cover-close` | domain `cover`, intent `HassTurnOff` | `confirm` | closing covers |
| `seed:block-area-lock` | domain `lock` + area set | `block` | no “every lock on the floor” |
| `seed:prefer-climate` | — | `prefer_entity` | only if `preferred_climate` is set; trainer fills `prefer` |

Household phrases that already look like rules (`reply` / `script`) can move here in a later phase. Clock/weather/undo stay code first, until the seed meets the same contract (tests in `tests/policy.rs` / household unit tests).

Seed rules **do not** count against the 64 house quota. A house rule with the same id **replaces** the seed. Extra house rules sit **in front** (first match wins, as today).

### 3. House overlay (operator + trainer)

Exactly today’s bundle in `klar_nlu.json`. New: `origin` (`operator` \| `trainer`) and `replaces` (seed id). Evaluate runs against the **merged** seed⊕house set.

### 4. Invariants

Even if someone disables `seed:confirm-lock`, `validate_plan`, Assist expose, and schema stay. Whether `compiled_risky` remains an invisible floor is an open question below — recommendation: **yes at first**. The trace distinguishes `hit: confirm` (rule) vs `compiled_risky: true` (floor). Once seeds cover the tests 1:1, the floor can hide behind a setting.

## LLM trainer

Not on the parse hot path. Not a tool that executes intents. Optional, operator-triggered.

```
Home graph (visible entities/areas/floors/tags)
  + govern seed of the bound language
  + current house rules
  + gaps (unnamed devices, missing areas)
  + optional redacted journal
      → trainer prompt with JSON schema = PolicyRule[]
      → sanitize_rules
      → grounding: entity_id / area / prefer exist on the graph
      → dry-run: evaluate on house smokes + locale smokes
      → UI diff: accept / reject / edit one-by-one
```

What the trainer **may** write: `PolicyRule`, optional aliases, `nlu_ignore`/`infra` tags, custom sentences.

What it **must not** write: PolicyIds, ranking thresholds, word lists, new effects outside the enum, entity ids that are not on the graph.

House examples the seed cannot know:

- `when.entity_id = climate.kids_room` → `block` (kids’ AC at night)
- `when.phrase = “good night”` → `script.good_night`
- `prefer_entity` for the living-room ceiling when several lights are named “lamp”
- `confirm` only for `lock.front_door`, not the shed lock

The model lives where the fallback agent already lives (HA). The engine stays without a net. The prompt gets a graph snapshot and the schema, not Assist tools.

## Visibility (“what happened and why”)

One why-trace per turn, one UI list with three origins:

```
matched_match:     area_command          (engine)
matched_govern:    seed:confirm-lock     (seed, visible)
overridden_by:     house:allow-shed-lock (house, trainer)
compiled_risky:    false
band:              execute
```

“What did you hear?” (`household.explain`) and `POST /api/lang/explain` should speak these ids, not only `decision: confirm`.

The evaluator on the Rules tab marks seed vs house hits and the ranking match policy.

## What deliberately does *not* move into the rules engine

- `PolicyId` functions and their precedence
- Confidence bands (`EXECUTE_MIN_CONFIDENCE` …) — at most later as documented settings, never as LLM output
- Resolver / fuzzy / compounds
- The language lexicon (that *is* already the per-language database)

Otherwise the LLM trains the NLU itself. That is exactly what Klar is not.

## Phases (technical, no calendar)

1. **Vocabulary + catalog + why-trace**  
   Match-catalog API; extend `PolicyTrace` with match id, seed id, risky reason. Rules UI: engine list read-only. No behavior change. Gate: contract tests for trace fields; DE/EN scorecard unchanged.

2. **Safety as seed, same behavior**  
   `risky_intent` and `allow_permitted` as seed rules for `de`/`en`. `compiled_risky` stays the floor until parity tests show seeds produce the same band. Quota: seed separate from the house limit.

3. **Govern seeds for every compiled locale**  
   Like lexicon packs: hand-written `de`/`en` reference, the rest generated or thin (safety universals are language-agnostic; phrase seeds are not). Generator freshness like `scripts/lang_packs`.

4. **Trainer endpoint**  
   `POST /api/v2/policies/propose` returns a proposal (no save). UI: diff, evaluate, apply. Prompt and schema versioned. Grounding tests on `tests/datasets/familienhaus_de` and `family_home_en`.

5. **Optional: household phrases → phrase rules**  
   Only if seed+overlay meet the same undo/explain/clock contract. Otherwise they stay code.

Each phase is its own PR. This ADR is the frame, not an implementation diff.

## Open questions (decide before phase 2/4)

1. May the operator really turn off `seed:confirm-lock`, or does `compiled_risky` always remain the floor?
2. Does the trainer run only in the Klar UI (v1 recommendation) or also as an Assist conversation (“set up my house”)?
3. Do household phrases become rules in phase 5, or stay lexicon+code?
4. After apply, does `origin: trainer` look like operator rules, or does provenance stay visible?
5. Infra needles: stay compiled, or does the trainer only tag graph entities (`infra` / `nlu_ignore`)?

Recommendations: (1) floor on at first, setting later. (2) UI only. (3) later. (4) keep provenance. (5) tags on the graph, needles as default suggestions.

## Consequences if we ship C

- The Rules tab becomes the source of truth: engine (visible, rigid), language (seed, overridable), house (editable, trainable).
- The trainer has a bounded write right and a dry-run against the same evaluate as a human.
- Match stays fast, tested, and local.
- The 64-rule cap applies to the house only; seeds are a second bundle.
- Phase-2 risk: seed safety must match `risky_intent` bit-for-bit, or lock confirms will drift.

## References

- Overlay rules: `src/types/policy.rs`, `src/nlu/policy_route.rs`, `src/io/policies.rs`
- Match policies: `src/parse/policy.rs`, `src/parse/clause.rs`
- Safety: `src/nlu/draft.rs` `safety_decision`, `src/nlu/validation.rs` `risky_intent`
- Ranking names: `IntentCandidate.policy`
- Language packs: [languages](../en/languages.md)
- API: [api](../en/api.md) (`/api/v2/policies`, `/api/v2/policies/evaluate`)
