# ADR 0004 — Operator UI is the product console; Home Assistant stays glue

[Deutsch](adr-0004-operator-console.md) · [English](adr-0004-operator-console.en.md)

Status: **proposed** — direction for a staging cycle. Implementation: [plan](adr-0004-plan.en.md). Ships on **`staging`**, not a main release.

Klar stays a deterministic, local NLU. `nlu::parse` has no network and no model. This ADR does not replace [ADR 0001](adr-0001-rules-and-trainer.en.md) (visible match / language / house), [ADR 0002](adr-0002-openai-llm-client.md) (engine LLM client), or [ADR 0003](adr-0003-python-rust-boundary.en.md) (engine-owned Assist product logic). It finishes the **operator** line those ADRs started: the person who runs Klar configures Klar **in Klar**, not in a long Home Assistant options form.

## Context

After ADR 0001–0003 the engine already stores personality, languages, refine, quiet ack, RAG, calendar LLM, and tool flags in `GET`/`POST /api/settings`, and the LLM endpoint in `/api/v2/llm/endpoint`. The operator UI already has a Settings page and a first-run wizard.

Home Assistant still **owns** those product knobs in `config_flow` options, **pushes** them onto the engine on every reload (overwriting the operator UI), and **sends** them again on every parse. The Settings page tells operators to set personality and Assist language in **Home Assistant → Klar NLU**. That is the wrong console.

Desired:

- **Fewer** fields in the Home Assistant integration.
- **More**, and **guided**, configuration in the operator UI.
- Python/HA keep platform glue only (URL, token, expose, leftover conversation agent, chime, registry sync).

## Decision

### Source of truth

| Concern | Owner | Home Assistant |
|---------|--------|----------------|
| Personality, Assist packs, refine on/off, extra refine line, quiet ack, NLU-RAG, calendar LLM, allow LLM tools, confirm-risky, mode | Engine `/api/settings` via operator UI | Read cache; do not overwrite after a one-time seed |
| LLM endpoint (URL, model, key) | Engine `/api/v2/llm/endpoint` via operator UI | Not an HA options field |
| Match / lexicon / house rules, trainer | Operator UI + overlay | No |
| Engine URL, token, local vs app, release channel | HA config entry | Edit in HA (connection) |
| Fallback conversation agent id | HA options | HA entity id; engine only stores `fallback_llm: bool` |
| Assist expose filter | HA options | Registry glue |
| Personality select + quiet-ack switch entities | Thin proxies that **write the engine** | Automations may toggle; they are not the setup UI |

### Home Assistant options form (keep)

Setup (`user`) and Configure (`options`) keep only:

1. Engine (bundled vs app/Docker)
2. Release channel
3. URL + write token
4. Optional legacy conversation agent
5. Assist expose filter

Description copy points at the operator UI for voice, languages, and LLM.

### Migration

Existing houses that already set personality / flags in HA options: **one-time seed** onto the engine when those options are non-default, then a `product_in_engine` flag on the config entry. After that, HA must not POST product fields over operator changes.

If the engine is unreachable, Assist keeps using leftover `entry.options` until the next successful fetch. After a successful fetch, Assist uses the engine cache (refreshed per turn).

### Operator UI

Settings is a **guided console**, not a dump of the same HA form:

1. **Voice** — personality, extra prompt, refine, quiet ack
2. **Assist languages** — all packs, or pin one (empty `languages` = every compiled locale)
3. **LLM** — existing endpoint card (Assist chat + trainer)
4. **When Klar misses** — NLU-RAG, calendar LLM, allow tools, confirm-risky, resolve-devices vs rooms
5. **This screen** — theme, operator chrome language, write token
6. **Diagnostics** — support bundle, adapters

The wizard remains the first-run path and points here for voice and LLM. Lovelace “Klar” stays the last Assist turn, not the product console.

Operator chrome uses the same keys for every compiled Assist locale (`en.ts` / `de.ts` hand-written; `messages/*.json` generated). New Settings copy is translated, not left as English leftovers.

### Visual overhaul

A later staging PR restyles the rest of the operator chrome (Home, Rules path, Lab, House) against Figma **05 Staging Overhaul** on the Klar Visual Refresh file. This ADR does not require that restyle to land in the same PR as the settings move. Code Connect stays out of the repo.

## Consequences

- Operators configure Klar where they already calibrate the house and train lanes.
- HA Configure is short enough to finish on a phone.
- Reloading the integration no longer resets operator personality to the HA form default.
- Personality select / quiet-ack switch remain for automations; they patch `/api/settings`.
- Do not put these knobs back into Python product modules. Do not grow `config_flow` options again. Do not put an LLM on `nlu::parse`.
