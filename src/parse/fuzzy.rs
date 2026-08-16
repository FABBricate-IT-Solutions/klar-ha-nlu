use crate::parse::normalize::fold_latin;
use std::cmp::Ordering;
use strsim::levenshtein;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Profile {
    Target,
    Structural,
    Phrase,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Evidence {
    pub score: f64,
    pub distance: usize,
    pub single_gap: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct UniqueMatch<'a> {
    pub key: &'a str,
    pub evidence: Evidence,
}

pub(crate) fn evidence(observed: &str, candidate: &str, profile: Profile) -> Option<Evidence> {
    let observed = normalize(observed, profile);
    let candidate = normalize(candidate, profile);
    if observed.is_empty() || candidate.is_empty() {
        return None;
    }
    if observed == candidate {
        return Some(Evidence { score: 1.0, distance: 0, single_gap: false });
    }
    let observed_len = observed.chars().count();
    let candidate_len = candidate.chars().count();
    if observed_len < profile.min_observed_len() || candidate_len < profile.min_candidate_len() {
        return None;
    }

    let distance = levenshtein(&observed, &candidate);
    let longest = observed_len.max(candidate_len);
    if longest == 0 {
        return None;
    }
    let similarity = 1.0 - distance as f64 / longest as f64;
    if distance <= profile.max_edits(longest) && similarity >= profile.min_similarity() {
        return Some(Evidence { score: similarity, distance, single_gap: false });
    }

    if profile == Profile::Target && single_gap_deletion(&observed, &candidate, distance) {
        let coverage = observed_len as f64 / candidate_len as f64;
        return Some(Evidence { score: 0.84 + coverage * 0.1, distance, single_gap: true });
    }
    None
}

pub(crate) fn select_unique<'a>(
    observed: &str,
    candidates: impl IntoIterator<Item = (&'a str, &'a str)>,
    profile: Profile,
) -> Option<UniqueMatch<'a>> {
    let mut ranked: Vec<UniqueMatch<'a>> = Vec::new();
    for (key, label) in candidates {
        let Some(hit) = evidence(observed, label, profile) else {
            continue;
        };
        if let Some(existing) = ranked.iter_mut().find(|entry| entry.key == key) {
            if evidence_cmp(hit, existing.evidence) == Ordering::Greater {
                existing.evidence = hit;
            }
        } else {
            ranked.push(UniqueMatch { key, evidence: hit });
        }
    }
    ranked.sort_by(|left, right| evidence_cmp(right.evidence, left.evidence));
    let best = *ranked.first()?;
    let Some(second) = ranked.get(1) else {
        return Some(best);
    };
    let clear_score = best.evidence.score - second.evidence.score >= profile.min_margin();
    let clear_distance = best.evidence.distance < second.evidence.distance;
    (clear_score && clear_distance).then_some(best)
}

fn evidence_cmp(left: Evidence, right: Evidence) -> Ordering {
    left.score
        .partial_cmp(&right.score)
        .unwrap_or(Ordering::Equal)
        .then_with(|| right.distance.cmp(&left.distance))
        .then_with(|| right.single_gap.cmp(&left.single_gap))
}

fn normalize(value: &str, profile: Profile) -> String {
    let folded = fold_latin(value);
    if profile == Profile::Phrase {
        folded.split(|c: char| !c.is_alphanumeric()).filter(|part| !part.is_empty()).collect::<Vec<_>>().join(" ")
    } else {
        folded.chars().filter(|c| c.is_alphanumeric()).collect()
    }
}

fn single_gap_deletion(observed: &str, candidate: &str, distance: usize) -> bool {
    let observed: Vec<char> = observed.chars().collect();
    let candidate: Vec<char> = candidate.chars().collect();
    if candidate.len() < 8 || observed.len() < 5 || candidate.len() <= observed.len() {
        return false;
    }
    let removed = candidate.len() - observed.len();
    if !(2..=3).contains(&removed) || distance != removed {
        return false;
    }
    let coverage = observed.len() as f64 / candidate.len() as f64;
    if coverage < 0.65 {
        return false;
    }

    let prefix = observed.iter().zip(&candidate).take_while(|(left, right)| left == right).count();
    if prefix == observed.len() {
        return coverage >= 0.7;
    }
    let suffix =
        observed.iter().rev().zip(candidate.iter().rev()).take_while(|(left, right)| left == right).count().min(observed.len() - prefix);
    (prefix >= 2 && suffix >= 2 && prefix + suffix == observed.len()) || (suffix == observed.len() && coverage >= 0.7)
}

impl Profile {
    fn min_observed_len(self) -> usize {
        match self {
            Self::Target => 5,
            Self::Structural => 6,
            Self::Phrase => 8,
        }
    }

    fn min_candidate_len(self) -> usize {
        match self {
            Self::Target | Self::Structural => 6,
            Self::Phrase => 8,
        }
    }

    fn max_edits(self, longest: usize) -> usize {
        match self {
            Self::Target => match longest {
                0..=7 => 1,
                8..=11 => 2,
                _ => 3,
            },
            Self::Structural => usize::from(longest >= 6) + usize::from(longest >= 10),
            Self::Phrase => (longest / 10).clamp(1, 3),
        }
    }

    fn min_similarity(self) -> f64 {
        match self {
            Self::Target => 0.8,
            Self::Structural => 0.82,
            Self::Phrase => 0.9,
        }
    }

    fn min_margin(self) -> f64 {
        match self {
            Self::Target => 0.06,
            Self::Structural => 0.08,
            Self::Phrase => 0.04,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_accepts_typo_and_swallowed_gap() {
        let typo = evidence("wohnzimer", "wohnzimmer", Profile::Target).expect("one missing letter");
        assert_eq!(typo.distance, 1);
        assert!(!typo.single_gap);

        let swallowed = evidence("wohnzim", "wohnzimmer", Profile::Target).expect("truncated syllable");
        assert_eq!(swallowed.distance, 3);
        assert!(swallowed.single_gap);
    }

    #[test]
    fn target_rejects_short_and_unrelated_tokens() {
        assert!(evidence("bad", "badezimmer", Profile::Target).is_none());
        assert!(evidence("arbeitsraum", "schlafzimmer", Profile::Target).is_none());
    }

    #[test]
    fn unique_match_rejects_equal_competitors() {
        let candidates = [("a", "foobar"), ("b", "foobat")];
        assert!(select_unique("foobaz", candidates, Profile::Structural).is_none());
    }

    #[test]
    fn unique_match_groups_aliases_by_key() {
        let candidates = [("living", "wohnzimmer"), ("living", "wohnraum"), ("dining", "esszimmer")];
        let hit = select_unique("wohnzimer", candidates, Profile::Target).expect("unique area");
        assert_eq!(hit.key, "living");
    }

    #[test]
    fn phrase_requires_high_similarity() {
        assert!(evidence("starte den filmabend", "starte den filmabent", Profile::Phrase).is_some());
        assert!(evidence("starte musik", "schalte heizung", Profile::Phrase).is_none());
    }
}
