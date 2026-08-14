# Tests

[Deutsch](../testing.md) · [English](testing.md)

```bash
cargo test -- --test-threads=1
```

`--test-threads=1` runs the large suites one after another — output stays readable and the thresholds apply cleanly.

## Suites

| Test | Dataset | Threshold |
|------|---------|-----------|
| `tests/german.rs` | fixed single sentences | 8/8 |
| `suite_deutsch` | `tests/datasets/wohnung_mittel` | ≥ 95%, target 100% |
| `suite_english_smoke` | `tests/datasets/wohnung_en` | 15/15 |
| `suite_deutsch_familienhaus` | `tests/datasets/familienhaus_de` | ≥ 99.5%, target 100% |
| `suite_english_family_home` | `tests/datasets/family_home_en` | ≥ 99.5%, target 100% |

Voice suites only:

```bash
cargo test --test german --test voice_suite -- --test-threads=1 --nocapture
```

Failures dump to `target/suite_fails_family_home_en.txt` and `target/suite_fails_familienhaus_de.txt`.

## Generating the German family suite

```bash
python3 scripts/gen_familienhaus_de.py
```

The script translates `family_home_en` into `familienhaus_de`, including `home_config.yaml`. Regenerating overwrites the folder.

## What the suites watch

- Device vs area (`lights in the living room` often stays area-level; a named lamp binds a device)
- Follow-ups (`turn it off`)
- Clarify when several lights share a room
- Cover vs lock vs garage door
- Timers and lists must not swallow lights
- `ein` is not the number 1

A change to `src/lang/de.rs` or `en.rs` without a suite run is incomplete. The lists are the tests.
