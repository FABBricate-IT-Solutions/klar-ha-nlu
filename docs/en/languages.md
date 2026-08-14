# Languages

[Deutsch](../languages.md) · [English](languages.md)

Word lists live in `src/lang/`. The engine only knows verb classes and sets (`is_conj`, `has_light_noun`, …).

Current packs: **de** and **en**, both on by default (`Settings.languages`).

## Adding a pack

French is the example — the language itself is not shipped yet.

1. Copy `src/lang/en.rs` to `src/lang/fr.rs`.
2. Fill the lists: verbs, fillers, nouns, numbers, colors, patterns (`group_clarify`, `strip_pairs`, …).
3. In `src/lang/mod.rs`:
   - `mod fr;`
   - `LangId::Fr`
   - `from_code("fr")`, `code()`, `pack()`
4. Put `"fr"` in `Settings.languages` (API or default).

`LanguagePack` in `src/lang/pack.rs` is the checklist. Every field the engine reads must exist on the new pack — empty slices are fine, missing fields are not.

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

## Numbers

`NumberStyle`:

- `GermanUnd` — `einundzwanzig`, `eins und zwanzig`
- `EnglishTens` — `twenty one`

A language with its own grammar (`vingt-et-un`) needs a new variant and a branch in `src/numbers.rs`.

`ein` is deliberately not the number 1. It stays a power word.

## Tokens

`fold_latin` maps `ä` → `ae`, `é` → `e`, `ç` → `c`. Packs list the folded form (`oeffnen`, `kueche`).

## Tests

After a new pack:

- existing suites must stay at 100%
- a suite under `tests/datasets/` is the proof, not the word list alone

German family-home generator: `scripts/gen_familienhaus_de.py` (overwrites `tests/datasets/familienhaus_de/`).
