# OpenAI-compatible LLM client — ADR 0002

`nlu::parse` stays deterministic and offline. Klar still owns **outbound** OpenAI-compatible chat (`/v1/chat/completions`, SSE streaming) so trainer and speech fallback do not keep a second Python client.

## Decision

- **Rust** (`src/llm/`): HTTP client, SSE parser, request caps, trainer system prompt.
- **Python**: glue only. Stream Assist deltas from Klar events. Do not copy a HA conversation agent onto Klar (that overwrites operator Settings). `async_converse` remains a legacy fallback when an old agent is still configured.
- The API key is stored in `data_dir/llm_endpoint.json` (not the overlay). `KLAR_LLM_*` env wins on boot. `GET /api/v2/llm/endpoint` never returns the key. Operator UI Settings is the config surface; Assist does not need a second Home Assistant conversation integration.
- Trainer: `POST /api/v2/policies/trainer/chat` builds context on the engine, streams tokens, extracts JSON, runs `validate`. Apply stays a human action on the lane write API.
- Speech: HA prefers `POST /api/v2/llm/chat` (stream) even when no fallback agent is configured. A leftover HA conversation agent remains an optional legacy path.

## Not this

- No model on the parse hot path.
- No new PolicyId matchers from the trainer.
- Python must not grow a parallel OpenAI SDK path for new features.

## See also

Remaining Python product logic (refine accept, Assist system prompts, post-execute speech) is [ADR 0003](adr-0003-python-rust-boundary.en.md) · [plan](adr-0003-plan.en.md). That cycle also deletes the leftover OpenAI SDK calls on Assist paths.
