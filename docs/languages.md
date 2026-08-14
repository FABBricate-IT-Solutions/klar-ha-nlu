# Sprachen

[Deutsch](languages.md) · [English](en/languages.md)

Wortlisten leben in `src/lang/`. Die Engine kennt nur Verbklassen und Mengen (`is_conj`, `has_light_noun`, …).

Aktuell: **de** und **en**, standardmäßig beide aktiv (`Settings.languages`).

## Paket anlegen

Beispiel Französisch — die Sprache selbst ist noch nicht enthalten.

1. `src/lang/en.rs` nach `src/lang/fr.rs` kopieren.
2. Listen füllen: Verben, Füller, Nomen, Zahlen, Farben, Muster (`group_clarify`, `strip_pairs`, …).
3. In `src/lang/mod.rs`:
   - `mod fr;`
   - `LangId::Fr`
   - `from_code("fr")`, `code()`, `pack()`
4. `"fr"` in `Settings.languages` setzen (API oder Default).

`LanguagePack` in `src/lang/pack.rs` ist die Checkliste. Jedes Feld, das die Engine abfragt, muss im neuen Paket stehen — leere Slices sind erlaubt, fehlende Felder nicht.

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

## Zahlen

`NumberStyle`:

- `GermanUnd` — `einundzwanzig`, `eins und zwanzig`
- `EnglishTens` — `twenty one`

Eine Sprache mit eigener Grammatik (`vingt-et-un`) braucht eine neue Variante und einen Zweig in `src/numbers.rs`.

`ein` ist absichtlich keine Zahl. Es bleibt Schaltwort.

## Token

`fold_latin` macht aus `ä` → `ae`, `é` → `e`, `ç` → `c`. Pakete listen die gefaltete Form (`oeffnen`, `kueche`).

## Tests

Nach einem neuen Paket:

- bestehende Suiten müssen weiter 100 % bleiben
- eine eigene Suite unter `tests/datasets/` ist der Nachweis, nicht die Wortliste allein

Generator für die deutsche Familiensuite: `scripts/gen_familienhaus_de.py` (überschreibt `tests/datasets/familienhaus_de/`).
