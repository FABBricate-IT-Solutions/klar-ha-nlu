pub const MAX_PARSE_CHARS: usize = 4096;
// Registry snapshots include every enabled entity plus device/place/exposure metadata.
// Bound the total bytes independently of the per-collection and per-field caps.
pub const MAX_HOME_SNAPSHOT_BYTES: usize = 8 * 1024 * 1024;
/// Body cap for `POST /api/v2/speech/render`. The snapshot schema allows 32 entities and
/// 48 hourly rows (~150 KiB JSON); the old 16 KiB limit rejected valid ~18 KiB bodies (R-18).
pub const MAX_SPEECH_RENDER_BYTES: usize = 256 * 1024;

/// Caps for `POST /api/v2/policies/propose/validate` (A-06).
pub const MAX_PROPOSAL_UTTERANCES: usize = 32;
pub const MAX_PROPOSAL_MATCH_CONTROLS: usize = 64;
pub const MAX_PROPOSAL_CUSTOM: usize = 64;
