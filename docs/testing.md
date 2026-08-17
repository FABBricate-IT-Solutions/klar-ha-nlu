# Tests

[Deutsch](testing.md) · [English](en/testing.md)

```bash
cargo fmt --check
cargo check
cargo nextest run
cargo build --release
```

CI nutzt [cargo-nextest](https://nexte.st/) (`cargo nextest run --locked --profile ci`): jeder Test läuft im eigenen Prozess, Scheduling über alle Kerne. `cargo test` bleibt als Fallback. Lokal `cargo test -- --test-threads=1` wenn die Ausgabe hintereinander lesbar bleiben soll.

## Suiten

| Test | Datensatz | Schwelle |
|------|-----------|----------|
| `tests/german.rs` / `german_except.rs` / `german_tags.rs` | feste Einzelsätze | alle grün |
| `suite_deutsch` | `tests/datasets/wohnung_mittel` inkl. `assist/` | ≥ 95 %, Ziel 100 % |
| `suite_wohnung_live_assist` | `tests/datasets/wohnung_live/assist` gegen `wohnung_live.json` | alle grün |
| `suite_english_smoke` | `tests/datasets/wohnung_en` | ≥ 99 %, Ziel 100 % |
| `suite_deutsch_familienhaus` | `tests/datasets/familienhaus_de` | ≥ 99,5 %, Ziel 100 % |
| `suite_english_family_home` | `tests/datasets/family_home_en` | ≥ 99,5 %, Ziel 100 % |
| `suite_m0_exact_*` / `suite_m2_floors_*` | `m0_exact` / `m2_floors` | `fail == 0` |
| `tests/contract.rs` | V2-`ParseOutcome` | alle grün |
| `tests/policy.rs` | Confirm/OOD/Multi-Intent | alle grün |
| `tests/eval.rs` held-out | `m7_heldout` EN/DE | Intent-F1 ≥ 0,98, Slot-F1 ≥ 0,99, Pairing ≥ 0,97, ASR ≥ 0,92, Clarify-P ≥ 0,95 |
| `tests/privacy.rs` / `tests/migrate.rs` | Bundle-Redaktion, V1-Import | alle grün |

Nur die Voice-Suite:

```bash
cargo test --test german --test german_except --test german_tags --test voice_suite -- --test-threads=1 --nocapture
```

Fehlschläge landen unter `target/suite_fails_family_home_en.txt` und `target/suite_fails_familienhaus_de.txt`.

## Vertrag für Voice-Fälle

Bestehende generierte Fälle mit `conditions` bleiben unterstützt. Sie nutzen
weiter den bisherigen semantischen Matcher, damit Generatoren nicht auf einmal
umgestellt werden müssen. Jeder Fall muss genau einen nichtleeren Oracle-Modus
wählen: Legacy-`conditions` oder den exakten Vertrag. Beides zusammen ist
unzulässig. Neue Fälle für Dependency-Gates verwenden das NLU-spezifische Feld
`nlu_expect`:

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

Die Intent-Liste wird in der angegebenen Reihenfolge verglichen. Jeder
Intent-Name und seine vollständige Slot-Map müssen passen; zusätzliche,
fehlende oder falsch zugeordnete Slots schlagen fehl. `reject` und `clarify`
sind explizite Felder innerhalb von `nlu_expect`. Ablehnung bedeutet
`ParseDecision::Reject` — keine Intents, keine Rückfrage und kein Chat. Die eingecheckten `m0_exact`-Datensätze frieren
repräsentative DE/EN-Ergebnisse für Multi-Intent, Timer, Listen, Rückfragen,
Ablehnung und Zustands-Follow-ups ein.

`setup` beschreibt simulierten Home-Assistant-Zustand, nicht
NLU-Konversationshistorie. Der Harness bewahrt unterstützte Entity-Attribute,
Einkaufslisten- und Todo-Einträge in einer getrennten Testwelt auf, wendet
ausgegebene Intents darauf an und prüft optionale `world_expect`-Endzustände.
Die aktuelle Parser-API verarbeitet keinen HA-Zustand; deshalb wird Setup nie
in `Session` eingefügt. Nur vorherige Sätze des Falls erzeugen NLU-Kontext.

Die exakte Testwelt simuliert Entity-An/Aus/Toggle, Medien-Pause/Fortsetzen,
Stumm/Laut, absolute Zustands-Slots, relative Lautstärkerichtung,
Timer-Start/Fortsetzen/Pause/Abbruch/Daueränderungen sowie Hinzufügen/Erledigen
für Todo-Listen, dedizierte Einkaufslisten-Intents und
`name: shopping_list`. Timer-Fortsetzen ohne Dauer bewahrt die Setup-Dauer;
relative Lautstärke zeichnet `volume_step` ausschließlich als `up` oder `down`
auf; andere oder erfundene Zahlenwerte sind ungültig. Das Erledigen eines Listeneintrags setzt voraus, dass der Eintrag im
Setup oder einem vorherigen Turn existiert. Benannte Todo-Listen binden nur
über sichtbare, eindeutige HomeGraph-Namen, Aliase oder Labels; nicht
zugeordnete generische Einkaufslisten-Sätze bleiben `name: shopping_list`.
Query-Intents sind nur lesend. Das ist Assertion-Infrastruktur, kein
Home-Assistant-Emulator: Ein vom Modell nicht unterstützter Übergang lässt
jeden Fall mit `world_expect` fehlschlagen, statt still nichts zu tun.
Unbekannte, ziellose, verschachtelte, Oracle-lose oder gemischte
Legacy-/Exact-Setup-/Schema-Einträge scheitern beim Laden der Suite und können
nicht von Prozent-Schwellen verdeckt werden. Dedizierte englische und deutsche
`m0_exact`-Tests verlangen zusätzlich `stats.fail == 0`.

## Sätze erzeugen

```bash
python3 scripts/gen_voice_suite.py      # wohnung_mittel + wohnung_en
python3 scripts/voice_suite/gen_family_de.py  # familienhaus_de aus family_home_en
```

`scripts/gen_voice_suite.py` ruft die Wohnungsgeneratoren unter `scripts/voice_suite/wohnung/` auf. `de_assist.py` schreibt die per Assist (`conversation.process` auf `conversation.klar_nlu`) geprüften Sätze nach `wohnung_mittel/assist` und `wohnung_live/assist`. `scripts/voice_suite/gen_family_de.py` übersetzt `family_home_en` nach `familienhaus_de` inklusive `home_config.yaml`. Neu erzeugen überschreibt die jeweiligen Fixture-Dateien.

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

## Release-Gates

```bash
cargo test --test eval held_out
cargo test --test privacy --test migrate --test policy --test contract --test semantic
cargo run --quiet -- lang validate packs
cargo run --quiet -- eval bench --repeat 8
klar eval gate          # volle Scorecard, Exit 1 unter den Schwellen
klar migrate import --from /data/klar_nlu.json
```

CI-Job `test` führt `cargo nextest run --locked --profile ci` aus (`[profile.test]` mit `opt-level = 1`). `release-gates` bleibt als Pflicht-Check, die Pack-Validierung und der Bench stecken in `tests/language.rs` bzw. `tests/eval.rs`. Engine und `custom_components/klar_nlu` gehören in denselben Cut: die Integration spricht nur `POST /api/v2/parse`.
