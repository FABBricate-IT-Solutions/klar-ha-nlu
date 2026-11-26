# Languages

[Deutsch](../languages.md) · [English](languages.md)

Every compiled Assist locale uses the same `LanguagePack` path. German and English are hand-written reference packs; the other packs are generated. A locale is **reference**, **beta**, or **experimental** (see [Tiers](#tiers)); `GET /api/v2/languages` lists the locales that can be picked, each with its tier.

## Supported locales

**67** compiled Assist locales (source of truth: `custom_components/klar_nlu/languages.py` / `GET /api/v2/languages`).

| Code | Name | Code | Name |
|------|------|------|------|
| `de` | Deutsch | `en` | English |
| `fr` | Français | `nl` | Nederlands |
| `es` | Español | `it` | Italiano |
| `pt` | Português | `pt-BR` | Português (Brasil) |
| `ca` | Català | `ro` | Română |
| `da` | Dansk | `nb` | Norsk Bokmål |
| `sv` | Svenska | `fi` | Suomi |
| `de-CH` | Schwyzerdütsch | `de-AT` | Deutsch (Österreich) |
| `en-GB` | English (UK) | `af` | Afrikaans |
| `cs` | Čeština | `sk` | Slovenčina |
| `pl` | Polski | `hu` | Magyar |
| `hr` | Hrvatski | `sl` | Slovenščina |
| `bg` | Български | `el` | Ελληνικά |
| `sr` | Српски | `sr-Latn` | Srpski |
| `uk` | Українська | `tr` | Türkçe |
| `zh-CN` | 简体中文 | `zh-TW` | 繁體中文（台灣） |
| `zh-HK` | 繁體中文（香港） | `ja` | 日本語 |
| `ko` | 한국어 | `th` | ภาษาไทย |
| `ar` | العربية | `he` | עברית |
| `fa` | فارسی | `ur` | اُردُو |
| `cy` | Cymraeg | `et` | Eesti |
| `eu` | Euskara | `ga` | Gaeilge |
| `gl` | Galician | `is` | islenska |
| `lb` | Letzebuergesch | `kw` | kernewek |
| `lt` | Lietuviu | `lv` | Latviesu |
| `id` | Bahasa Indonesia | `ms` | Bahasa Melayu |
| `sw` | Kiswahili | `vi` | Tieng Viet |
| `hi` | Hindi | `bn` | Bangla |
| `gu` | Gujarati | `kn` | Kannada |
| `ml` | Malayalam | `mr` | Marathi |
| `ta` | Tamil | `te` | Telugu |
| `pa` | Punjabi | `ne` | Nepali |
| `hy` | Armenian | `ka` | Georgian |
| `mn` | Mongolian | | |

`de` / `en` are hand-written reference packs. The other locales are generated; each one passes the same minimal-pair gate (`tests/language_tiers.rs`) and is listed as reference. Coverage of everyday sentences still differs per language.

**Not shipped:** Russian (`ru`, `ru-RU`) — no pack, no registry row, `pin_language("ru")` stays unknown.

YAML under `packs/` holds user overlays (`klar lang import-hassil`), **assist phrase corrections** (`packs/assist/{code}.yaml`, one file per locale — [fix a phrase in three steps](fix-phrase.md)), and the generated pack data mirror (`packs/generated/{code}.yaml`). Packs are never silently merged into one giant default catalog.

Import HassIL into an overlay (not into a merged default catalog):

```bash
klar lang import-hassil --from path/to/hassil --into /data --language de --dry-run
```

## Layout

- `packs/generated/{code}.yaml` — generated pack as data (read/diff; do not hand-edit)
- `packs/assist/{code}.yaml` — phrase corrections (one file; [fix-phrase](fix-phrase.md); `klar lang new-case`)
- `src/lang/packs/{code}/` — compiled tables (de/en hand-written; others generated — do not hand-edit)
- `src/lang/registry.rs` — compiled ids, `from_code`, `pack()`, `GET /api/v2/languages`
- `scripts/lang_packs/` — generator (HassIL harvest is bootstrap only). Do not run `generate.py` in pre-commit.

A generated pack may enter the binary when those fields are filled and the representative suite executes.

## Tiers

A tier says how far a locale is tested. It does not remove a pack. Published table (`scripts/lang_packs/tiers.py`, `GET /api/v2/languages`):

| Tier | Codes | Gate |
|------|-------|------|
| **reference** | every compiled locale (all 67) | Full on/off/negation minimal pairs: every basic command acts the right way round; a question or a negated command switches nothing |
| **beta** | none today | No opposite action + gold natural sentences (`tests/datasets/gold/`) |
| **experimental** | none today; a new locale starts here | Behind `experimental_languages` |

`tests/language_tiers.rs` and `tests/language_gold.rs` decide. Every shipped locale also ships hand-written natural sentences (`tests/datasets/gold/{code}.yaml`, at least ten per locale, each with its own small home named in that language). No sentence may take a wrong action family (a list add never switches a device, a power command never edits a list, a chat or confirm sentence never executes); every sentence not marked `soft:` must also match exactly. `de-AT`, `de-CH` and `en-GB` inherit the `de` and `en` files; every other locale, `pt-BR`, `zh-TW` and `zh-HK` included, has its own. Sentences marked `soft:` are known gaps kept as probes (see `tests/datasets/gold/README.md`). Every shipped locale lists its own negation words (the generated packs in `scripts/lang_packs/mood.py`), so a negated command is refused in every language. Every shipped locale is always offered; the switch below only matters for a locale that is added later and has not earned reference yet. Advertised tags (`zh`, `zh-Hans`, `no`, `nn`, …) resolve through the variants table and can be pinned. In a bilingual house the reply language follows the detected utterance language, not the alphabetically first pack.

The engine setting `experimental_languages` (off by default) is the switch:

- A locale that is configured already (`languages`, `default_language`) keeps working and stays listed, whatever its tier. An update never changes the language of a house.
- A request that pins a language (`language` on `POST /api/v2/parse`, the Assist pipeline language) is answered in that language, whatever its tier.
- With the switch off, an experimental locale (none ships today) is not listed for a new selection and a settings save that adds one is refused (`422`). Reference and Beta are always offered.

## Catalog model

`Catalog` merges the **pinned** packs for that request. Assist and `POST /api/v2/parse` should send `language`. Empty `Settings.languages` means every compiled locale may be pinned, not “merge de+en.” A request that names no language then binds the default pack (`Settings.default_language`, English when unset), never an empty catalog. Binding every compiled locale at once would collide tokens (for example German `an` vs other packs); the engine returns an empty catalog for that case instead of a giant merge. Adding an experimental locale (none ships today) while the switch is off is refused with `422`.

An explicit short list such as `["de", "en"]` still merges those packs for unpinned parse. That is a user choice, not a support tier.

### Regional packs

`de-AT`, `de-CH`, `en-GB` and `pt-BR` are regional packs. Their registry row names a `parent` (`de`, `en`, `pt`), and the catalog binds the regional pack on top of that parent: the parent's words are read first, the regional pack adds its own and wins a shared key. Replies and household texts stay regional. `GET /api/v2/languages` lists the `parent` of each locale.

A regional pack holds only what the region says differently. In `scripts/lang_packs` that is `like` (a regional word and the standard word it stands for: `"aa": "an"`; the generator writes it into every parent list that holds the standard word, with the same verb class) and `add` (words with no single standard counterpart, by pack path). The generator refuses a regional word the parent already knows.

A regional pack has no sentences of its own. `tests/regional_overlays.rs`, the parity matrix and the full-home tiers run the parent's sentences under the regional code and expect the same decisions.

`parse()` binds `Settings.languages`; helpers read `catalog()`. New engine fields belong on `LanguagePack` and in the existing `extend_sets!` merge.

## Layout

- `packs/generated/{code}.yaml` — generated pack as data (read/diff; do not hand-edit)
- `packs/assist/{code}.yaml` — phrase corrections (one file; [fix-phrase](fix-phrase.md); `klar lang new-case`)
- `src/lang/packs/{code}/` — compiled tables (de/en hand-written; others generated — do not hand-edit)
- `src/lang/registry.rs` — compiled ids, `from_code`, `pack()`, `GET /api/v2/languages`
- `scripts/lang_packs/` — generator (HassIL harvest is bootstrap only). Do not run `generate.py` in pre-commit.

## Adding a pack

1. Add a compact lexicon in `scripts/lang_packs/`. Prefer target-language words; `scripts/lang_packs/pack_lint.py` fails when chat/speech fields still carry English filler stubs.
2. Run `python3 scripts/lang_packs/generate.py` (writes data under `packs/generated/` and compiled Rust).
3. Review the generated data YAML. Do not add `match LangId` arms in `src/parse/`.
4. Existing suites must stay green; add the same assist/parity smoke for the new locale.
5. Later phrase fixes go only in `packs/assist/{code}.yaml`.

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

`fold_latin` (NFKC, lowercase, then per-script fold) maps `ä` → `ae`, `é` → `e`, `ç` → `c`, `ı/ş/ğ`, `ș/ț`. Pack tables keep Latin digraphs that already match that fold (for example `oeffnen`); speech and labels still run through `fold_latin` at parse time. CJK/Thai splitting is script-gated; Latin `tokenize` stays space-split.

## Home Assistant

The integration reads `custom_components/klar_nlu/languages.py` (generated). Assist languages are set in the operator UI (Settings): empty = every compiled locale. Default follows the request/pipeline language. A single pinned pack still restricts parsing to that pack. `pt-BR` and `de-CH` are not stripped to ISO-639-1.

Operator chrome (the official Klar app) is **not** the Assist pin and **not** the Home Assistant profile. Set it in App → Settings → Operator language, or with `KLAR_UI_LOCALE` before the first save. Saved Settings wins. Integration form labels stay on Home Assistant `translations/{lang}.json`.

## Tests

- `tests/assist_langs.rs` — Execute smoke per compiled locale (including de/en)
- `tests/parity_langs.rs` — same Wohn+Family+m0+m2 rubric per compiled locale
- `tests/datasets/assist/{code}/representative.yaml` — representative gate
- `tests/language.rs` — pin, isolation, overlays, household cues
- `tests/regional_overlays.rs` — a regional pack decides like its parent on the parent's suites
- `tests/language_tiers.rs` — minimal pairs per locale decide the tier; empty `languages` binds the default pack
- DE/EN voice suites (`wohnung_mittel`, `wohnung_en`, `familienhaus_de`, `family_home_en`) are the **oracle** graphs; other locales overlay native sentences on those same graphs

## Dataset generation (every locale, local)

One command writes parity overlays for every generated locale (not Russian):

```bash
python3 scripts/parity/generate.py
```

It reads the DE oracles (`wohnung_mittel`, `familienhaus_de`, `m0_exact`, `m2_floors`) and the locale lexicon, then writes `tests/datasets/parity/{code}/{suite}/`. Room aliases go to `tests/datasets/parity/rooms.yaml`.

DE and EN are not overlays: they **are** the oracles. Regenerate those with `python3 scripts/gen_voice_suite.py` (and the family-home scripts in `docs/en/testing.md`). Then re-run `scripts/parity/generate.py` so every other locale stays in lockstep.

CI checks that this generator is a no-op (freshness) and runs the full `parity_langs` matrix (every compiled locale, no fail-fast). Locally:

```bash
python3 scripts/lang_packs/generate.py   # packs + assist smokes
python3 scripts/parity/generate.py       # per-locale datasets
python3 scripts/check_lang_packs.py
cargo nextest run --locked -E 'test(/assist_langs|language|parity_langs|voice_suite/)'
```

## Natural Dutch commands

The Dutch pack supports separated particles (`doe … uit`, `schakel … aan`),
joined verbs (`uitdoen`, `aandoen`, `uitschakelen`, `inschakelen`), polite modal
requests, and plural `lampen`/`lichten`. These are lexical rules shared across
rooms and devices, not stored sentence matches. The source lexicon is
`scripts/lang_packs/lexicons_west.py`; regenerate only the Dutch pack when
changing it.

Window questions such as `Welke ramen staan er open?` produce a read-only
`HassGetState` query with `domain: binary_sensor`, `device_class: window`,
`state: on`, and `response: list`. Closed-window questions use `state: off`.
An explicit known room scopes the query; unknown rooms and conflicting states
are rejected. Home Assistant matches exposed window sensors at query time;
Klar does not infer live state from its home graph. The list is spoken in Dutch,
including an explicit empty result, and does not include unmatched sensors.
`response` controls Klar's speech and is removed before invoking the HA intent.

Dutch negation (`niet`, `geen`, `nooit`, `nee`, `zonder`) fails closed before
custom rules, confirmation, and session replay. A negated utterance rejects
the whole request and clears pending clarification/confirmation. Negated
queries and free-text payloads containing these words are conservatively
rejected until their scope can be represented safely; negation is never removed
as filler or turned into an opposite action.

Regression coverage: `cargo nextest run --locked -E 'test(/dutch/)'`, plus the Dutch parity suite and
HA dispatch, floor-query, and speech-snapshot tests. All tests run against local
fixtures; they do not contact a running Home Assistant installation.
