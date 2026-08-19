use super::catalog::Catalog;
use super::verbs::VerbKind;
use crate::parse::fuzzy::{evidence, select_unique, Evidence, Profile};
use strsim::levenshtein;

impl Catalog {
    pub fn fuzzy_verb(&self, token: &str) -> Option<VerbKind> {
        if !self.safe_structural_token(token) {
            return None;
        }
        let mut ranked: Vec<(VerbKind, Evidence)> = Vec::new();
        for (&word, &kind) in &self.verbs {
            if word.len() < 6
                || matches!(
                    kind,
                    VerbKind::Color
                        | VerbKind::Percent
                        | VerbKind::Pause
                        | VerbKind::Playback
                        | VerbKind::Play
                        | VerbKind::Next
                        | VerbKind::Mute
                        | VerbKind::List
                        | VerbKind::Add
                        | VerbKind::ListComplete
                )
            {
                continue;
            }
            let Some(hit) = evidence(token, word, Profile::Structural) else {
                continue;
            };
            if let Some(existing) = ranked.iter_mut().find(|(existing, _)| *existing == kind) {
                if hit.score > existing.1.score || (hit.score == existing.1.score && hit.distance < existing.1.distance) {
                    existing.1 = hit;
                }
            } else {
                ranked.push((kind, hit));
            }
        }
        ranked.sort_by(|left, right| {
            right
                .1
                .score
                .partial_cmp(&left.1.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.1.distance.cmp(&right.1.distance))
        });
        let best = *ranked.first()?;
        if let Some(second) = ranked.get(1) {
            if best.1.score - second.1.score < 0.08 || best.1.distance >= second.1.distance {
                return None;
            }
        }
        Some(best.0)
    }

    pub fn is_protected_typo(&self, token: &str) -> bool {
        token.chars().count() >= 6
            && self.number(token).is_none()
            && token.parse::<i32>().is_err()
            && self.color(token).is_none()
            && self.verb(token).is_none()
            && !self.domain_map.contains_key(token)
            && !self.generic().contains(token)
            && !self.all_words().contains(token)
            && !self.is_except(token)
            && (self
                .colors
                .keys()
                .copied()
                .chain(self.numbers.keys().copied())
                .filter(|candidate| candidate.chars().count() >= 6)
                .any(|candidate| levenshtein(token, candidate) <= 2)
                || self.has_german_compound_number_typo(token))
    }

    fn has_german_compound_number_typo(&self, token: &str) -> bool {
        let parts: Vec<&str> = token.split("und").collect();
        if parts.len() != 2 {
            return false;
        }
        let ones = parts[0] == "ein" || self.number(parts[0]).is_some_and(|value| (1..10).contains(&value));
        ones && self.number(parts[1]).is_none() && self.numbers.iter().any(|(word, value)| *value >= 20 && levenshtein(parts[1], word) <= 2)
    }

    pub fn fuzzy_domain(&self, tokens: &[String]) -> Option<&'static str> {
        let anchored = tokens.iter().any(|token| self.verb(token).is_some());
        if !anchored {
            return None;
        }
        let groups = [
            ("light", self.light_nouns()),
            ("climate", self.climate_nouns()),
            ("cover", self.cover_nouns()),
            ("fan", self.fan_nouns()),
            ("media_player", self.media_nouns()),
            ("lock", self.lock_nouns()),
            ("timer", self.timer_nouns()),
            ("todo", self.list_nouns()),
            ("vacuum", self.vacuum_nouns()),
            ("scene", self.scene_nouns()),
        ];
        let candidates: Vec<(&'static str, &'static str)> = groups
            .into_iter()
            .flat_map(|(domain, words)| words.iter().copied().filter(|word| word.len() >= 6).map(move |word| (domain, word)))
            .collect();
        let hits: Vec<&str> = tokens
            .iter()
            .filter(|token| self.safe_structural_token(token))
            .filter(|token| !self.domain_map.contains_key(token.as_str()))
            .filter_map(|token| select_unique(token, candidates.iter().copied(), Profile::Structural).map(|hit| hit.key))
            .collect();
        (hits.len() == 1).then(|| hits[0])
    }

    fn safe_structural_token(&self, token: &str) -> bool {
        token.chars().count() >= 6
            && !self.is_filler(token)
            && !self.is_particle(token)
            && !self.generic().contains(token)
            && !self.on_words().contains(token)
            && !self.off_words().contains(token)
            && !self.all_words().contains(token)
            && !self.is_except(token)
            && self.number(token).is_none()
            && token.parse::<i32>().is_err()
            && self.color(token).is_none()
    }
}
