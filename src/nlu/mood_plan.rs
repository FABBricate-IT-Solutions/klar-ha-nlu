//! What a plan step means to the mood guard: does it change something, does it move a
//! device now, and which state query stands in for it.

use crate::lang::{Catalog, VerbKind};
use crate::types::{HomeGraph, Intent};

/// Slots that carry what the user dictated. Mood words inside them are content.
pub(super) const PAYLOAD_SLOTS: &[&str] = &["item", "search_query", "media_id", "summary", "name", "artist"];

/// Reading a state, the calendar or a queue changes nothing. A reminder is the one
/// supported way to say "later", so its wording is its own.
pub(super) fn changes_state(name: &str) -> bool {
    !matches!(
        name,
        "HassGetState"
            | "HassClimateGetTemperature"
            | "HassTimerStatus"
            | "MassGetQueue"
            | "KlarGetCalendarEvents"
            | "KlarNoMusicPlayer"
            | "SetReminderRelative"
            | "SetReminderAbsolute"
            | "SetReminderDaypart"
    )
}

/// Intents that switch or move a device now. Timers, lists and the calendar carry
/// their own times, so a time word does not close them.
pub(super) fn controls_device(name: &str) -> bool {
    matches!(
        name,
        "HassTurnOn"
            | "HassTurnOff"
            | "HassToggle"
            | "HassLightSet"
            | "HassClimateSetTemperature"
            | "HassSetPosition"
            | "HassOpenCover"
            | "HassCloseCover"
            | "HassLock"
            | "HassUnlock"
            | "HassFanSetSpeed"
            | "HassFanSetPercentage"
            | "HassVacuumStart"
            | "HassVacuumReturnToBase"
            | "HassMediaPause"
            | "HassMediaUnpause"
            | "HassMediaNext"
            | "HassMediaPrevious"
            | "HassMediaPlayerMute"
            | "HassMediaPlayerUnmute"
            | "HassSetVolume"
            | "HassSetVolumeRelative"
            | "MassTransferQueue"
    )
}

/// The step sets exactly this number, so a bare "at 21" or "um 2" is its value.
pub(super) fn sets_value(intent: &Intent, number: i32) -> bool {
    ["brightness", "brightness_step", "temperature", "position", "percentage", "volume_level", "volume_step"]
        .iter()
        .filter_map(|slot| intent.slot(slot))
        .any(|value| value.parse::<f64>().is_ok_and(|value| (value.abs() - f64::from(number)).abs() < 0.01))
}

/// The clause carries a word that switches, opens, starts or stops something. Such a
/// command is complete without a number, so a number after a clock word is an hour.
pub(super) fn switches(tokens: &[String], cat: &Catalog) -> bool {
    let words = [cat.on_words(), cat.off_words(), cat.open_words(), cat.close_words()];
    words.iter().any(|set| cat.any(tokens, set)) || tokens.iter().any(|token| cat.verb(token).is_some_and(completes))
}

/// Verb kinds that need no value to make a command.
fn completes(kind: VerbKind) -> bool {
    matches!(
        kind,
        VerbKind::On
            | VerbKind::OnParticle
            | VerbKind::Off
            | VerbKind::Open
            | VerbKind::OpenDoor
            | VerbKind::Close
            | VerbKind::Stop
            | VerbKind::Toggle
            | VerbKind::Pause
            | VerbKind::Play
            | VerbKind::Playback
            | VerbKind::Next
            | VerbKind::Mute
            | VerbKind::Vacuum
            | VerbKind::Dock
            | VerbKind::Scene
    )
}

/// The same intent on the same slots, in whatever order two parses listed them.
pub(super) fn same_step(left: &Intent, right: &Intent) -> bool {
    left.name == right.name
        && left.slots.len() == right.slots.len()
        && left.slots.iter().all(|slot| right.slots.iter().any(|other| other == slot))
}

/// The state query that answers a question about the target of `intent`.
pub(super) fn state_query(intent: &Intent, home: &HomeGraph) -> Option<Intent> {
    let query = Intent::new("HassGetState");
    if let Some(entity_id) = intent.slot("entity_id") {
        let entity = home.entities.iter().find(|entity| entity.entity_id == entity_id)?;
        // A scene or a script has no state worth reading back.
        if matches!(entity.domain.as_str(), "scene" | "script") {
            return None;
        }
        return Some(query.with("entity_id", entity_id));
    }
    let domain = intent.slot("domain");
    if matches!(domain, Some("scene" | "script")) {
        return None;
    }
    let query = match (intent.slot("area"), intent.slot("floor")) {
        (Some(area), _) => query.with("area", area),
        (None, Some(floor)) => query.with("floor", floor),
        (None, None) => return None,
    };
    Some(match domain {
        Some(domain) => query.with("domain", domain),
        None => query,
    })
}
