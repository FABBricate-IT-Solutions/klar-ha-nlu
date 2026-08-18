# Languages

[Deutsch](../languages.md) · [English](languages.md)

Every compiled Assist locale is first-class. German and English are hand-written reference packs; generated packs use the same `LanguagePack` path and the same enablement UX. `GET /api/v2/languages` lists the compiled set.

YAML under `packs/` is for user overlays and `klar lang import-hassil`, not Assist coverage. Packs are never silently merged into one giant default catalog.

Russian (`ru`, `ru-RU`) is not shipped: no pack, no registry row, `pin_language("ru")` stays unknown.

Import HassIL into an overlay (not into a merged default catalog):

```bash
klar lang import-hassil --from path/to/hassil --into /data --language de --dry-run
```

## Layout

- `src/lang/de_pack.rs` / `en_pack.rs` — hand-written reference packs
- `src/lang/de.rs` / `en.rs` — verb tables used by those packs
- `src/lang/packs/{code}/` — generated `verbs.rs`, `speech.rs`, `pack.rs`
- `src/lang/registry.rs` — compiled ids, `from_code`, `pack()`, `GET /api/v2/languages`
- `scripts/lang_packs/` — generator (HassIL harvest is bootstrap only). Do not run `generate.py` in pre-commit.

A generated pack may enter the binary when those fields are filled and the representative suite executes.

## Catalog model

`Catalog` merges the **pinned** packs for that request. Assist and `POST /api/v2/parse` should send `language`. Empty `Settings.languages` means every compiled locale is enabled for Assist, not “merge de+en.” Merging every lexicon into one catalog collides tokens (for example German `an` vs other packs) and is refused.

An explicit short list such as `["de", "en"]` still merges those packs for unpinned parse. That is a user choice, not a support tier.

`parse()` binds `Settings.languages`; helpers read `catalog()`. New engine fields belong on `LanguagePack` and in the existing `extend_sets!` merge.

## Adding a pack

1. Add a compact lexicon in `scripts/lang_packs/` (not a stub, not English filler tokens).
2. Run `python3 scripts/lang_packs/generate.py`.
3. Review the Rust like hand-written code: `rustfmt`, unique folded tokens, no comment narration.
4. Keep files under 500 lines. Do not add `match LangId` arms in `src/parse/`.
5. Existing suites must stay green; add the same assist/parity smoke for the new locale.

`LanguagePack` in `src/lang/groups.rs` is the checklist. Empty slices are allowed only when the language has no such concept.

## Verb classes

`VerbKind` is the role of a word, not the Home Assistant action. New classes need an explicit branch in `src/parse/action.rs` (no silent `_ =>`).

## Numbers

`NumberStyle`:

- `GermanUnd` — `einundzwanzig`
- `EnglishTens` — `twenty one`
- `ListedOnly` — listed words only (default for new packs)

A new combinator is a new variant plus tests. Do not extend `De | En` matches.

## Tokens

`fold_latin` maps `ä` → `ae`, `é` → `e`, `ç` → `c`, `ı/ş/ğ`, `ș/ț`. Packs store the folded form. CJK/Thai splitting is script-gated; Latin `tokenize` stays space-split.

## Home Assistant

The integration reads `custom_components/klar_nlu/languages.py` (generated). Options list every compiled locale with its native name. Default enablement is the full compiled set. Assist still pins one pack per request. `pt-BR` and `de-CH` are not stripped to ISO-639-1.

## Tests

- `tests/assist_langs.rs` — Execute smoke per compiled locale (including de/en)
- `tests/parity_langs.rs` — same Wohn+Family+m0+m2 rubric per compiled locale
- `tests/datasets/assist/{code}/representative.yaml` — representative gate
- `tests/language.rs` — pin, isolation, overlays, household cues
- DE/EN voice suites (`wohnung_mittel`, `wohnung_en`, `familienhaus_de`, `family_home_en`) are the **oracle** graphs; other locales overlay native sentences on those same graphs

## Dataset generation (every locale, local)

One command writes parity overlays for every generated locale (not Russian):

```bash
python3 scripts/parity/generate.py
```

It reads the DE oracles (`wohnung_mittel`, `familienhaus_de`, `m0_exact`, `m2_floors`) and the locale lexicon, then writes `tests/datasets/parity/{code}/{suite}/`. Room aliases go to `tests/datasets/parity/rooms.yaml`.

DE and EN are not overlays: they **are** the oracles. Regenerate those with `python3 scripts/gen_voice_suite.py` (and the family-home scripts in `docs/en/testing.md`). Then re-run `scripts/parity/generate.py` so every other locale stays in lockstep.

CI checks that this generator is a no-op (freshness). It does not run the full 65-locale matrix. If a PR changes a pack or dataset path, CI runs that locale's suite (`scripts/ci_lang_tests.py`): de/en hard-gate, others report-only. Locally:

```bash
python3 scripts/lang_packs/generate.py   # packs + assist smokes
python3 scripts/parity/generate.py       # per-locale datasets
python3 scripts/check_lang_packs.py
cargo nextest run --test assist_langs --test language --test parity_langs --test voice_suite
```
