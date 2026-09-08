//! Post-execute Assist speech. Interpolates pack templates from an HA snapshot.
//! Never called from `nlu::parse`. Personality prefix stays in Assist finish.

mod generated;
mod render;
mod render_media;
mod render_place;

pub use generated::ACTION_TEMPLATES;
pub use render::render_snapshot;
