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

CI uses `cargo nextest run --locked --profile ci`: same assist smoke for every compiled locale. If a PR touches `src/lang/packs/{code}/`, a de/en pack, or that locale's datasets, CI also runs that locale's Wohn+Family suite (`scripts/ci_lang_tests.py`). de/en stay a hard gate; other locales are report-only until fail==0. A generator rewrite that touches more than 8 locales is skipped (use `language-parity.yml`). Locally, `cargo nextest run` still runs every locale.

Do **not** run `python3 scripts/lang_packs/generate.py` in pre-commit or as a drive-by. Regeneration is a deliberate pack change.

Per-locale **datasets** (Wohn+Family+m0+m2 overlays): `python3 scripts/parity/generate.py`. Same rubric for every generated locale. DE/EN oracles: `python3 scripts/gen_voice_suite.py`, then re-run the parity generator.

## Language packs

- Hand-written reference: `src/lang/de_pack.rs`, `src/lang/en_pack.rs` (verb tables in `src/lang/de.rs` / `en.rs`)
- Generated locales: `src/lang/packs/{code}/` via `scripts/lang_packs/generate.py`
- Generated locales use the same `LanguagePack` path as the hand-written de/en reference packs
- Russian (`ru`) is not shipped

How to add a pack: [docs/en/languages.md](docs/en/languages.md). Review generated Rust like hand-written code. `de`/`en` suites stay green first.

## PR checklist

- [ ] `cargo nextest run` (or the CI profile) is green
- [ ] No secrets (`KLAR_TOKEN`, unredacted bundles)
- [ ] Docs twins: if you add `docs/en/foo.md`, add `docs/foo.md` (German)
- [ ] Do not special-case de+en as the only real Assist languages
