mod merge;

use super::groups::{GroupClarify, LanguagePack as Pack, NumberStyle};
use super::morphology::Morphology;
use super::speech::Speech;
use super::verbs::VerbKind;
use super::LangId;
use crate::types::CustomSentence;
use std::collections::{HashMap, HashSet};

pub struct Catalog {
    packs: Vec<&'static Pack>,
    pub langs: Vec<LangId>,
    pub(super) verbs: HashMap<&'static str, VerbKind>,
    pub domain_map: HashMap<&'static str, &'static str>,
    pub(super) colors: HashMap<&'static str, &'static str>,
    pub(super) numbers: HashMap<&'static str, i32>,
    pub number_styles: Vec<NumberStyle>,
    fixture_aliases: HashMap<&'static str, &'static [&'static str]>,
    group_clarify: Vec<GroupClarify>,
    pub strip_pairs: Vec<(&'static str, &'static str)>,
    pub keep_after: Vec<(&'static [&'static str], &'static str)>,
    pub synonym_pairs: Vec<(&'static str, &'static str)>,
    pub scene_synonyms: Vec<(&'static str, &'static str)>,
    pub speech: Vec<&'static Speech>,
    pub fillers: HashSet<&'static str>,
    pub action_keep: HashSet<&'static str>,
    pub conjunctions: HashSet<&'static str>,
    pub particles: HashSet<&'static str>,
    pub affirm: HashSet<&'static str>,
    pub or_words: HashSet<&'static str>,
    pub all_words: HashSet<&'static str>,
    pub query_hint: HashSet<&'static str>,
    pub question_starts: HashSet<&'static str>,
    pub question_words: HashSet<&'static str>,
    pub correction: HashSet<&'static str>,
    pub correction_phrases: HashSet<&'static str>,
    pub clarify_pick: HashSet<&'static str>,
    pub light_nouns: HashSet<&'static str>,
    pub light_singular: HashSet<&'static str>,
    pub light_plural: HashSet<&'static str>,
    pub cover_nouns: HashSet<&'static str>,
    pub curtain_nouns: HashSet<&'static str>,
    pub fan_nouns: HashSet<&'static str>,
    pub climate_nouns: HashSet<&'static str>,
    pub media_nouns: HashSet<&'static str>,
    pub lock_nouns: HashSet<&'static str>,
    pub door_nouns: HashSet<&'static str>,
    pub garage_words: HashSet<&'static str>,
    pub garage_cover: HashSet<&'static str>,
    pub timer_nouns: HashSet<&'static str>,
    pub list_nouns: HashSet<&'static str>,
    pub vacuum_nouns: HashSet<&'static str>,
    pub scene_nouns: HashSet<&'static str>,
    pub script_words: HashSet<&'static str>,
    pub switch_plural: HashSet<&'static str>,
    pub device_side: HashSet<&'static str>,
    pub named_device: HashSet<&'static str>,
    pub island: HashSet<&'static str>,
    pub ceiling: HashSet<&'static str>,
    pub lamp_fixture: HashSet<&'static str>,
    pub pendant: HashSet<&'static str>,
    pub bedside: HashSet<&'static str>,
    pub left: HashSet<&'static str>,
    pub right: HashSet<&'static str>,
    pub sides: HashSet<&'static str>,
    pub singular_lamp: HashSet<&'static str>,
    pub singular_lamp_block: HashSet<&'static str>,
    pub power_words: HashSet<&'static str>,
    pub command_hedges: HashSet<&'static str>,
    pub skip_light: HashSet<&'static str>,
    pub laundry_area: HashSet<&'static str>,
    pub laundry_machines: HashSet<&'static str>,
    pub kitchen: HashSet<&'static str>,
    pub open_words: HashSet<&'static str>,
    pub close_words: HashSet<&'static str>,
    pub roll_close: HashSet<&'static str>,
    pub unlock_follow: HashSet<&'static str>,
    pub cover_open_follow: HashSet<&'static str>,
    pub garage_lock_block: HashSet<&'static str>,
    pub on_words: HashSet<&'static str>,
    pub off_words: HashSet<&'static str>,
    pub scene_named: HashSet<&'static str>,
    pub temp_query: HashSet<&'static str>,
    pub timer_query: HashSet<&'static str>,
    pub brightness: HashSet<&'static str>,
    pub start_words: HashSet<&'static str>,
    pub replay_on_off: HashSet<&'static str>,
    pub replay_off: HashSet<&'static str>,
    pub sensor_words: HashSet<&'static str>,
    pub lock_verbs: HashSet<&'static str>,
    pub entry_words: HashSet<&'static str>,
    pub oven: HashSet<&'static str>,
    pub laundry_timer: HashSet<&'static str>,
    pub illuminate: HashSet<&'static str>,
    pub list_down: HashSet<&'static str>,
    pub chores: HashSet<&'static str>,
    pub weak_scene: HashSet<&'static str>,
    pub timer_cancel: HashSet<&'static str>,
    pub timer_pause: HashSet<&'static str>,
    pub timer_add: HashSet<&'static str>,
    pub list_complete: HashSet<&'static str>,
    pub playback_resume: HashSet<&'static str>,
    pub vacuum_start: HashSet<&'static str>,
    pub hours: HashSet<&'static str>,
    pub minutes: HashSet<&'static str>,
    pub seconds: HashSet<&'static str>,
    pub list_skip: HashSet<&'static str>,
    pub shopping_names: HashSet<&'static str>,
    pub status_words: HashSet<&'static str>,
    pub window_words: HashSet<&'static str>,
    pub open_close: HashSet<&'static str>,
    pub laundry_hint: HashSet<&'static str>,
    pub bare_switch: HashSet<&'static str>,
    pub outlet_words: HashSet<&'static str>,
    pub tv_words: HashSet<&'static str>,
    pub climate_cool: HashSet<&'static str>,
    pub climate_heat: HashSet<&'static str>,
    pub role_light: HashSet<&'static str>,
    pub role_climate: HashSet<&'static str>,
    pub role_media: HashSet<&'static str>,
    pub role_fan: HashSet<&'static str>,
    pub generic: HashSet<&'static str>,
    pub room_level: HashSet<&'static str>,
    pub extra_device_nouns: HashSet<&'static str>,
    pub article_one: HashSet<&'static str>,
    pub room_index_nouns: HashSet<&'static str>,
    pub chat_greet: HashSet<&'static str>,
    pub chat_thanks: HashSet<&'static str>,
    pub chat_feeling: HashSet<&'static str>,
    pub chat_identity: HashSet<&'static str>,
    pub chat_tell: HashSet<&'static str>,
    pub chat_yarn: HashSet<&'static str>,
    pub chat_world: HashSet<&'static str>,
    pub chat_advice: HashSet<&'static str>,
    pub chat_open: HashSet<&'static str>,
    pub chat_news: HashSet<&'static str>,
    pub chat_news_dismiss: HashSet<&'static str>,
    pub news_intro: &'static str,
    pub news_nudge: &'static str,
    pub news_done: &'static str,
    pub morphology: Morphology,
    pub pack_intents: Vec<CustomSentence>,
}

impl Catalog {
    pub fn verb(&self, t: &str) -> Option<VerbKind> {
        self.verbs.get(t).copied()
    }

    pub fn is_filler(&self, t: &str) -> bool {
        self.fillers.contains(t)
    }

    pub fn is_action_keep(&self, t: &str) -> bool {
        self.action_keep.contains(t)
    }

    pub fn is_conj(&self, t: &str) -> bool {
        self.conjunctions.contains(t)
    }

    pub fn is_particle(&self, t: &str) -> bool {
        self.particles.contains(t)
    }

    pub fn is_affirm(&self, t: &str) -> bool {
        self.affirm.contains(t)
    }

    pub fn is_or(&self, t: &str) -> bool {
        self.or_words.contains(t)
    }

    pub fn is_all(&self, t: &str) -> bool {
        self.all_words.contains(t)
    }

    pub fn is_except(&self, t: &str) -> bool {
        self.pack_has(t, |pack| pack.talk.except_words)
    }

    pub fn is_query_hint(&self, t: &str) -> bool {
        self.query_hint.contains(t)
    }

    pub fn any(&self, tokens: &[String], set: &HashSet<&'static str>) -> bool {
        tokens.iter().any(|t| set.contains(t.as_str()))
    }

    /// Query language packs directly. New word lists belong on `LanguagePack`; the HashSets are a cache.
    pub fn pack_has(&self, token: &str, pick: fn(&Pack) -> &[&str]) -> bool {
        self.packs.iter().any(|pack| pick(pack).contains(&token))
    }

    pub fn pack_any(&self, tokens: &[String], pick: fn(&Pack) -> &[&str]) -> bool {
        tokens.iter().any(|token| self.pack_has(token, pick))
    }

    pub fn household(&self) -> Option<&'static crate::lang::groups::Household> {
        self.packs.first().map(|pack| &pack.household)
    }

    pub fn household_hit(&self, blob: &str, pick: fn(&Pack) -> &[&str]) -> bool {
        self.packs.iter().any(|pack| pick(pack).iter().any(|phrase| !phrase.is_empty() && blob.contains(phrase)))
    }

    pub fn household_prefix(&self, blob: &str, pick: fn(&Pack) -> &[&str]) -> Option<String> {
        for pack in &self.packs {
            for prefix in pick(pack) {
                if let Some(rest) = blob.strip_prefix(prefix) {
                    let alias = rest.trim().trim_matches(|ch: char| ch == '.' || ch == '!' || ch == '?');
                    if !alias.is_empty() {
                        return Some(alias.to_string());
                    }
                }
            }
        }
        None
    }

    pub fn color(&self, t: &str) -> Option<&'static str> {
        self.colors.get(t).copied().or_else(|| {
            self.morphology
                .effective_color_suffixes()
                .iter()
                .find_map(|suffix| t.strip_suffix(suffix).and_then(|stem| self.colors.get(stem)).copied())
        })
    }

    pub fn color_spoken(&self, canonical: &str) -> String {
        self.packs
            .iter()
            .find_map(|pack| {
                pack.maps
                    .colors
                    .iter()
                    .filter(|(_, color)| *color == canonical)
                    .min_by_key(|(word, _)| word.len())
                    .map(|(word, _)| (*word).to_string())
            })
            .unwrap_or_else(|| canonical.to_string())
    }

    pub fn number(&self, t: &str) -> Option<i32> {
        self.numbers.get(t).copied()
    }

    pub fn number_word(&self, n: i32) -> Option<&'static str> {
        self.numbers.iter().find(|(_, val)| **val == n).map(|(word, _)| *word)
    }

    pub fn fixture_alias(&self, t: &str) -> &[&str] {
        self.fixture_aliases.get(t).copied().unwrap_or(&[])
    }

    pub fn wants_group_clarify(&self, raw: &[String]) -> bool {
        self.group_clarify.iter().any(|g| g.matches(raw))
    }

    pub fn wants_singular_lamp(&self, tokens: &[String]) -> bool {
        self.any(tokens, &self.singular_lamp) && !self.any(tokens, &self.singular_lamp_block)
    }

    pub fn is_question_start(&self, t: &str) -> bool {
        self.question_starts.contains(t)
    }

    pub fn is_question_word(&self, t: &str) -> bool {
        self.question_words.contains(t)
    }

    pub fn knows_surface(&self, token: &str) -> bool {
        self.verbs.contains_key(token)
            || self.domain_map.contains_key(token)
            || self.colors.contains_key(token)
            || self.numbers.contains_key(token)
            || self.light_nouns.contains(token)
            || self.cover_nouns.contains(token)
            || self.climate_nouns.contains(token)
            || self.media_nouns.contains(token)
            || self.lock_nouns.contains(token)
            || self.timer_nouns.contains(token)
            || self.list_nouns.contains(token)
            || self.fan_nouns.contains(token)
            || self.vacuum_nouns.contains(token)
            || self.scene_nouns.contains(token)
            || self.on_words.contains(token)
            || self.off_words.contains(token)
            || self.open_words.contains(token)
            || self.close_words.contains(token)
            || self.generic.contains(token)
            || self.all_words.contains(token)
            || self.conjunctions.contains(token)
            || self.question_words.contains(token)
            || self.is_except(token)
            || self.synonym_pairs.iter().any(|(alias, _)| *alias == token)
    }

    pub fn codes(&self) -> Vec<&'static str> {
        self.langs.iter().map(|l| l.code()).collect()
    }

    pub fn has_german_und(&self) -> bool {
        self.number_styles.contains(&NumberStyle::GermanUnd)
    }

    pub fn has_english_tens(&self) -> bool {
        self.number_styles.contains(&NumberStyle::EnglishTens)
    }

    pub fn speech(&self) -> &'static Speech {
        self.speech.first().copied().unwrap_or(&super::de::PACK.speech)
    }

    pub fn synonyms<'a>(&'a self, token: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.synonym_pairs.iter().filter_map(move |(a, b)| {
            if *a == token {
                Some(*b)
            } else if *b == token {
                Some(*a)
            } else {
                None
            }
        })
    }

    pub fn scene_token(&self, token: &str) -> String {
        self.scene_synonyms.iter().find(|(from, _)| *from == token).map(|(_, to)| (*to).to_string()).unwrap_or_else(|| token.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::lang::catalog;

    #[test]
    fn pack_query_matches_flattened_cache() {
        let _bind = crate::lang::bind(&["de".into()]);
        let cat = catalog();
        let licht = vec!["licht".into()];
        assert!(cat.pack_any(&licht, |p| p.nouns.light_nouns));
        assert_eq!(cat.pack_any(&licht, |p| p.nouns.light_nouns), cat.any(&licht, &cat.light_nouns));
        assert!(!cat.pack_has("licht", |p| p.nouns.cover_nouns));
    }

    #[test]
    fn color_accepts_german_endings() {
        let _bind = crate::lang::bind(&["de".into()]);
        let cat = catalog();
        assert_eq!(cat.color("rot"), Some("red"));
        assert_eq!(cat.color("rote"), Some("red"));
        assert_eq!(cat.color("rotes"), Some("red"));
        assert_eq!(cat.color_spoken("red"), "rot");
    }
}
