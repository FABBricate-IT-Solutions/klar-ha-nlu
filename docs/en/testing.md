# Tests

[Deutsch](../testing.md) · [English](testing.md)

```bash
cargo fmt --check
cargo check
cargo test -- --test-threads=1
cargo build --release
```

`--test-threads=1` runs the large suites one after another — output stays readable and the thresholds apply cleanly.

## Suites

| Test | Dataset | Threshold |
|------|---------|-----------|
| `tests/german.rs` / `german_except.rs` / `german_tags.rs` | fixed single sentences | all green |
| `suite_deutsch` | `tests/datasets/wohnung_mittel` including `assist/` | ≥ 95%, target 100% |
| `suite_wohnung_live_assist` | `tests/datasets/wohnung_live/assist` against `wohnung_live.json` | all green |
| `suite_english_smoke` | `tests/datasets/wohnung_en` | target 100% |
| `suite_deutsch_familienhaus` | `tests/datasets/familienhaus_de` | ≥ 99.5%, target 100% |
| `suite_english_family_home` | `tests/datasets/family_home_en` | ≥ 99.5%, target 100% |

Voice suites only:

```bash
cargo test --test german --test german_except --test german_tags --test voice_suite -- --test-threads=1 --nocapture
```

Failures dump to `target/suite_fails_family_home_en.txt` and `target/suite_fails_familienhaus_de.txt`.

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
