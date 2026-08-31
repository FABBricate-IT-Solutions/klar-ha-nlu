mod keys;
mod merge;
mod words;

use super::groups::{GroupClarify, LanguagePack as Pack, NumberStyle};
use super::morphology::Morphology;
use super::speech::Speech;
use super::verbs::{extra_verb, VerbKind};
use super::LangId;
use crate::types::CustomSentence;
use std::collections::{HashMap, HashSet};

pub use keys::WordKey;

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
    pub(super) sets: HashMap<keys::WordKey, HashSet<&'static str>>,
    pub news_intro: &'static str,
    pub news_nudge: &'static str,
    pub news_done: &'static str,
    pub morphology: Morphology,
    pub pack_intents: Vec<CustomSentence>,
}

impl Catalog {
    pub fn verb(&self, t: &str) -> Option<VerbKind> {
        self.verbs.get(t).copied().or_else(|| extra_verb(t))
    }

    pub fn is_filler(&self, t: &str) -> bool {
        self.fillers().contains(t)
    }

    pub fn is_action_keep(&self, t: &str) -> bool {
        self.action_keep().contains(t)
    }

    pub fn is_conj(&self, t: &str) -> bool {
        self.conjunctions().contains(t)
    }

    pub fn is_particle(&self, t: &str) -> bool {
        self.particles().contains(t)
    }

    pub fn is_affirm(&self, t: &str) -> bool {
        self.affirm().contains(t)
    }

    pub fn is_or(&self, t: &str) -> bool {
        self.or_words().contains(t)
    }

    pub fn is_all(&self, t: &str) -> bool {
        self.all_words().contains(t)
    }

    pub fn is_except(&self, t: &str) -> bool {
        self.pack_has(t, |pack| pack.talk.except_words)
    }

    pub fn is_query_hint(&self, t: &str) -> bool {
        self.query_hint().contains(t)
    }

    pub fn any(&self, tokens: &[String], set: &HashSet<&'static str>) -> bool {
        tokens.iter().any(|t| set.contains(t.as_str()))
            || set.iter().any(|phrase| {
                let parts: Vec<&str> = phrase.split_whitespace().collect();
                parts.len() > 1 && tokens.windows(parts.len()).any(|window| window.iter().map(String::as_str).eq(parts.iter().copied()))
            })
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
        self.any(tokens, self.singular_lamp()) && !self.any(tokens, self.singular_lamp_block())
    }

    pub fn is_question_start(&self, t: &str) -> bool {
        self.question_starts().contains(t)
    }

    pub fn is_question_word(&self, t: &str) -> bool {
        self.question_words().contains(t)
    }

    pub fn knows_surface(&self, token: &str) -> bool {
        self.verbs.contains_key(token)
            || self.domain_map.contains_key(token)
            || self.colors.contains_key(token)
            || self.numbers.contains_key(token)
            || self.light_nouns().contains(token)
            || self.cover_nouns().contains(token)
            || self.climate_nouns().contains(token)
            || self.media_nouns().contains(token)
            || self.lock_nouns().contains(token)
            || self.timer_nouns().contains(token)
            || self.list_nouns().contains(token)
            || self.calendar_nouns().contains(token)
            || self.calendar_query().contains(token)
            || self.calendar_create().contains(token)
            || self.calendar_delete().contains(token)
            || self.calendar_move().contains(token)
            || self.calendar_today().contains(token)
            || self.calendar_tomorrow().contains(token)
            || self.calendar_when().contains(token)
            || self.fan_nouns().contains(token)
            || self.vacuum_nouns().contains(token)
            || self.scene_nouns().contains(token)
            || self.on_words().contains(token)
            || self.off_words().contains(token)
            || self.open_words().contains(token)
            || self.close_words().contains(token)
            || self.generic().contains(token)
            || self.all_words().contains(token)
            || self.conjunctions().contains(token)
            || self.question_words().contains(token)
            || self.affirm().contains(token)
            || self.clarify_pick().contains(token)
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
        self.speech.first().copied().unwrap_or(&super::packs::en::PACK.speech)
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
        assert_eq!(cat.pack_any(&licht, |p| p.nouns.light_nouns), cat.any(&licht, cat.light_nouns()));
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

    #[test]
    fn empty_catalog_speech_is_english() {
        let _bind = crate::lang::bind(&[]);
        let speech = catalog().speech();
        assert_eq!(speech.clarify, "Do you mean {names}?");
        assert_eq!(speech.clarify_or, " or ");
        assert_eq!(speech.confirm, "Should I really do that?");
        assert!(!speech.clarify.contains("Meinst du"));
    }
}
