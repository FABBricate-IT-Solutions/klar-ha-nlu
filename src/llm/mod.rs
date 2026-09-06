//! OpenAI-compatible chat client. Never called from `nlu::parse`.

mod assist;
mod assist_prompt;
mod assist_rag;
mod assist_yarn;
mod client;
mod custom_voice;
mod endpoint;
mod extract;
mod refine;
mod refine_accept;
mod refine_prompt;
mod refine_shots;
mod refine_shots_a;
mod refine_shots_b;
mod refine_shots_c;
mod refine_shots_d;
mod refine_shots_e;
mod refine_shots_f;
mod refine_voices;
mod sse;
mod trainer;
mod trainer_choices;
mod trainer_handbook;
mod trainer_tools;
mod types;

pub use assist::{assist, assist_on, AssistOutcome, AssistRequest};
pub use assist_prompt::{keeps_calendar_reply, AssistKind};
pub use assist_rag::leaks_klar_tools;
pub use assist_yarn::yarn_request;
pub use client::{chat, chat_stream, chat_stream_turn, chat_turn, list_models, LlmClient};
pub use custom_voice::{generate_custom_voice, CustomVoiceOut, CustomVoiceRequest};
pub use endpoint::{LlmEndpoint, LlmPublic};
pub use extract::{json_array, json_object};
pub use refine::{refine, refine_on, RefineOutcome, RefineRequest};
pub use refine_accept::{accept_refined, streams_refine_prefix, weather_claim};
pub use refine_prompt::{personality_preview, personality_preview_for, refine_prompt, refine_prompt_for, usable_extra, PersonalityPreview};
pub use trainer::{history_messages, system_prompt, TrainerTurn};
pub use trainer_choices::{asks_operator, ensure_reply_choices, parse_lotse_choices};
pub use trainer_tools::{is_write_tool, openai_tools, openai_tools_for, parse_text_tools, tool_allowed_for_layer, WRITE_TOOLS};
pub use types::{ChatEvent, ChatMessage, ChatRequest, CompletionTurn, LlmError, ToolCall};

#[cfg(test)]
mod tests;
