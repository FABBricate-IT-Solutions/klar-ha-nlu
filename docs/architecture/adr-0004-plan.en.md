# Implementation plan — ADR 0004

[Deutsch](adr-0004-plan.md) · [English](adr-0004-plan.en.md)

Frame: [ADR 0004](adr-0004-operator-console.en.md). Each stage is its own PR **against `staging`**. Defaults stay today’s Assist behavior until someone changes Settings. No calendar.

## Delivery channel: staging, not a main release

Same channel rules as ADR 0001 / 0003: base every PR on `staging`; after merge the existing staging workflow tags a prerelease and docker `staging`; `staging` → `main` is not part of this plan.

## Stage 1 — Engine is the product settings store (shipped)

**Goal.** Home Assistant Configure shrinks to connection glue. Operator Settings becomes the guided editor. Assist reads engine settings.

| Change | Detail |
|--------|--------|
| `Settings.extra_prompt` | Optional extra refine/assist line; empty = packed personality only |
| HA options schema | Keep mode, channel, URL, token, fallback agent, assist filter. Drop personality, languages, refine, quiet ack, RAG, calendar LLM, tools |
| Seed | One-time POST of non-default HA product options onto `/api/settings`, then `product_in_engine` |
| Assist | Cache `GET /api/settings` per turn; fall back to leftover options only when the cache is empty |
| Select / switch | Patch the engine, not `entry.options` (options remain last-resort fallback) |
| Operator Settings | Guided cards: Voice, Assist languages, LLM, misses, this screen, diagnostics |
| Copy | HA strings + operator i18n stop sending people to the integration form for voice/language |

**Gate:** `python3 -m unittest discover -s tests -p 'test_*.py'` (config_flow schema, engine merge/seed, conversation still reads options when cache empty). `cargo nextest run --locked` for the `extra_prompt` default. Web typecheck.

**Rollback:** leave leftover option keys in `entry.options`; Assist falls back if GET fails.

## Stage 2 — Figma 05 Staging Overhaul

New page on [Klar Visual Refresh](https://www.figma.com/design/IOMwQ0Fkkg3YhFTfkRhGed), do not smash **04 Visual Refresh**. Screens: guided Settings, Home, Rules path, Lab path, House graph. IBM Plex Sans. shadcn mapping, not Code Connect.

Needs: Figma file edit access; a running staging UI helps `generate_figma_design` screenshots.

## Stage 3 — Apply the overhaul in `web/`

Implement Figma 05 with existing `web/src/components/ui/*`. Better path visualization and lane controls. Browser-verify clicks, not a single screenshot. Operator chrome: same keys for every compiled Assist locale, translated — not English leftovers.

## Stage 4 — Wizard owns first-run voice + LLM (this PR)

Wizard steps write `/api/settings` and `/api/v2/llm/endpoint` so a new house never opens HA Configure for product knobs. Replay setup from Settings stays. Wizard chrome is generated for every compiled Assist locale.

## Out of scope

- Promoting `staging` → `main`
- CalVer bump
- Putting product flags back into Python
- Rewriting `conversation.py` as a Rust plugin
- Removing the personality select / quiet-ack switch (automations)
