# Tests

[Deutsch](../testing.md) · [English](testing.md)

```bash
cargo fmt --check
cargo check
cargo nextest run
cargo build --release
```

CI uses [cargo-nextest](https://nexte.st/) (`cargo nextest run --locked --profile ci`): each test runs in its own process and is scheduled across all cores. `cargo test` remains a fallback. Locally, `cargo test -- --test-threads=1` keeps output sequential and easier to read.

## Suites

| Test | Dataset | Threshold |
|------|---------|-----------|
| `tests/german.rs` / `german_except.rs` / `german_tags.rs` | fixed single sentences | all green |
| `suite_deutsch` | `tests/datasets/wohnung_mittel` including `assist/` | ≥ 95%, target 100% |
| `suite_wohnung_live_assist` | `tests/datasets/wohnung_live/assist` against `wohnung_live.json` | all green |
| `suite_english_smoke` | `tests/datasets/wohnung_en` | ≥ 99%, target 100% |
| `suite_deutsch_familienhaus` | `tests/datasets/familienhaus_de` | ≥ 99.5%, target 100% |
| `suite_english_family_home` | `tests/datasets/family_home_en` | ≥ 99.5%, target 100% |
| `suite_m0_exact_*` / `suite_m2_floors_*` | `m0_exact` / `m2_floors` | `fail == 0` |
| `tests/contract.rs` | V2 `ParseOutcome` | all green |
| `tests/policy.rs` | Confirm/OOD/multi-intent | all green |
| `tests/eval.rs` held-out | `m7_heldout` EN/DE | Intent-F1 ≥ 0.98, Slot-F1 ≥ 0.99, pairing ≥ 0.97, ASR ≥ 0.92, Clarify P ≥ 0.95 |
| `tests/privacy.rs` / `tests/migrate.rs` | bundle redaction, V1 import | all green |

Voice suites only:

```bash
cargo test --test german --test german_except --test german_tags --test voice_suite -- --test-threads=1 --nocapture
```

Failures dump to `target/suite_fails_family_home_en.txt` and `target/suite_fails_familienhaus_de.txt`.

## Voice case contract

Existing generated cases using `conditions` remain supported. They use the
legacy semantic matcher so generators do not need an all-at-once migration.
Every case must select exactly one non-empty oracle mode: legacy `conditions`
or the exact contract. They cannot be combined. New dependency-gate cases use
the NLU-specific `nlu_expect` field:

```yaml
- name: kitchen_then_dining
  setup:
    - {entity_id: light.kitchen, state: "off"}
  world_expect:
    - {entity_id: light.kitchen, state: "on"}
  sentences:
    - Turn on the kitchen and dining lights
  nlu_expect:
    intents:
      - intent: HassTurnOn
        slots: {area: kitchen, domain: light}
      - intent: HassTurnOn
        slots: {area: dining, domain: light}
    reject: false
    clarify: false
```

The intent list is compared in declared order. Every intent name and its
complete slot map must match; extra, missing, or cross-paired slots fail.
`reject` and `clarify` are explicit fields inside `nlu_expect`. Rejection means
`ParseDecision::Reject` — no intents, no clarification, and not Chat. The checked-in `m0_exact` datasets freeze
representative DE/EN results for multi-intent, timer, list, clarification,
rejection, and state-persistence behavior.

`setup` describes simulated Home Assistant state, not NLU conversation
history. The harness preserves supported entity attributes, shopping-list
items, and todo-list items in a separate test world, applies emitted intents to
that world, and checks optional `world_expect` post-state records. The current
parser API does not consume HA state, so setup is never inserted into
`Session`; only preceding sentences in the case create NLU context.

The exact test world simulates entity on/off/toggle, media pause/resume,
mute/unmute, absolute state slots, relative-volume direction, timer
start/resume/pause/cancel/duration changes, and add/complete operations for
todo lists, dedicated shopping-list intents, and `name: shopping_list`.
Duration-less timer resume preserves the setup duration; relative volume
records `volume_step` only as `up` or `down`; other or invented numeric values
are invalid. Completing a list
item requires that item to exist in setup or an earlier turn. Named todo lists
bind only through visible, unambiguous HomeGraph names, aliases, or labels;
unmatched generic shopping wording remains `name: shopping_list`. Query
intents are read-only. This is assertion infrastructure, not a Home Assistant emulator:
an emitted transition unsupported by the model fails any case with
`world_expect` instead of silently doing nothing. Unknown, targetless, nested,
missing-oracle, or mixed legacy/exact setup/schema records fail while loading
the suite and cannot be hidden by percentage thresholds. Dedicated English
and German `m0_exact` tests additionally require `stats.fail == 0`.

## Generating suites

```bash
python3 scripts/gen_voice_suite.py      # wohnung_mittel + wohnung_en
python3 scripts/voice_suite/gen_family_de.py  # familienhaus_de from family_home_en
```

`scripts/gen_voice_suite.py` calls the apartment generators under `scripts/voice_suite/wohnung/`. `de_assist.py` writes sentences checked via Assist (`conversation.process` on `conversation.klar_nlu`) to `wohnung_mittel/assist` and `wohnung_live/assist`. `scripts/voice_suite/gen_family_de.py` translates `family_home_en` into `familienhaus_de`, including `home_config.yaml`. Regenerating overwrites the affected fixture files.

## What the suites watch

- Device vs area (`lights in the living room` often stays area-level; a named lamp binds a device)
- Follow-ups (`turn it off`)
- Clarify when several lights share a room
- Cover vs lock vs garage door
- Timers and lists must not swallow lights
- `ein` is not the number 1
- Natural phrasing (except, bedrooms, living and dining) — add variants in `scripts/gen_voice_suite.py`, not only as a Rust assert
- Every new variant in **German and English** (apartment + family home)

A change to `src/lang/de.rs`, `src/lang/en.rs`, `src/lang/de_pack.rs`, or `src/lang/en_pack.rs` without a suite run in both languages is incomplete. The lists are the tests.

For changes to `src/parse/action.rs`, `src/parse/resolve/`, `src/parse/slots.rs`, or `src/home/roles.rs`, run apartment and family-home suites. Those modules affect several domains and languages at once.

## Documentation Drift

After structure changes, search for stale paths:

```bash
rg 'src/(parse\.rs|web\.rs|wyoming\.rs|lexicon\.rs|numbers\.rs)|parse_help|home_policy' docs README.md README.de.md
```

When generator structure changes, update `docs/testing.md` and `docs/en/testing.md` first.

## Home Assistant helpers

Stdlib tests, no Home Assistant install:

```bash
python3 tests/ha/test_refine.py
python3 tests/ha/test_speech.py
python3 tests/ha/test_fallback.py
```

`test_refine.py` locks the per-personality prompts, digit/fact guards, and that the spoken cue stays aligned with the refine voice.

## Release gates

```bash
cargo test --test eval held_out
cargo test --test privacy --test migrate --test policy --test contract --test semantic
cargo run --quiet -- lang validate packs
cargo run --quiet -- eval bench --repeat 8
klar eval gate          # full scorecard, exit 1 below thresholds
klar migrate import --from /data/klar_nlu.json
```

CI job `test` runs `cargo nextest run --locked --profile ci` (`[profile.test]` with `opt-level = 1`). `release-gates` stays as a required check; pack validation and the bench live in `tests/language.rs` and `tests/eval.rs`. Ship the engine and `custom_components/klar_nlu` in the same cut: the integration only calls `POST /api/v2/parse`.
