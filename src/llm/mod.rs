//! OpenAI-compatible chat client. Never called from `nlu::parse`.

mod assist;
mod assist_prompt;
mod assist_rag;
mod assist_yarn;
mod client;
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
mod types;

pub use assist::{assist, AssistOutcome, AssistRequest};
pub use assist_prompt::{keeps_calendar_reply, AssistKind};
pub use assist_rag::leaks_klar_tools;
pub use assist_yarn::yarn_request;
pub use client::{chat, chat_stream, LlmClient};
pub use endpoint::{LlmEndpoint, LlmPublic};
pub use extract::json_object;
pub use refine::{refine, RefineOutcome, RefineRequest};
pub use refine_accept::{accept_refined, weather_claim};
pub use refine_prompt::refine_prompt;
pub use trainer::{history_messages, system_prompt, TrainerTurn};
pub use types::{ChatEvent, ChatMessage, ChatRequest, LlmError};

#[cfg(test)]
mod tests;
