# Tests

[Deutsch](testing.md) · [English](en/testing.md)

```bash
cargo fmt --check
cargo check
cargo test -- --test-threads=1
cargo build --release
```

`--test-threads=1` hält die großen Suiten nacheinander — die Ausgabe bleibt lesbar, die Schwellen greifen zuverlässig.

## Suiten

| Test | Datensatz | Schwelle |
|------|-----------|----------|
| `tests/german.rs` / `german_except.rs` / `german_tags.rs` | feste Einzelsätze | alle grün |
| `suite_deutsch` | `tests/datasets/wohnung_mittel` | ≥ 95 %, Ziel 100 % |
| `suite_english_smoke` | `tests/datasets/wohnung_en` | Ziel 100 % |
| `suite_deutsch_familienhaus` | `tests/datasets/familienhaus_de` | ≥ 99,5 %, Ziel 100 % |
| `suite_english_family_home` | `tests/datasets/family_home_en` | ≥ 99,5 %, Ziel 100 % |

Nur die Voice-Suite:

```bash
cargo test --test german --test german_except --test german_tags --test voice_suite -- --test-threads=1 --nocapture
```

Fehlschläge landen unter `target/suite_fails_family_home_en.txt` und `target/suite_fails_familienhaus_de.txt`.

## Sätze erzeugen

```bash
python3 scripts/gen_voice_suite.py      # wohnung_mittel + wohnung_en
python3 scripts/voice_suite/gen_family_de.py  # familienhaus_de aus family_home_en
```

`scripts/gen_voice_suite.py` ruft die Wohnungsgeneratoren unter `scripts/voice_suite/wohnung/` auf. `scripts/voice_suite/gen_family_de.py` übersetzt `family_home_en` nach `familienhaus_de` inklusive `home_config.yaml`. Neu erzeugen überschreibt die jeweiligen Fixture-Dateien.

## Worauf die Suiten achten

- Gerät vs. Area (`Licht im Wohnzimmer` bleibt oft Area, `Wohnzimmerlampe` bindet ein Gerät)
- Follow-ups (`mach sie aus`)
- Clarify bei mehreren Leuchten
- Cover vs. Schloss vs. Garagentor
- Timer und Listen dürfen Licht nicht schlucken
- `ein` ist kein Zahlenwert 1
- Natürliche Mundart (außer, lichte, Schlafzimmern, Wohn und Esszimmer) — neue Variante nach `scripts/gen_voice_suite.py`, nicht nur als Rust-Assert
- Jede neue Variante in **Deutsch und Englisch** (Wohnung + Familienhaus)

Eine Änderung an `src/lang/de.rs`, `src/lang/en.rs`, `src/lang/de_pack.rs` oder `src/lang/en_pack.rs` ohne Suite-Lauf in beiden Sprachen ist unvollständig. Die Listen sind die Tests.

Bei Änderungen an `src/parse/action.rs`, `src/parse/resolve/`, `src/parse/slots.rs` oder `src/home/roles.rs` immer Wohnung und Familienhaus laufen lassen. Diese Module beeinflussen mehrere Domänen und Sprachen gleichzeitig.

## Dokumentations-Drift

Nach Strukturänderungen alte Pfade suchen:

```bash
rg 'src/(parse\.rs|web\.rs|wyoming\.rs|lexicon\.rs|numbers\.rs)|parse_help|home_policy' docs README.md README.de.md
```

Wenn eine Generatorstruktur geändert wird, zuerst `docs/testing.md` und `docs/en/testing.md` anpassen.

## Home-Assistant-Helfer

Stdlib-Tests, ohne installiertes Home Assistant:

```bash
python3 tests/ha/test_refine.py
python3 tests/ha/test_speech.py
python3 tests/ha/test_fallback.py
```

`test_refine.py` hält die Prompts pro Persönlichkeit, die Zahlen-/Faktenwachen und dass Sprechformel und Refine-Stimme zusammenpassen.
