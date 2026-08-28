/// Spoken templates for one language. The engine interpolates `{target}`, `{n}`, `{names}`, `{noun}`, `{loc}`.
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
    pub light_set: &'static str,
    pub light_color: &'static str,
    pub climate_set: &'static str,
    pub heat_noun: &'static str,
    pub cool_noun: &'static str,
    pub get_temp: &'static str,
    pub get_state: &'static str,
    pub media_pause: &'static str,
    pub media_play: &'static str,
    pub media_next: &'static str,
    pub media_previous: &'static str,
    pub media_mute: &'static str,
    pub media_unmute: &'static str,
    pub media_volume: &'static str,
    pub media_search: &'static str,
    pub media_transfer: &'static str,
    pub media_favorite: &'static str,
    pub fan_set: &'static str,
    pub vacuum_start: &'static str,
    pub vacuum_dock: &'static str,
    pub vacuum_default: &'static str,
    pub timer_start: &'static str,
    pub timer_cancel: &'static str,
    pub timer_pause: &'static str,
    pub list_add: &'static str,
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
    pub area_light: &'static str,
    pub loc_in: &'static str,
    pub loc_in_der: &'static str,
    pub loc_home: &'static str,
    pub or_home: &'static str,
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
}
