# Contributing

Public name: **Klar NLU**. Household docs: [getting started](docs/en/getting-started.md) · [Einstieg](docs/getting-started.md).

## Basics

- MSRV **1.85** (`rust-toolchain.toml`, `Cargo.toml` `rust-version`)
- Files stay under **500 lines** (`scripts/check_lang_packs.py` enforces this on packs)
- Do not invent household languages. Every compiled locale is first-class; do not restore a de+en-only default catalog
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

## Language packs

- Hand-written reference: `src/lang/packs/de/`, `src/lang/packs/en/`
- Generated locales: `src/lang/packs/{code}/` via `scripts/lang_packs/generate.py`
- Generated locales use the same `LanguagePack` path as the hand-written de/en reference packs
- Russian (`ru`) is not shipped

How to add a pack: [docs/en/languages.md](docs/en/languages.md). Review generated Rust like hand-written code. `de`/`en` suites stay green first.

## PR checklist

- [ ] `cargo nextest run` (or the CI profile) is green
- [ ] No secrets (`KLAR_TOKEN`, unredacted bundles)
- [ ] Docs twins: if you add `docs/en/foo.md`, add `docs/foo.md` (German)
- [ ] Do not special-case de+en as the only real Assist languages
