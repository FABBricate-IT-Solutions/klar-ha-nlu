//! OpenAI-compatible chat client. Never called from `nlu::parse`.

mod client;
mod endpoint;
mod extract;
mod sse;
mod trainer;
mod types;

pub use client::{chat, chat_stream, LlmClient};
pub use endpoint::{LlmEndpoint, LlmPublic};
pub use extract::json_object;
pub use trainer::{history_messages, system_prompt, TrainerTurn};
pub use types::{ChatEvent, ChatMessage, ChatRequest, LlmError};

#[cfg(test)]
mod tests;
