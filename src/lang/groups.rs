use super::speech::Speech;
use super::verbs::VerbKind;
use super::LangId;

/// How number words combine. A new grammar is a new variant plus pack lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberStyle {
    GermanUnd,
    EnglishTens,
}

pub struct GroupClarify {
    pub trigger: &'static [&'static str],
    pub pairs: &'static [[&'static str; 2]],
    pub triples: &'static [[&'static str; 3]],
}

impl GroupClarify {
    pub fn matches(&self, raw: &[String]) -> bool {
        if self.trigger.is_empty() || !raw.iter().any(|t| self.trigger.contains(&t.as_str())) {
            return false;
        }
        raw.windows(2).any(|w| self.pairs.iter().any(|p| w[0] == p[0] && w[1] == p[1]))
            || raw.windows(3).any(|w| self.triples.iter().any(|p| w[0] == p[0] && w[1] == p[1] && w[2] == p[2]))
    }
}

pub struct Talk {
    pub fillers: &'static [&'static str],
    pub action_keep: &'static [&'static str],
    pub conjunctions: &'static [&'static str],
    pub particles: &'static [&'static str],
    pub affirm: &'static [&'static str],
    pub or_words: &'static [&'static str],
    pub all_words: &'static [&'static str],
    pub query_hint: &'static [&'static str],
    pub question_starts: &'static [&'static str],
    pub question_words: &'static [&'static str],
    pub correction: &'static [&'static str],
    pub correction_phrases: &'static [&'static str],
    pub clarify_pick: &'static [&'static str],
}

pub struct Nouns {
    pub light_nouns: &'static [&'static str],
    pub light_singular: &'static [&'static str],
    pub light_plural: &'static [&'static str],
    pub cover_nouns: &'static [&'static str],
    pub curtain_nouns: &'static [&'static str],
    pub fan_nouns: &'static [&'static str],
    pub climate_nouns: &'static [&'static str],
    pub media_nouns: &'static [&'static str],
    pub lock_nouns: &'static [&'static str],
    pub door_nouns: &'static [&'static str],
    pub garage_words: &'static [&'static str],
    pub garage_cover: &'static [&'static str],
    pub timer_nouns: &'static [&'static str],
    pub list_nouns: &'static [&'static str],
    pub vacuum_nouns: &'static [&'static str],
    pub scene_nouns: &'static [&'static str],
    pub script_words: &'static [&'static str],
    pub switch_plural: &'static [&'static str],
    pub device_side: &'static [&'static str],
    pub named_device: &'static [&'static str],
}

pub struct Fixtures {
    pub island: &'static [&'static str],
    pub ceiling: &'static [&'static str],
    pub lamp_fixture: &'static [&'static str],
    pub pendant: &'static [&'static str],
    pub bedside: &'static [&'static str],
    pub left: &'static [&'static str],
    pub right: &'static [&'static str],
    pub sides: &'static [&'static str],
    pub fixture_aliases: &'static [(&'static str, &'static [&'static str])],
    pub group_clarify: Option<GroupClarify>,
    pub singular_lamp: &'static [&'static str],
    pub singular_lamp_block: &'static [&'static str],
}

pub struct Cues {
    pub power_words: &'static [&'static str],
    pub command_hedges: &'static [&'static str],
    pub skip_light: &'static [&'static str],
    pub laundry_area: &'static [&'static str],
    pub laundry_machines: &'static [&'static str],
    pub kitchen: &'static [&'static str],
    pub open_words: &'static [&'static str],
    pub close_words: &'static [&'static str],
    pub roll_close: &'static [&'static str],
    pub unlock_follow: &'static [&'static str],
    pub cover_open_follow: &'static [&'static str],
    pub garage_lock_block: &'static [&'static str],
    pub on_words: &'static [&'static str],
    pub off_words: &'static [&'static str],
    pub scene_named: &'static [&'static str],
    pub temp_query: &'static [&'static str],
    pub timer_query: &'static [&'static str],
    pub brightness: &'static [&'static str],
    pub start_words: &'static [&'static str],
    pub replay_on_off: &'static [&'static str],
    pub replay_off: &'static [&'static str],
    pub sensor_words: &'static [&'static str],
    pub lock_verbs: &'static [&'static str],
    pub entry_words: &'static [&'static str],
    pub oven: &'static [&'static str],
    pub laundry_timer: &'static [&'static str],
    pub illuminate: &'static [&'static str],
    pub list_down: &'static [&'static str],
    pub chores: &'static [&'static str],
    pub weak_scene: &'static [&'static str],
    pub timer_cancel: &'static [&'static str],
    pub timer_pause: &'static [&'static str],
    pub timer_add: &'static [&'static str],
    pub list_complete: &'static [&'static str],
    pub playback_resume: &'static [&'static str],
    pub vacuum_start: &'static [&'static str],
    pub hours: &'static [&'static str],
    pub minutes: &'static [&'static str],
    pub seconds: &'static [&'static str],
    pub list_skip: &'static [&'static str],
    pub shopping_names: &'static [&'static str],
    pub status_words: &'static [&'static str],
    pub window_words: &'static [&'static str],
    pub open_close: &'static [&'static str],
    pub laundry_hint: &'static [&'static str],
    pub bare_switch: &'static [&'static str],
    pub outlet_words: &'static [&'static str],
    pub tv_words: &'static [&'static str],
    pub climate_cool: &'static [&'static str],
    pub climate_heat: &'static [&'static str],
    pub role_light: &'static [&'static str],
    pub role_climate: &'static [&'static str],
    pub role_media: &'static [&'static str],
    pub role_fan: &'static [&'static str],
    pub generic: &'static [&'static str],
    pub room_level: &'static [&'static str],
    pub extra_device_nouns: &'static [&'static str],
    pub synonym_pairs: &'static [(&'static str, &'static str)],
    pub scene_synonyms: &'static [(&'static str, &'static str)],
    pub article_one: &'static [&'static str],
    pub strip_pairs: &'static [(&'static str, &'static str)],
    pub keep_after: &'static [(&'static [&'static str], &'static str)],
}

pub struct Maps {
    pub domain_map: &'static [(&'static str, &'static str)],
    pub colors: &'static [(&'static str, &'static str)],
    pub numbers: &'static [(&'static str, i32)],
    pub number_style: NumberStyle,
    pub room_index_nouns: &'static [&'static str],
}

pub struct Chat {
    pub greet: &'static [&'static str],
    pub thanks: &'static [&'static str],
    pub feeling: &'static [&'static str],
    pub identity: &'static [&'static str],
    pub tell: &'static [&'static str],
    pub yarn: &'static [&'static str],
    pub world: &'static [&'static str],
    pub advice: &'static [&'static str],
    pub open: &'static [&'static str],
}

/// Static word lists for one language. Add a new file (`fr.rs`) and register it on `LangId`.
pub struct LanguagePack {
    pub id: LangId,
    pub verbs: &'static [(&'static str, VerbKind)],
    pub talk: Talk,
    pub nouns: Nouns,
    pub fixtures: Fixtures,
    pub cues: Cues,
    pub maps: Maps,
    pub chat: Chat,
    pub speech: Speech,
}
