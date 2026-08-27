use crate::parse::action::Action;
use crate::parse::slots::ClauseOut;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PolicyId {
    LaundrySwitch,
    Timer,
    Calendar,
    List,
    Media,
    NamedScene,
    AllLights,
    FollowNamed,
    PreferredAreaCommand,
    AreaCommand,
    FloorCommand,
    QueryArea,
    QueryUngrounded,
    MultiArea,
    GroundedEntities,
    GroundedAmbiguous,
    GroundedAreas,
    SessionClimateCover,
    SessionEntities,
    SessionAreas,
    LightRoomsClarify,
    FallbackTemp,
    FallbackCover,
    LeftoverCommand,
}

#[derive(Debug, Clone)]
pub(crate) struct ClauseCandidate {
    pub policy: PolicyId,
    pub precedence: u16,
    pub score: f64,
    pub action: Action,
    pub outcome: ClauseOut,
}

impl PolicyId {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::LaundrySwitch => "laundry_switch",
            Self::Timer => "timer",
            Self::Calendar => "calendar",
            Self::List => "list",
            Self::Media => "media",
            Self::NamedScene => "named_scene",
            Self::AllLights => "all_lights",
            Self::FollowNamed => "follow_named",
            Self::PreferredAreaCommand => "preferred_area_command",
            Self::AreaCommand => "area_command",
            Self::FloorCommand => "floor_command",
            Self::QueryArea => "query_area",
            Self::QueryUngrounded => "query_ungrounded",
            Self::MultiArea => "multi_area",
            Self::GroundedEntities => "grounded_entities",
            Self::GroundedAmbiguous => "grounded_ambiguous",
            Self::GroundedAreas => "grounded_areas",
            Self::SessionClimateCover => "session_climate_cover",
            Self::SessionEntities => "session_entities",
            Self::SessionAreas => "session_areas",
            Self::LightRoomsClarify => "light_rooms_clarify",
            Self::FallbackTemp => "fallback_temperature",
            Self::FallbackCover => "fallback_cover",
            Self::LeftoverCommand => "leftover_command",
        }
    }

    const fn precedence(self) -> u16 {
        match self {
            Self::LaundrySwitch => 0,
            Self::Timer => 1,
            Self::Calendar => 1,
            Self::List => 2,
            Self::Media => 3,
            Self::NamedScene => 4,
            Self::AllLights => 5,
            Self::FollowNamed => 6,
            Self::PreferredAreaCommand => 7,
            Self::AreaCommand | Self::FloorCommand => 8,
            Self::QueryArea => 9,
            Self::QueryUngrounded => 10,
            Self::MultiArea => 11,
            Self::GroundedEntities => 12,
            Self::GroundedAmbiguous => 13,
            Self::GroundedAreas => 14,
            Self::SessionClimateCover => 15,
            Self::SessionEntities => 16,
            Self::SessionAreas => 17,
            Self::LightRoomsClarify => 18,
            Self::FallbackTemp => 19,
            Self::FallbackCover => 20,
            Self::LeftoverCommand => 21,
        }
    }
}

pub(crate) fn candidate(policy: PolicyId, action: Action, outcome: ClauseOut) -> ClauseCandidate {
    let precedence = policy.precedence();
    ClauseCandidate { policy, precedence, score: (1.0 - f64::from(precedence) * 0.0125).max(0.7), action, outcome }
}

pub(crate) fn media_claimed_empty(candidates: &[ClauseCandidate]) -> bool {
    candidates.iter().any(|candidate| {
        candidate.policy == PolicyId::Media && matches!(&candidate.outcome, ClauseOut::Intents(intents) if intents.is_empty())
    })
}

pub(crate) fn retain_after_media_claim(candidates: &mut Vec<ClauseCandidate>, named: bool) {
    candidates.retain(|candidate| match candidate.policy {
        PolicyId::GroundedEntities | PolicyId::GroundedAmbiguous => named,
        policy => media_fallback_allowed(policy),
    });
}

pub(crate) fn media_fallback_allowed(policy: PolicyId) -> bool {
    matches!(
        policy,
        PolicyId::Media
            | PolicyId::LaundrySwitch
            | PolicyId::Timer
            | PolicyId::Calendar
            | PolicyId::List
            | PolicyId::FollowNamed
            | PolicyId::GroundedEntities
            | PolicyId::GroundedAmbiguous
    )
}
