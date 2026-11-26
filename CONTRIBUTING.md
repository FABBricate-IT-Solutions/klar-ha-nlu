# Contributing

Public name: **Klar NLU**. Household docs: [getting started](docs/en/getting-started.md) · [Einstieg](docs/getting-started.md).

## Basics

- MSRV **1.85** (`rust-toolchain.toml`, `Cargo.toml` `rust-version`)
- One module, one job. There is no line limit to stay under; see [Guardrails](#guardrails)
- Do not invent household languages. Every compiled locale runs through the same engine path; do not restore a de+en-only default catalog. A locale's tier (`reference`, `beta` or `experimental`) says what is tested: it is decided by the minimal pairs in `tests/language_tiers.rs`, never by hand. Every shipped locale is reference; a new locale starts experimental and is listed in `scripts/lang_packs/tiers.py` once it earns reference (it needs its own negation words in `scripts/lang_packs/mood.py`)
- Conventional Commits (`feat:`, `fix:`, `docs:`). Releases: [docs/en/releases.md](docs/en/releases.md). Land language/feature work on `staging` via PR; do not push `staging` or `main` directly.

## Tests

```bash
cargo fmt --check
cargo check
cargo nextest run
```

CI uses `cargo nextest run --locked --profile ci --no-fail-fast`: assist smoke plus the full Wohn+Family+m0+m2 parity suite for every compiled locale. Pull requests into `staging` use the profile `ci-staging` (no multi-locale parity). Klar's own dataset suites (`voice_suite` apartment suites, `full_home` quick per locale and full for de/en, `parity_own_*`) run on every pull request in the job `dataset-suites`: `cargo nextest run --locked --profile ci-datasets --run-ignored all --test voice_suite --test full_home --test parity_langs`.

Do **not** run `python3 scripts/lang_packs/generate.py` in pre-commit or as a drive-by. Regeneration is a deliberate pack change.

Per-locale **datasets** (Wohn+Family+m0+m2 overlays): `python3 scripts/parity/generate.py`. Same rubric for every generated locale. DE/EN oracles: `python3 scripts/gen_voice_suite.py`, then re-run the parity generator.

Two checks CI runs that you can run before pushing:

- `bash scripts/check_generated.sh` runs every generator and fails when a file in the tree changes, translations included. Change the generator source, not its output.
- `bash scripts/lint_ci.sh` lints the workflows and shell scripts (actionlint, zizmor, shellcheck; needs `uv`). `python3 scripts/check_links.py` checks the links between the Markdown files.

Workflows pin every action by commit SHA with the release in a trailing comment. Tests take scratch directories from `tempfile` (`TestDir` in unit tests), never a fixed name under `/tmp`.

## Guardrails

`python3 scripts/check_guardrails.py` runs in CI (job `guardrails`). Its data is `scripts/guardrails.toml`.

- **Module responsibility.** Above 500 lines a hand-written module needs an entry with its owner and the one job it does. There is no failure at 499 versus 501, so never move lines only to get under a number. Split when the entry would need "and" for a second, unrelated job; a logic module past 1000 lines must be split along those jobs. Generated files and tests are exempt.
- **ADR 0003 boundary.** No new regex in `custom_components/klar_nlu/`. Reading what a person said belongs in the Rust engine ([ADR 0003](docs/architecture/adr-0003-python-rust-boundary.en.md)). The few regexes that remain are listed per file with a kind and a ceiling; a ceiling only goes down.
- **Focus.** The news briefing, Wikipedia answers, personality few-shots and the custom-voice interview are frozen non-core areas. They ship off by default and are owned in `.github/CODEOWNERS`. A `fix` or `feat` commit that mostly changes them needs a `Frozen-area: security|data-loss|regression|legal` trailer. The share of fixes that went to news and LLM paths is printed by `python3 scripts/check_guardrails.py share` and should stay below 5 %.

## Language packs

- Hand-written reference: `src/lang/packs/de/`, `src/lang/packs/en/`
- Generated locales: compiled under `src/lang/packs/{code}/` via `scripts/lang_packs/generate.py`, data form under `packs/generated/{code}.yaml`
- Phrase corrections: **one file** `packs/assist/{code}.yaml` — see [Fix a phrase in three steps](docs/en/fix-phrase.md) / [Phrase in drei Schritten](docs/fix-phrase.md). Use `klar lang new-case`.
- Do not hand-edit generated Rust under `src/lang/packs/` (except de/en)
- Russian (`ru`) is not shipped

How to add a pack: [docs/en/languages.md](docs/en/languages.md). `de`/`en` suites stay green first.

## PR checklist

- [ ] `cargo nextest run` (or the CI profile) is green
- [ ] No secrets (`KLAR_TOKEN`, unredacted bundles)
- [ ] Docs twins: if you add `docs/en/foo.md`, add `docs/foo.md` (German)
- [ ] Do not special-case de+en as the only real Assist languages (a locale becomes reference by passing `tests/language_tiers.rs`)
