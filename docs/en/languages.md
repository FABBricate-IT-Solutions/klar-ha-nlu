# Languages

[Deutsch](../languages.md) · [English](languages.md)

Word lists live in `src/lang/`. The engine only knows verb classes and sets (`is_conj`, `has_light_noun`, …).

Current packs: **de** and **en**, both on by default (`Settings.languages`).

## Adding a pack

French is the example — the language itself is not shipped yet.

1. Copy `src/lang/en.rs` to `src/lang/fr.rs` and fill the verb table.
2. Copy `src/lang/en_pack.rs` to `src/lang/fr_pack.rs` and fill fillers, nouns, numbers, colors, patterns (`group_clarify`, `strip_pairs`, …), and `Speech`.
3. In `src/lang/mod.rs`:
   - `mod fr;`
   - `mod fr_pack;`
   - `LangId::Fr`
   - `from_code("fr")`, `code()`, `pack()`
4. Put `"fr"` in `Settings.languages` (API or default).

`LanguagePack` in `src/lang/groups.rs` is the checklist. Every field the engine reads must exist on the new pack — empty slices are fine, missing fields are not. `src/lang/pack.rs` is only a re-export shim.

`Catalog` merges the active packs for each request. `parse()` binds the catalog from `Settings.languages`; helpers then read it through `catalog()`. New pack fields therefore need to appear in both the schema and the catalog merge.

## Verb classes

`VerbKind` in `src/lang/verbs.rs` is the role of a word, not the HA action.

| Class | Rough meaning |
|-------|----------------|
| `On` / `OnParticle` | turn on; `on` has extra logic before conjunctions |
| `Open` / `OpenDoor` | open; German *öffnen* prefers door/lock |
| `Close`, `Lock`, `Unlock` | close, lock, unlock |
| `Query` | status question |
| `Timer`, `List`, `Color` | their own domains |

The same class in several languages is intentional. Collisions (same token, different classes) are last-wins in the catalog — do not merge packs blindly if a word is a filler in language A and a verb in B.

The mapping from `VerbKind` to NLU actions lives in `src/parse/action.rs`. New verb classes need an explicit branch there.

## Numbers

`NumberStyle`:

- `GermanUnd` — `einundzwanzig`, `eins und zwanzig`
- `EnglishTens` — `twenty one`

A language with its own grammar (`vingt-et-un`) needs a new variant and a branch in `src/parse/numbers.rs`.

`ein` is deliberately not the number 1. It stays a power word.

## Tokens

`fold_latin` maps `ä` → `ae`, `é` → `e`, `ç` → `c`. Packs list the folded form (`oeffnen`, `kueche`).

## Tests

After a new pack:

- existing suites must stay at 100%
- a suite under `tests/datasets/` is the proof, not the word list alone
- run variants in German and English against apartment and family-home data when shared parse rules are affected

Generators:

```bash
python3 scripts/gen_voice_suite.py
python3 scripts/voice_suite/gen_family_de.py
```

`scripts/gen_voice_suite.py` builds the apartment suites. `scripts/voice_suite/gen_family_de.py` creates the German family-home suite from `family_home_en`.
