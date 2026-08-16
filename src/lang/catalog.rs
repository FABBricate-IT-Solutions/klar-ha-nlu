use super::groups::{GroupClarify, LanguagePack as Pack, NumberStyle};
use super::speech::Speech;
use super::LangId;
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
}

use super::verbs::VerbKind;

macro_rules! extend_sets {
    ($dst:expr, $src:expr; $($field:ident),+ $(,)?) => {
        $($dst.$field.extend($src.$field.iter().copied());)+
    };
}

impl Catalog {
    pub(super) fn merge(packs: &[&'static Pack]) -> Self {
        let mut c = Self::empty();
        c.packs = packs.to_vec();
        for p in packs {
            c.langs.push(p.id);
            c.speech.push(&p.speech);
            for &(w, k) in p.verbs {
                c.verbs.insert(w, k);
            }
            for &(w, d) in p.maps.domain_map {
                c.domain_map.insert(w, d);
            }
            for &(w, color) in p.maps.colors {
                c.colors.insert(w, color);
            }
            for &(w, n) in p.maps.numbers {
                c.numbers.insert(w, n);
            }
            c.number_styles.push(p.maps.number_style);
            for &(w, aliases) in p.fixtures.fixture_aliases {
                c.fixture_aliases.insert(w, aliases);
            }
            if let Some(g) = &p.fixtures.group_clarify {
                c.group_clarify.push(GroupClarify { trigger: g.trigger, pairs: g.pairs, triples: g.triples });
            }
            c.strip_pairs.extend(p.cues.strip_pairs.iter().copied());
            c.keep_after.extend(p.cues.keep_after.iter().copied());
            c.synonym_pairs.extend(p.cues.synonym_pairs.iter().copied());
            c.scene_synonyms.extend(p.cues.scene_synonyms.iter().copied());
            extend_sets!(c, p.talk; fillers, action_keep, conjunctions, particles, affirm, or_words, all_words, query_hint, question_starts, question_words, correction, correction_phrases, clarify_pick);
            extend_sets!(c, p.nouns; light_nouns, light_singular, light_plural, cover_nouns, curtain_nouns, fan_nouns, climate_nouns, media_nouns, lock_nouns, door_nouns, garage_words, garage_cover, timer_nouns, list_nouns, vacuum_nouns, scene_nouns, script_words, switch_plural, device_side, named_device);
            extend_sets!(c, p.fixtures; island, ceiling, lamp_fixture, pendant, bedside, left, right, sides, singular_lamp, singular_lamp_block);
            extend_sets!(c, p.cues; power_words, command_hedges, skip_light, laundry_area, laundry_machines, kitchen, open_words, close_words, roll_close, unlock_follow, cover_open_follow, garage_lock_block, on_words, off_words, scene_named, temp_query, timer_query, brightness, start_words, replay_on_off, replay_off, sensor_words, lock_verbs, entry_words, oven, laundry_timer, illuminate, list_down, chores, weak_scene, timer_cancel, timer_pause, timer_add, list_complete, playback_resume, vacuum_start, hours, minutes, seconds, list_skip, shopping_names, status_words, window_words, open_close, laundry_hint, bare_switch, outlet_words, tv_words, climate_cool, climate_heat, role_light, role_climate, role_media, role_fan, generic, room_level, extra_device_nouns, article_one);
            extend_sets!(c, p.maps; room_index_nouns);
            c.chat_greet.extend(p.chat.greet.iter().copied());
            c.chat_thanks.extend(p.chat.thanks.iter().copied());
            c.chat_feeling.extend(p.chat.feeling.iter().copied());
            c.chat_identity.extend(p.chat.identity.iter().copied());
            c.chat_tell.extend(p.chat.tell.iter().copied());
            c.chat_yarn.extend(p.chat.yarn.iter().copied());
            c.chat_world.extend(p.chat.world.iter().copied());
            c.chat_advice.extend(p.chat.advice.iter().copied());
            c.chat_open.extend(p.chat.open.iter().copied());
            c.chat_news.extend(p.chat.news.iter().copied());
            c.chat_news_dismiss.extend(p.chat.news_dismiss.iter().copied());
            if c.news_intro.is_empty() {
                c.news_intro = p.chat.news_intro;
                c.news_nudge = p.chat.news_nudge;
                c.news_done = p.chat.news_done;
            }
        }
        c
    }

    fn empty() -> Self {
        Self {
            packs: Vec::new(),
            langs: Vec::new(),
            verbs: HashMap::new(),
            domain_map: HashMap::new(),
            colors: HashMap::new(),
            numbers: HashMap::new(),
            number_styles: Vec::new(),
            fixture_aliases: HashMap::new(),
            group_clarify: Vec::new(),
            strip_pairs: Vec::new(),
            keep_after: Vec::new(),
            synonym_pairs: Vec::new(),
            scene_synonyms: Vec::new(),
            speech: Vec::new(),
            fillers: HashSet::new(),
            action_keep: HashSet::new(),
            conjunctions: HashSet::new(),
            particles: HashSet::new(),
            affirm: HashSet::new(),
            or_words: HashSet::new(),
            all_words: HashSet::new(),
            query_hint: HashSet::new(),
            question_starts: HashSet::new(),
            question_words: HashSet::new(),
            correction: HashSet::new(),
            correction_phrases: HashSet::new(),
            clarify_pick: HashSet::new(),
            light_nouns: HashSet::new(),
            light_singular: HashSet::new(),
            light_plural: HashSet::new(),
            cover_nouns: HashSet::new(),
            curtain_nouns: HashSet::new(),
            fan_nouns: HashSet::new(),
            climate_nouns: HashSet::new(),
            media_nouns: HashSet::new(),
            lock_nouns: HashSet::new(),
            door_nouns: HashSet::new(),
            garage_words: HashSet::new(),
            garage_cover: HashSet::new(),
            timer_nouns: HashSet::new(),
            list_nouns: HashSet::new(),
            vacuum_nouns: HashSet::new(),
            scene_nouns: HashSet::new(),
            script_words: HashSet::new(),
            switch_plural: HashSet::new(),
            device_side: HashSet::new(),
            named_device: HashSet::new(),
            island: HashSet::new(),
            ceiling: HashSet::new(),
            lamp_fixture: HashSet::new(),
            pendant: HashSet::new(),
            bedside: HashSet::new(),
            left: HashSet::new(),
            right: HashSet::new(),
            sides: HashSet::new(),
            singular_lamp: HashSet::new(),
            singular_lamp_block: HashSet::new(),
            power_words: HashSet::new(),
            command_hedges: HashSet::new(),
            skip_light: HashSet::new(),
            laundry_area: HashSet::new(),
            laundry_machines: HashSet::new(),
            kitchen: HashSet::new(),
            open_words: HashSet::new(),
            close_words: HashSet::new(),
            roll_close: HashSet::new(),
            unlock_follow: HashSet::new(),
            cover_open_follow: HashSet::new(),
            garage_lock_block: HashSet::new(),
            on_words: HashSet::new(),
            off_words: HashSet::new(),
            scene_named: HashSet::new(),
            temp_query: HashSet::new(),
            timer_query: HashSet::new(),
            brightness: HashSet::new(),
            start_words: HashSet::new(),
            replay_on_off: HashSet::new(),
            replay_off: HashSet::new(),
            sensor_words: HashSet::new(),
            lock_verbs: HashSet::new(),
            entry_words: HashSet::new(),
            oven: HashSet::new(),
            laundry_timer: HashSet::new(),
            illuminate: HashSet::new(),
            list_down: HashSet::new(),
            chores: HashSet::new(),
            weak_scene: HashSet::new(),
            timer_cancel: HashSet::new(),
            timer_pause: HashSet::new(),
            timer_add: HashSet::new(),
            list_complete: HashSet::new(),
            playback_resume: HashSet::new(),
            vacuum_start: HashSet::new(),
            hours: HashSet::new(),
            minutes: HashSet::new(),
            seconds: HashSet::new(),
            list_skip: HashSet::new(),
            shopping_names: HashSet::new(),
            status_words: HashSet::new(),
            window_words: HashSet::new(),
            open_close: HashSet::new(),
            laundry_hint: HashSet::new(),
            bare_switch: HashSet::new(),
            outlet_words: HashSet::new(),
            tv_words: HashSet::new(),
            climate_cool: HashSet::new(),
            climate_heat: HashSet::new(),
            role_light: HashSet::new(),
            role_climate: HashSet::new(),
            role_media: HashSet::new(),
            role_fan: HashSet::new(),
            generic: HashSet::new(),
            room_level: HashSet::new(),
            extra_device_nouns: HashSet::new(),
            article_one: HashSet::new(),
            room_index_nouns: HashSet::new(),
            chat_greet: HashSet::new(),
            chat_thanks: HashSet::new(),
            chat_feeling: HashSet::new(),
            chat_identity: HashSet::new(),
            chat_tell: HashSet::new(),
            chat_yarn: HashSet::new(),
            chat_world: HashSet::new(),
            chat_advice: HashSet::new(),
            chat_open: HashSet::new(),
            chat_news: HashSet::new(),
            chat_news_dismiss: HashSet::new(),
            news_intro: "",
            news_nudge: "",
            news_done: "",
        }
    }

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

    pub fn color(&self, t: &str) -> Option<&'static str> {
        self.colors.get(t).copied().or_else(|| {
            ["en", "em", "er", "es", "e"].iter().find_map(|suffix| t.strip_suffix(suffix).and_then(|stem| self.colors.get(stem)).copied())
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
        let cat = catalog();
        let licht = vec!["licht".into()];
        assert!(cat.pack_any(&licht, |p| p.nouns.light_nouns));
        assert_eq!(cat.pack_any(&licht, |p| p.nouns.light_nouns), cat.any(&licht, &cat.light_nouns));
        assert!(!cat.pack_has("licht", |p| p.nouns.cover_nouns));
    }

    #[test]
    fn color_accepts_german_endings() {
        let cat = catalog();
        assert_eq!(cat.color("rot"), Some("red"));
        assert_eq!(cat.color("rote"), Some("red"));
        assert_eq!(cat.color("rotes"), Some("red"));
        assert_eq!(cat.color_spoken("red"), "rot");
    }
}
