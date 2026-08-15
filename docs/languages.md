# Sprachen

[Deutsch](languages.md) · [English](en/languages.md)

Wortlisten leben in `src/lang/`. Die Engine kennt nur Verbklassen und Mengen (`is_conj`, `has_light_noun`, …).

Aktuell: **de** und **en**, standardmäßig beide aktiv (`Settings.languages`).

## Paket anlegen

Beispiel Französisch — die Sprache selbst ist noch nicht enthalten.

1. `src/lang/en.rs` nach `src/lang/fr.rs` kopieren und die Verb-Tabelle füllen.
2. `src/lang/en_pack.rs` nach `src/lang/fr_pack.rs` kopieren und Füller, Nomen, Zahlen, Farben, Muster (`group_clarify`, `strip_pairs`, …) sowie `Speech` füllen.
3. In `src/lang/mod.rs`:
   - `mod fr;`
   - `mod fr_pack;`
   - `LangId::Fr`
   - `from_code("fr")`, `code()`, `pack()`
4. `"fr"` in `Settings.languages` setzen (API oder Default).

`LanguagePack` in `src/lang/groups.rs` ist die Checkliste. Jedes Feld, das die Engine abfragt, muss im neuen Paket stehen — leere Slices sind erlaubt, fehlende Felder nicht. `src/lang/pack.rs` ist nur ein Reexport-Shim.

`Catalog` merge't die aktiven Packs pro Request. `parse()` bindet den Catalog aus `Settings.languages`; Hilfsfunktionen lesen ihn danach über `catalog()`. Deshalb müssen neue Pack-Felder sowohl im Schema als auch im Catalog-Merge auftauchen.

## Verbklassen

`VerbKind` in `src/lang/verbs.rs` beschreibt die Rolle eines Wortes, nicht die HA-Aktion.

| Klasse | grobe Bedeutung |
|--------|------------------|
| `On` / `OnParticle` | einschalten; `on` hat Extra-Logik vor Konjunktionen |
| `Open` / `OpenDoor` | öffnen; Deutsch *öffnen* bevorzugt Tür/Schloss |
| `Close`, `Lock`, `Unlock` | zu, abschließen, aufschließen |
| `Query` | Statusfrage |
| `Timer`, `List`, `Color` | eigene Domänen |

Gleiche Klasse in mehreren Sprachen ist Absicht. Kollisionen (dasselbe Token, verschiedene Klassen) löst der Catalog last-wins — deshalb Pakete nicht blind mergen, wenn ein Wort in Sprache A Füller und in B Verb ist.

Die Abbildung von `VerbKind` auf NLU-Aktionen liegt in `src/parse/action.rs`. Neue Verbklassen brauchen dort eine bewusste Behandlung.

## Zahlen

`NumberStyle`:

- `GermanUnd` — `einundzwanzig`, `eins und zwanzig`
- `EnglishTens` — `twenty one`

Eine Sprache mit eigener Grammatik (`vingt-et-un`) braucht eine neue Variante und einen Zweig in `src/parse/numbers.rs`.

`ein` ist absichtlich keine Zahl. Es bleibt Schaltwort.

## Token

`fold_latin` macht aus `ä` → `ae`, `é` → `e`, `ç` → `c`. Pakete listen die gefaltete Form (`oeffnen`, `kueche`).

## Tests

Nach einem neuen Paket:

- bestehende Suiten müssen weiter 100 % bleiben
- eine eigene Suite unter `tests/datasets/` ist der Nachweis, nicht die Wortliste allein
- Varianten in Deutsch und Englisch gegen Wohnung und Familienhaus laufen lassen, wenn gemeinsame Parse-Regeln betroffen sind

Generatoren:

```bash
python3 scripts/gen_voice_suite.py
python3 scripts/voice_suite/gen_family_de.py
```

`scripts/gen_voice_suite.py` baut die Wohnungssuiten. `scripts/voice_suite/gen_family_de.py` erzeugt die deutsche Familiensuite aus `family_home_en`.
