//! Post-execute Assist speech. Interpolates pack templates from an HA snapshot.
//! Never called from `nlu::parse`. Personality prefix stays in Assist finish.
//! ASR boost prompts for external Whisper live here too (no STT engine in Klar).

mod asr_boost;
mod generated;
mod render;
mod render_climate;
mod render_media;
mod render_place;
mod render_status;
mod render_weather;
mod weather_i18n;

pub use asr_boost::{build_asr_boost, AsrBoostOut, AsrBoostTerm, ASR_BOOST_SCHEMA, DEFAULT_MAX_TOKENS, MAX_MAX_TOKENS};
pub use generated::ACTION_TEMPLATES;
pub use render::render_snapshot;
