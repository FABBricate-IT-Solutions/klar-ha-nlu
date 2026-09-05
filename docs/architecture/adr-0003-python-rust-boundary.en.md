# ADR 0003 — Engine owns Assist product logic; HA stays the platform

[Deutsch](adr-0003-python-rust-boundary.md) · [English](adr-0003-python-rust-boundary.en.md)

Status: **proposed** — direction for a staging cycle. Implementation: [plan](adr-0003-plan.en.md). Ships on **`staging`**, not a main release.

Klar stays a deterministic, local NLU. `nlu::parse` has no network and no model. An LLM may **talk and rewrite**; it must not **drive parse**. Product rules that are not Home Assistant platform glue belong in the Rust engine, not in `custom_components/klar_nlu/`.

This ADR does not replace [ADR 0002](adr-0002-openai-llm-client.md) (outbound OpenAI-compatible client) or [ADR 0001](adr-0001-rules-and-trainer.en.md) (visible match / language / house). It finishes the ownership line those ADRs started: Python is glue; the engine owns speech product rules, Assist system prompts, and post-execute templates.

## Context

The Home Assistant integration is ~8500 lines. About one third is product logic that conceptually belongs in Rust (refine accept/prompt, Yarn/chat/RAG prompts, post-execute spoken sentences, quiet-ack eligibility). The rest **must** stay Python: binary lifecycle, config flow, registry sync, HA service execution, ConversationEntity/ChatLog orchestration, leftover-agent field reads, ESPHome/assist_satellite chime, entities/panel/services.

Today that split is violated:

- Assist LLM paths still grow a **parallel OpenAI SDK** (`refine.py` `_async_refine_raw`, `stream.py` `iter_completion_tokens`, `conversation.py` `_fallback` / `_stream_fallback`) after the engine `/api/v2/llm/chat` path. ADR 0002 forbids growing that.
- Refine **accept** rules, weather-invention guards, and personality prompt blocks live in Python while HTTP already goes through Klar.
- Fallback / Yarn / RAG **system prompts** and the `KLAR_PARSE:` / `KLAR_ACT:` protocol live in Python. HA should only stream Assist deltas.
- `src/parse/respond.rs` already documents that Assist **overwrites** parse speech with `speech.py` after the HA intent so the spoken line matches what actually ran. Those templates need a **live HA state snapshot**; the home graph is not enough (climate `current_temperature`, MASS `media_title` / `volume_level`, calendar rows, local clock).
- Quiet-ack eligibility is a product rule (`quiet_ack_applies`); `play_chime` is platform glue.

`scripts/lang_packs/` (~12k LOC) stays Python codegen. Do not rewrite generators in Rust. Post-execute templates move by **extending** that generator so it emits into `src/lang/packs/*/speech.rs` (and de/en by hand, as today).

## Decision

### Ownership

| Layer | Owner | Not |
|-------|--------|-----|
| `nlu::parse`, ranking, policy, pack lexicon | Engine | No model, no HA I/O |
| OpenAI-compatible HTTP + SSE (`src/llm/`) | Engine | No second Python SDK for new paths |
| Refine prompt, `accept_refined`, weather/number/stamp guards, voice blocks | Engine | Python must not keep a second copy as the source of truth |
| Yarn / chat-only / news / calendar / RAG system prompts; `KLAR_*` protocol parse | Engine | HA does not assemble those strings |
| Post-execute speech templates (acks, queries, media, calendar say, clock, floor/area status) | Engine, from a **snapshot** HA sends after execute | Engine must not scrape live HA state |
| Quiet-ack **eligibility** | Engine (flag on execute plan / outcome) | Chime playback |
| ChatLog deltas, ConversationEntity, `async_converse` legacy | Python | Do not rewrite `conversation.py` as a Rust HA plugin |
| Engine process, config flow, registry sync, HA services, expose, ESPHome chime | Python | — |
| `contracts.py` `validate_v2_payload` | Python (client schema guard) | Do not “dedupe” into Rust |
| `intents.py` `_fold_latin` / `_umlaut_eq` | Python freeze; area resolve vs HA registry is glue | Do not grow; canonical fold is `src/parse/normalize.rs` |
| `llm_endpoint.py` leftover HA agent fields | Python | Copy into engine settings; do not drive chat |
| Lang-pack **generators** | Python `scripts/lang_packs/` | Do not port the generator |

### Parse stays offline

`POST /api/v2/parse` does not call the LLM, does not fetch HA state, and does not render post-execute speech. Parse speech remains the Wyoming / Lab / pre-execute line from `respond.rs`. Assist continues to replace it after devices run.

Quiet-ack **eligibility** may be a boolean derived from the execute plan (one `HassTurnOn`/`HassTurnOff` on light/switch). That is still offline. Python confirms success after dispatch, then plays the chime.

### LLM: purpose routes, not a second client

Keep `POST /api/v2/llm/chat` as the raw transport (messages in, SSE or JSON out). New Assist product calls do **not** send a prebuilt system prompt from Python.

| Route | Role |
|-------|------|
| `POST /api/v2/llm/chat` | Raw messages. Operator/debug. Unchanged. |
| `POST /api/v2/llm/refine` | Engine builds the refine prompt from pack + personality + extra; runs the model; applies `accept_refined`; returns accepted speech or the original. |
| `POST /api/v2/llm/assist` | Engine classifies Yarn/chat/RAG/calendar/news (or honors an explicit `kind`); builds the system prompt; streams the same SSE events as chat. Structured `tool` events for `klar.parse` / `klar.act` — no leaked `KLAR_PARSE:` line into TTS. |
| `POST /api/v2/policies/trainer/chat` | Unchanged (ADR 0001 / 0002). |

Python streams Assist deltas from those events (`engine_llm.py` + `stream.py` ChatLog glue). `async_converse` remains the **documented legacy** when an old HA agent is still configured **and** the engine returns 503 (no endpoint). Do not grow `client.chat.completions.create`.

`agent_has_home_control` / `can_use_fallback_agent` stay Python: they read HA `ConversationEntityFeature`. The engine receives `allow_tools: bool`; it does not inspect HA agent objects.

### Post-execute speech: snapshot contract

Naive port of `speech.py` without live state is blind: parse-time `respond.rs` does not see climate attributes, MASS now-playing, or HA `matched_states` after intent.

New **write** route, not on the parse path:

`POST /api/v2/speech/render`

- Request: schema versioned snapshot (allow-listed entity fields, optional calendar rows, optional media queue, `now` in the house timezone, intent + execute outcome, language, personality).
- Response: `{ "speech": "…", "quiet_ack": false, "source": "post_execute" }`.
- Caps and unknown keys: reject or drop; never pass raw HA state objects through.
- Personality **prefix** is applied once, at Assist finish (`style` / refine), not inside the renderer and again in Python.

`POST /api/v2/home` stays the graph sync. Do not overload it with live attributes.

Until the renderer exists, Python keeps `speech.py`. After parity, Assist calls render and `speech.py` becomes a 404 fallback, then is deleted.

## Consequences

### Positive

- One owner for spoken product rules and Assist prompts; every locale goes through packs + `scripts/lang_packs`, not a growing Python dictionary.
- ADR 0002 becomes true in Assist, not only in the trainer.
- Post-execute speech can match what ran without putting HA I/O in `nlu::parse`.

### Negative

- Version skew: an old engine has no `/llm/refine` or `/speech/render`. Staging ships engine + integration together; Python keeps a 404 fallback for one bake, then deletes the duplicate.
- Dual speech sources exist **today** (`respond.rs` vs `speech.py`). The renderer must not TTS both. Assist already overwrites parse speech after execute; keep that.

### Neutral

- `conversation.py` stays the HA orchestrator. It gets thinner (no prompt strings, no accept rules, no template format).
- Freeze `intents.py` fold helpers; do not “clean up” them into a shared crate.

## Not this

- Rewriting `scripts/lang_packs/` in Rust
- Rewriting `conversation.py` as a Rust Home Assistant plugin
- LLM or network inside `nlu::parse`
- New `PolicyId` matchers (ADR 0001)
- Deleting `validate_v2_payload` or growing Python `_fold_latin`
- Promoting this cycle `staging` → `main` without an explicit go-ahead

## Links

- [Implementation plan](adr-0003-plan.en.md)
- [ADR 0002 — OpenAI-compatible LLM client](adr-0002-openai-llm-client.md)
- [ADR 0001 — Visible rules and trainer](adr-0001-rules-and-trainer.en.md)
- `src/parse/respond.rs`, `src/llm/`, `custom_components/klar_nlu/{refine,fallback,rag_tools,speech,quiet,conversation,stream,engine_llm}.py`
