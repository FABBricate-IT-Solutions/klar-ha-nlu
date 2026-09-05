# OpenAI-compatible LLM client — ADR 0002

`nlu::parse` stays deterministic and offline. Klar still owns **outbound** OpenAI-compatible chat (`/v1/chat/completions`, SSE streaming) so trainer and speech fallback do not keep a second Python client.

## Decision

- **Rust** (`src/llm/`): HTTP client, SSE parser, request caps, trainer system prompt.
- **Python**: glue only. Copy `base_url` / `api_key` / `model` from the HA conversation agent onto `POST /api/v2/llm/endpoint`. Stream Assist deltas from Klar events. `async_converse` remains the fallback when the agent is not OpenAI-compatible.
- The API key is **runtime memory** (and `KLAR_LLM_*` env). It is not written to the overlay. `GET /api/v2/llm/endpoint` never returns the key.
- Trainer: `POST /api/v2/policies/trainer/chat` builds context on the engine, streams tokens, extracts JSON, runs `validate`. Apply stays a human action on the lane write API.
- Speech: HA prefers `POST /api/v2/llm/chat` (stream). Refine uses the same endpoint with `stream: false`.

## Not this

- No model on the parse hot path.
- No new PolicyId matchers from the trainer.
- Python must not grow a parallel OpenAI SDK path for new features.
