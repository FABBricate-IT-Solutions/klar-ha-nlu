//! Spoken templates for one language.
//! The engine interpolates `{target}`, `{n}`, `{names}`, `{noun}`, `{loc}`, `{item}`,
//! `{state}`, `{unit}`, `{task}`, `{kind}`, `{title}`, `{artist}`.

use crate::types::UnitSystem;

#[derive(Clone, Copy)]
pub struct Speech {
    pub unknown: &'static str,
    pub need_on: &'static str,
    pub need_off: &'static str,
    pub need_which: &'static str,
    pub correction: &'static str,
    pub clarify: &'static str,
    pub clarify_or: &'static str,
    pub and_join: &'static str,
    pub group_on: &'static str,
    pub group_off: &'static str,
    pub turn_on: &'static str,
    pub turn_on_scene: &'static str,
    pub turn_off: &'static str,
    pub toggle: &'static str,
    pub cover_open: &'static str,
    pub cover_close: &'static str,
    pub lock: &'static str,
    pub unlock: &'static str,
    pub light_set: &'static str,
    pub light_step_up: &'static str,
    pub light_step_down: &'static str,
    pub set_position: &'static str,
    pub light_color: &'static str,
    pub climate_set: &'static str,
    pub climate_at: &'static str,
    pub temp_reading: &'static str,
    pub heat_noun: &'static str,
    pub cool_noun: &'static str,
    pub get_temp: &'static str,
    pub get_state: &'static str,
    pub entity_state: &'static str,
    pub media_pause: &'static str,
    pub media_play: &'static str,
    pub media_next: &'static str,
    pub media_previous: &'static str,
    pub media_mute: &'static str,
    pub media_unmute: &'static str,
    pub media_volume: &'static str,
    pub media_volume_set: &'static str,
    pub media_volume_up: &'static str,
    pub media_volume_down: &'static str,
    pub media_volume_query: &'static str,
    pub media_playing: &'static str,
    pub media_playing_by: &'static str,
    pub media_search: &'static str,
    pub media_transfer: &'static str,
    pub media_favorite: &'static str,
    pub queue_empty: &'static str,
    pub fan_set: &'static str,
    pub vacuum_start: &'static str,
    pub vacuum_dock: &'static str,
    pub vacuum_default: &'static str,
    pub timer_start: &'static str,
    pub timer_cancel: &'static str,
    pub timer_pause: &'static str,
    pub timer_how_long: &'static str,
    pub list_add: &'static str,
    pub list_remove: &'static str,
    pub list_complete: &'static str,
    pub calendar_list: &'static str,
    pub calendar_empty: &'static str,
    pub calendar_none: &'static str,
    pub calendar_created: &'static str,
    pub calendar_need_title: &'static str,
    pub calendar_need_when: &'static str,
    pub calendar_readonly: &'static str,
    pub calendar_deleted: &'static str,
    pub calendar_moved: &'static str,
    pub calendar_which: &'static str,
    pub calendar_no_uid: &'static str,
    pub no_music_player: &'static str,
    pub done: &'static str,
    pub light_suffix: &'static str,
    pub light_noun: &'static str,
    pub lights_on: &'static str,
    pub lights_off: &'static str,
    pub area_light: &'static str,
    pub area_lights: &'static str,
    pub loc_in: &'static str,
    pub loc_in_der: &'static str,
    pub loc_home: &'static str,
    pub or_home: &'static str,
    pub empty_place: &'static str,
    pub unit_c: &'static str,
    pub unit_f: &'static str,
    pub reminder: &'static str,
    pub presence_yes: &'static str,
    pub presence_no: &'static str,
    pub list_none_found: &'static str,
    pub list_none_state: &'static str,
    pub list_state: &'static str,
    pub list_all: &'static str,
    pub kind_window: &'static str,
    pub kind_door: &'static str,
    pub kind_motion: &'static str,
    pub kind_smoke: &'static str,
    pub state_open: &'static str,
    pub state_closed: &'static str,
    pub states: &'static [(&'static str, &'static str)],
    pub room_names: &'static [(&'static str, &'static str)],
    pub loc_der_rooms: &'static [&'static str],
    pub personality: &'static [(&'static str, &'static [&'static str])],
    pub confirm: &'static str,
}

impl Speech {
    pub fn personality_prefixes(self, name: &str) -> &'static [&'static str] {
        self.personality.iter().find(|(key, _)| *key == name).map(|(_, prefixes)| *prefixes).unwrap_or(&[])
    }

    pub fn personality_prefix(self, name: &str) -> &'static str {
        self.personality_prefixes(name).first().copied().unwrap_or("")
    }

    pub fn room_name(self, folded: &str) -> Option<&'static str> {
        self.room_names.iter().find(|(key, _)| *key == folded).map(|(_, name)| *name)
    }

    pub fn state_word(self, raw: &str) -> String {
        self.states.iter().find(|(key, _)| *key == raw).map(|(_, spoken)| (*spoken).to_string()).unwrap_or_else(|| raw.to_string())
    }

    pub fn unit_word(self, system: UnitSystem) -> &'static str {
        match system {
            UnitSystem::Metric => self.unit_c,
            UnitSystem::Imperial => self.unit_f,
        }
    }

    pub fn entity_line(self, target: &str, ha_state: &str) -> String {
        if target.trim().is_empty() && ha_state.trim().is_empty() {
            return String::new();
        }
        match ha_state {
            "on" => self.turn_on.replace("{target}", target),
            "off" => self.turn_off.replace("{target}", target),
            "" => String::new(),
            _ => self.entity_state.replace("{target}", target).replace("{state}", &self.state_word(ha_state)),
        }
    }

    /// Reply for a to-do list intent, with `{item}` filled in or dropped when nothing was heard.
    pub fn list_reply(self, intent: &str, item: Option<&str>) -> Option<String> {
        let template = match intent {
            "HassListAddItem" | "HassShoppingListAddItem" => self.list_add,
            "HassListCompleteItem" | "HassShoppingListCompleteItem" => self.list_complete,
            "HassListRemoveItem" => self.list_remove,
            _ => return None,
        };
        let line = match item {
            Some(item) => template.replace("{item}", item),
            None => template.replace("{item} ", "").replace("{item}", ""),
        };
        let line = line.trim();
        let mut chars = line.chars();
        let first = chars.next()?;
        Some(first.to_uppercase().chain(chars).collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::lang::LangId;

    fn speech(code: &str) -> crate::lang::Speech {
        LangId::from_code(code).unwrap_or_else(|| panic!("missing pack {code}")).pack().speech
    }

    #[test]
    fn generated_slots_are_not_need_on() {
        for code in ["es", "ja", "de-AT", "en-GB", "ar", "fa", "hi"] {
            let s = speech(code);
            assert_ne!(s.timer_start, s.need_on, "{code} timer_start");
            assert_ne!(s.media_search, s.need_on, "{code} media_search");
            assert_ne!(s.vacuum_default, s.need_on, "{code} vacuum_default");
            assert_ne!(s.timer_cancel, s.need_off, "{code} timer_cancel");
            assert_ne!(s.timer_pause, s.need_off, "{code} timer_pause");
            assert!(!s.timer_how_long.is_empty(), "{code} timer_how_long");
            assert_ne!(s.heat_noun, s.cool_noun, "{code} heat/cool");
            assert_ne!(s.loc_home, s.need_on, "{code} loc_home");
            assert!(!s.empty_place.is_empty(), "{code} empty_place");
            assert!(!s.entity_state.is_empty(), "{code} entity_state");
            let jarvis = s.personality_prefixes("jarvis");
            assert!(!jarvis.iter().any(|prefix| prefix.contains("Of course, sir")), "{code} jarvis should not be English");
        }
    }

    #[test]
    fn cover_and_lock_words_are_not_on_off() {
        for code in ["en", "de"] {
            let s = speech(code);
            for word in [s.cover_open, s.cover_close, s.lock, s.unlock] {
                assert!(word.contains("{target}"), "{code} missing target: {word}");
                assert_ne!(word, s.turn_on, "{code}");
                assert_ne!(word, s.turn_off, "{code}");
            }
        }
    }
}
