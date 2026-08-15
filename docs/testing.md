# Tests

[Deutsch](testing.md) · [English](en/testing.md)

```bash
cargo test -- --test-threads=1
```

`--test-threads=1` hält die großen Suiten nacheinander — die Ausgabe bleibt lesbar, die Schwellen greifen zuverlässig.

## Suiten

| Test | Datensatz | Schwelle |
|------|-----------|----------|
| `tests/german.rs` | feste Einzelsätze | 8/8 |
| `suite_deutsch` | `tests/datasets/wohnung_mittel` | ≥ 95 %, Ziel 100 % |
| `suite_english_smoke` | `tests/datasets/wohnung_en` | 15/15 |
| `suite_deutsch_familienhaus` | `tests/datasets/familienhaus_de` | ≥ 99,5 %, Ziel 100 % |
| `suite_english_family_home` | `tests/datasets/family_home_en` | ≥ 99,5 %, Ziel 100 % |

Nur die Voice-Suite:

```bash
cargo test --test german --test voice_suite -- --test-threads=1 --nocapture
```

Fehlschläge landen unter `target/suite_fails_family_home_en.txt` und `target/suite_fails_familienhaus_de.txt`.

## Deutsche Familiensuite erzeugen

```bash
python3 scripts/gen_familienhaus_de.py
```

Das Skript übersetzt `family_home_en` nach `familienhaus_de` inklusive `home_config.yaml`. Neu erzeugen überschreibt den Ordner.

## Worauf die Suiten achten

- Gerät vs. Area (`Licht im Wohnzimmer` bleibt oft Area, `Wohnzimmerlampe` bindet ein Gerät)
- Follow-ups (`mach sie aus`)
- Clarify bei mehreren Leuchten
- Cover vs. Schloss vs. Garagentor
- Timer und Listen dürfen Licht nicht schlucken
- `ein` ist kein Zahlenwert 1

Eine Änderung an `src/lang/de.rs` oder `en.rs` ohne Suite-Lauf ist unvollständig. Die Listen sind die Tests.

## Home-Assistant-Helfer

Stdlib-Tests, ohne installiertes Home Assistant:

```bash
python3 tests/test_refine.py
python3 tests/test_speech.py
python3 tests/test_fallback.py
```

`test_refine.py` hält die Prompts pro Persönlichkeit, die Zahlen-/Faktenwachen und dass Sprechformel und Refine-Stimme zusammenpassen.
