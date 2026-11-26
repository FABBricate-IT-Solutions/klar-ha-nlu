# Fix a phrase in three steps

[Deutsch](../fix-phrase.md) · [English](fix-phrase.md)

Generated Assist locales ship as **data files** under `packs/generated/{code}.yaml`
(inspect/diff) and as compiled Rust under `src/lang/packs/` (engine). Do **not**
hand-edit the generated Rust. A pack correction touches **one** file:
`packs/assist/{code}.yaml`.

## Three steps

1. **Reproduce** the wrong parse:

   ```bash
   cargo run --quiet -- lang preview --language fr --text "mode cinema"
   ```

2. **Correct** the phrase in the assist data file (create it with the CLI if needed):

   ```bash
   cargo run --quiet -- lang new-case \
     --language fr \
     --phrase "mode cinema" \
     --intent HassTurnOn \
     --slot entity_id=scene.cinema
   ```

   Or edit `packs/assist/fr.yaml` by hand — add a verb, a lexicon set, or an intent.
   That file is the only path that must change for the fix.

3. **Check** the preview and the suites that cover the locale:

   ```bash
   cargo run --quiet -- lang preview --language fr --text "mode cinema" --pack packs/assist/fr.yaml
   cargo nextest run --locked -E 'test(/assist_langs|language/)'
   ```

## What goes where

| Path | Role |
|------|------|
| `packs/assist/{code}.yaml` | Phrase corrections — **one file per locale**; never overwritten by the generator |
| `packs/generated/{code}.yaml` | Generated pack as data (read/diff); regenerated; do not hand-edit |
| `src/lang/packs/{mod}/` | Compiled engine tables from the generator — do not hand-edit |
| `scripts/lang_packs/` | Lexicon source for a **new** locale or a bulk refresh |

German and English stay hand-written under `src/lang/packs/de/` and `…/en/`.
