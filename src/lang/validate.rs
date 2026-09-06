//! Validate external packs against the schema and the builtin they extend.

use super::catalog::WordKey;
use super::external::{ExternalPack, PACK_FORMAT};
use super::locale::LocaleId;
use super::Catalog;
use crate::types::known_intent;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

impl ValidationReport {
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

pub fn validate_pack(pack: &ExternalPack, base: Option<&Catalog>) -> ValidationReport {
    let mut report = ValidationReport::default();
    if pack.klar_lang_pack != PACK_FORMAT {
        report.errors.push(issue("klar_lang_pack", format!("expected {PACK_FORMAT}, got {}", pack.klar_lang_pack)));
    }
    if pack.id.trim().is_empty() {
        report.errors.push(issue("id", "pack id is required"));
    }
    if LocaleId::parse(&pack.extends).is_err() {
        report.errors.push(issue("extends", format!("invalid builtin tag {}", pack.extends)));
    } else if pack.base_lang().is_err() {
        report.errors.push(issue("extends", format!("no builtin pack for {}", pack.extends)));
    }
    if pack.locales().is_err() {
        report.errors.push(issue("bcp47", "contains an invalid BCP-47 tag"));
    }
    match pack.verb_entries() {
        Ok(entries) => {
            if let Some(catalog) = base {
                for (token, kind) in entries {
                    if let Some(existing) = catalog.verb(&token) {
                        if existing != kind {
                            report.errors.push(issue(
                                &format!("verbs.{token}"),
                                format!("conflicts with builtin {} (use a new token or keep {existing:?})", pack.extends),
                            ));
                        }
                    }
                }
            }
        }
        Err(err) => report.errors.push(issue("verbs", err)),
    }
    for (path, words) in &pack.sets {
        if set_field(path).is_none() {
            report.errors.push(issue(&format!("sets.{path}"), "unknown set path"));
        }
        if words.iter().any(|word| word.trim().is_empty()) {
            report.errors.push(issue(&format!("sets.{path}"), "empty token"));
        }
    }
    for intent in &pack.intents {
        if intent.phrase.trim().len() < 2 {
            report.errors.push(issue("intents.phrase", "phrase is too short"));
        }
        if !known_intent(&intent.intent) {
            report.errors.push(issue("intents.intent", format!("unknown intent {}", intent.intent)));
        }
    }
    report
}

const SET_KEYS: &[(&str, WordKey)] = &[
    ("talk.fillers", WordKey::Fillers),
    ("talk.action_keep", WordKey::ActionKeep),
    ("talk.conjunctions", WordKey::Conjunctions),
    ("talk.particles", WordKey::Particles),
    ("talk.affirm", WordKey::Affirm),
    ("nouns.light_nouns", WordKey::LightNouns),
    ("nouns.light_singular", WordKey::LightSingular),
    ("nouns.light_plural", WordKey::LightPlural),
    ("nouns.cover_nouns", WordKey::CoverNouns),
    ("nouns.curtain_nouns", WordKey::CurtainNouns),
    ("nouns.fan_nouns", WordKey::FanNouns),
    ("nouns.climate_nouns", WordKey::ClimateNouns),
    ("nouns.media_nouns", WordKey::MediaNouns),
    ("nouns.lock_nouns", WordKey::LockNouns),
    ("nouns.door_nouns", WordKey::DoorNouns),
    ("nouns.garage_words", WordKey::GarageWords),
    ("nouns.garage_cover", WordKey::GarageCover),
    ("nouns.timer_nouns", WordKey::TimerNouns),
    ("nouns.list_nouns", WordKey::ListNouns),
    ("nouns.calendar_nouns", WordKey::CalendarNouns),
    ("nouns.vacuum_nouns", WordKey::VacuumNouns),
    ("nouns.scene_nouns", WordKey::SceneNouns),
    ("nouns.script_words", WordKey::ScriptWords),
    ("nouns.switch_plural", WordKey::SwitchPlural),
    ("nouns.device_side", WordKey::DeviceSide),
    ("nouns.named_device", WordKey::NamedDevice),
    ("cues.on_words", WordKey::OnWords),
    ("cues.off_words", WordKey::OffWords),
    ("cues.extra_device_nouns", WordKey::ExtraDeviceNouns),
    ("cues.power_words", WordKey::PowerWords),
    ("cues.command_hedges", WordKey::CommandHedges),
    ("cues.skip_light", WordKey::SkipLight),
    ("cues.laundry_area", WordKey::LaundryArea),
    ("cues.laundry_machines", WordKey::LaundryMachines),
    ("cues.kitchen", WordKey::Kitchen),
    ("cues.open_words", WordKey::OpenWords),
    ("cues.close_words", WordKey::CloseWords),
    ("cues.roll_close", WordKey::RollClose),
    ("cues.unlock_follow", WordKey::UnlockFollow),
    ("cues.cover_open_follow", WordKey::CoverOpenFollow),
    ("cues.garage_lock_block", WordKey::GarageLockBlock),
    ("cues.scene_named", WordKey::SceneNamed),
    ("cues.temp_query", WordKey::TempQuery),
    ("cues.timer_query", WordKey::TimerQuery),
    ("cues.brightness", WordKey::Brightness),
    ("cues.start_words", WordKey::StartWords),
    ("cues.replay_on_off", WordKey::ReplayOnOff),
    ("cues.replay_off", WordKey::ReplayOff),
    ("cues.sensor_words", WordKey::SensorWords),
    ("cues.lock_verbs", WordKey::LockVerbs),
    ("cues.entry_words", WordKey::EntryWords),
    ("cues.oven", WordKey::Oven),
    ("cues.laundry_timer", WordKey::LaundryTimer),
    ("cues.illuminate", WordKey::Illuminate),
    ("cues.list_down", WordKey::ListDown),
    ("cues.chores", WordKey::Chores),
    ("cues.weak_scene", WordKey::WeakScene),
    ("cues.timer_cancel", WordKey::TimerCancel),
    ("cues.timer_pause", WordKey::TimerPause),
    ("cues.timer_add", WordKey::TimerAdd),
    ("cues.list_complete", WordKey::ListComplete),
    ("cues.playback_resume", WordKey::PlaybackResume),
    ("cues.calendar_query", WordKey::CalendarQuery),
    ("cues.calendar_create", WordKey::CalendarCreate),
    ("cues.calendar_today", WordKey::CalendarToday),
    ("cues.calendar_tomorrow", WordKey::CalendarTomorrow),
    ("cues.calendar_when", WordKey::CalendarWhen),
    ("cues.calendar_delete", WordKey::CalendarDelete),
    ("cues.calendar_move", WordKey::CalendarMove),
    ("cues.vacuum_start", WordKey::VacuumStart),
    ("cues.hours", WordKey::Hours),
    ("cues.minutes", WordKey::Minutes),
    ("cues.seconds", WordKey::Seconds),
    ("cues.list_skip", WordKey::ListSkip),
    ("cues.shopping_names", WordKey::ShoppingNames),
    ("cues.status_words", WordKey::StatusWords),
    ("cues.window_words", WordKey::WindowWords),
    ("cues.open_close", WordKey::OpenClose),
    ("cues.laundry_hint", WordKey::LaundryHint),
    ("cues.bare_switch", WordKey::BareSwitch),
    ("cues.outlet_words", WordKey::OutletWords),
    ("cues.tv_words", WordKey::TvWords),
    ("cues.climate_cool", WordKey::ClimateCool),
    ("cues.climate_heat", WordKey::ClimateHeat),
    ("cues.role_light", WordKey::RoleLight),
    ("cues.role_climate", WordKey::RoleClimate),
    ("cues.role_media", WordKey::RoleMedia),
    ("cues.role_fan", WordKey::RoleFan),
    ("cues.generic", WordKey::Generic),
    ("cues.room_level", WordKey::RoomLevel),
    ("cues.article_one", WordKey::ArticleOne),
];

pub fn lexicon_set_paths() -> Vec<&'static str> {
    SET_KEYS.iter().map(|(name, _)| *name).collect()
}

pub fn is_lexicon_path(path: &str) -> bool {
    set_field(path).is_some()
}

pub(super) fn set_field(path: &str) -> Option<WordKey> {
    SET_KEYS.iter().copied().find(|(name, _)| *name == path).map(|(_, key)| key)
}

fn issue(path: &str, message: impl Into<String>) -> ValidationIssue {
    ValidationIssue { path: path.to_string(), message: message.into() }
}

#[cfg(test)]
mod tests {
    use super::super::catalog::WordKey;
    use super::set_field;

    #[test]
    fn overlay_set_paths_cover_nouns_and_cues_not_verbs() {
        assert_eq!(set_field("nouns.curtain_nouns"), Some(WordKey::CurtainNouns));
        assert_eq!(set_field("nouns.media_nouns"), Some(WordKey::MediaNouns));
        assert_eq!(set_field("cues.open_words"), Some(WordKey::OpenWords));
        assert_eq!(set_field("talk.fillers"), Some(WordKey::Fillers));
        assert!(set_field("verbs.an").is_none());
        assert!(set_field("cues.synonym_pairs").is_none());
        assert!(set_field("nope.words").is_none());
    }
}
