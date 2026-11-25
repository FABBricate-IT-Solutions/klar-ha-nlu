pub const MAX_PARSE_CHARS: usize = 4096;
// Registry snapshots include every enabled entity plus device/place/exposure metadata.
// Bound the total bytes independently of the per-collection and per-field caps.
pub const MAX_HOME_SNAPSHOT_BYTES: usize = 8 * 1024 * 1024;

/// Caps for `POST /api/v2/policies/propose/validate` (A-06).
pub const MAX_PROPOSAL_UTTERANCES: usize = 32;
pub const MAX_PROPOSAL_MATCH_CONTROLS: usize = 64;
pub const MAX_PROPOSAL_CUSTOM: usize = 64;
