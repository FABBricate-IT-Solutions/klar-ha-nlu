//! Ranked ASR bias terms for wyoming-faster-whisper `--initial-prompt`.
//! Built from the Klar home graph + pack lexicon + custom phrase anchors.
//! Never touches audio or Whisper itself.

use crate::home::expose::assist_visible;
use crate::home::policy::{is_infra, is_nlu_ignored};
use crate::lang::Catalog;
use crate::lang::VerbKind;
use crate::parse::normalize::fold_umlaut;
use crate::types::{CustomSentence, HomeGraph};
use serde::Serialize;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub const ASR_BOOST_SCHEMA: &str = "1";
pub const DEFAULT_MAX_TOKENS: usize = 200;
pub const MAX_MAX_TOKENS: usize = 223;

const MIN_TERM_CHARS: usize = 2;
const MAX_TERM_CHARS: usize = 40;
const FUZZY_SHORT_CHARS: usize = 6;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AsrBoostTerm {
    pub text: String,
    pub tier: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AsrBoostOut {
    pub schema_version: String,
    pub language: String,
    pub max_tokens: usize,
    pub prompt: String,
    pub terms: Vec<AsrBoostTerm>,
    pub dropped: usize,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
struct Candidate {
    text: String,
    tier: u8,
    entity_id: Option<String>,
    /// Lower fills the prompt first (areas → fuzzy entities → other entities → …).
    sort_group: u8,
}

/// Build a capped ASR boost prompt. Pure: no I/O, no network.
pub fn build_asr_boost(
    home: &HomeGraph,
    catalog: &Catalog,
    custom: &[CustomSentence],
    language: &str,
    max_tokens: usize,
) -> AsrBoostOut {
    let budget = max_tokens.clamp(1, MAX_MAX_TOKENS);
    let mut seen_fold = HashSet::new();
    let mut candidates: Vec<Candidate> = Vec::new();

    push_places(home, &mut candidates, &mut seen_fold);
    push_entities(home, &mut candidates, &mut seen_fold, catalog);
    push_custom_anchors(custom, catalog, &mut candidates, &mut seen_fold);
    push_pack_terms(catalog, &mut candidates, &mut seen_fold);

    candidates.sort_by(|a, b| {
        a.sort_group
            .cmp(&b.sort_group)
            .then_with(|| a.tier.cmp(&b.tier))
            .then_with(|| {
                // Places + custom anchors: prefer longer distinctive words.
                // Entities: prefer short/fuzzy names.
                if matches!(a.sort_group, 1 | 4) {
                    b.text.len().cmp(&a.text.len())
                } else {
                    a.text.len().cmp(&b.text.len())
                }
            })
            .then_with(|| a.text.to_lowercase().cmp(&b.text.to_lowercase()))
    });

    // Keep ~15% for custom anchors + pack verbs so large graphs cannot starve them.
    let tail_reserve = (budget * 15 / 100).max(8).min(budget / 3);
    let head_budget = budget.saturating_sub(tail_reserve);
    let (head, tail): (Vec<_>, Vec<_>) = candidates.into_iter().partition(|c| c.sort_group <= 3);

    let mut terms = Vec::new();
    let mut used_tokens = 0usize;
    let mut dropped = 0usize;
    fill_terms(&head, head_budget, &mut terms, &mut used_tokens, &mut dropped);
    fill_terms(&tail, budget, &mut terms, &mut used_tokens, &mut dropped);

    let prompt = terms.iter().map(|t| t.text.as_str()).collect::<Vec<_>>().join(" ");
    let updated_at = content_stamp(language, budget, &terms);

    AsrBoostOut {
        schema_version: ASR_BOOST_SCHEMA.into(),
        language: language.to_string(),
        max_tokens: budget,
        prompt,
        terms,
        dropped,
        updated_at,
    }
}

fn push_places(home: &HomeGraph, out: &mut Vec<Candidate>, seen: &mut HashSet<String>) {
    for floor in &home.floors {
        accept(out, seen, &floor.name, 1, None, 1);
        for alias in &floor.aliases {
            // Skip tiny aliases (OG, ess) — they burn budget and barely help Whisper.
            if alias.chars().count() < 4 {
                continue;
            }
            accept(out, seen, alias, 1, None, 1);
        }
    }
    for area in &home.areas {
        if crate::home::policy::is_whole_home(area) {
            continue;
        }
        accept(out, seen, &area.name, 1, None, 1);
        for alias in &area.aliases {
            if alias.chars().count() < 4 {
                continue;
            }
            accept(out, seen, alias, 1, None, 1);
        }
    }
}

fn push_entities(home: &HomeGraph, out: &mut Vec<Candidate>, seen: &mut HashSet<String>, catalog: &Catalog) {
    for entity in &home.entities {
        if !assist_visible(entity, home) || is_infra(entity) || is_nlu_ignored(entity) {
            continue;
        }
        let names: Vec<&str> = std::iter::once(entity.name.as_str()).chain(entity.aliases.iter().map(String::as_str)).collect();
        for name in names {
            let fuzzy = is_fuzzy_prone(name, catalog);
            let (tier, group) = if fuzzy { (3, 2) } else { (2, 3) };
            accept(out, seen, name, tier, Some(entity.entity_id.clone()), group);
        }
    }
}

fn push_custom_anchors(custom: &[CustomSentence], catalog: &Catalog, out: &mut Vec<Candidate>, seen: &mut HashSet<String>) {
    for row in custom {
        for token in tokenize_phrase(&row.phrase) {
            if token.chars().count() < 5 {
                continue;
            }
            let folded = fold_umlaut(&token);
            let lower = token.to_ascii_lowercase();
            if catalog.is_filler(&folded)
                || catalog.is_filler(&lower)
                || catalog.is_particle(&folded)
                || catalog.is_conj(&folded)
                || catalog.open_words().contains(folded.as_str())
                || catalog.close_words().contains(folded.as_str())
            {
                continue;
            }
            if catalog.verb(&folded).is_some() || catalog.verb(&lower).is_some() {
                continue;
            }
            accept(out, seen, &token, 4, None, 4);
        }
    }
}

fn push_pack_terms(catalog: &Catalog, out: &mut Vec<Candidate>, seen: &mut HashSet<String>) {
    const BOOST_VERBS: &[VerbKind] = &[
        VerbKind::Dim,
        VerbKind::Open,
        VerbKind::Close,
        VerbKind::Brightness,
        VerbKind::Climate,
        VerbKind::Temperature,
        VerbKind::Lock,
        VerbKind::Unlock,
        VerbKind::Pause,
        VerbKind::Play,
        VerbKind::Mute,
        VerbKind::Position,
    ];
    for word in catalog
        .cover_nouns()
        .iter()
        .chain(catalog.curtain_nouns().iter())
        .chain(catalog.climate_nouns().iter())
        .chain(catalog.light_singular().iter())
        .chain(catalog.lock_nouns().iter())
        .chain(catalog.fan_nouns().iter())
        .chain(catalog.media_nouns().iter())
        .chain(catalog.vacuum_nouns().iter())
    {
        if catalog.is_filler(word) {
            continue;
        }
        accept(out, seen, word, 5, None, 5);
    }
    for (word, kind) in catalog.verb_entries() {
        if BOOST_VERBS.contains(&kind) && word.chars().count() >= 4 {
            accept(out, seen, word, 5, None, 5);
        }
    }
}

fn fill_terms(
    candidates: &[Candidate],
    budget: usize,
    terms: &mut Vec<AsrBoostTerm>,
    used_tokens: &mut usize,
    dropped: &mut usize,
) {
    for cand in candidates {
        let cost = estimate_tokens(&cand.text);
        if *used_tokens + cost > budget {
            *dropped += 1;
            continue;
        }
        *used_tokens += cost;
        terms.push(AsrBoostTerm { text: cand.text.clone(), tier: cand.tier, entity_id: cand.entity_id.clone() });
    }
}

fn accept(out: &mut Vec<Candidate>, seen: &mut HashSet<String>, raw: &str, tier: u8, entity_id: Option<String>, sort_group: u8) {
    let text = normalize_term(raw);
    if text.is_empty() {
        return;
    }
    let chars = text.chars().count();
    if chars < MIN_TERM_CHARS || chars > MAX_TERM_CHARS {
        return;
    }
    let key = fold_umlaut(&text);
    if !seen.insert(key) {
        return;
    }
    out.push(Candidate { text, tier, entity_id, sort_group });
}

fn normalize_term(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    // Keep original casing for Whisper bias; collapse internal whitespace.
    trimmed.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn tokenize_phrase(phrase: &str) -> Vec<String> {
    phrase
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | ':' | '!' | '?' | '.' | '"' | '\''))
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect()
}

fn is_fuzzy_prone(text: &str, catalog: &Catalog) -> bool {
    let folded = fold_umlaut(text);
    let chars = folded.chars().count();
    if chars == 0 {
        return false;
    }
    if has_non_ascii_letter(text) {
        return true;
    }
    if chars <= FUZZY_SHORT_CHARS {
        return true;
    }
    let needle = folded.as_str();
    catalog.curtain_nouns().iter().any(|w| fold_umlaut(w) == needle)
        || catalog.cover_nouns().iter().any(|w| fold_umlaut(w) == needle)
}

fn has_non_ascii_letter(text: &str) -> bool {
    text.chars().any(|c| c.is_alphabetic() && !c.is_ascii())
}

/// Whisper-style rough budget: ~4 chars per token, at least 1 per whitespace word.
pub fn estimate_tokens(text: &str) -> usize {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.is_empty() {
        return 0;
    }
    words.iter().map(|w| ((w.chars().count() + 3) / 4).max(1)).sum()
}

fn content_stamp(language: &str, max_tokens: usize, terms: &[AsrBoostTerm]) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    language.hash(&mut hasher);
    max_tokens.hash(&mut hasher);
    for term in terms {
        term.text.hash(&mut hasher);
        term.tier.hash(&mut hasher);
        term.entity_id.hash(&mut hasher);
    }
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::default_home;
    use crate::lang::catalog_for;
    use crate::types::{AreaRec, EntityRec, FloorRec, HomeGraph};
    use std::collections::{BTreeSet, HashMap};

    fn cat() -> &'static Catalog {
        catalog_for(&["de".into()])
    }

    #[test]
    fn budget_drops_overflow_and_keeps_high_priority() {
        let mut home = default_home();
        home.assist = Some(home.entities.iter().map(|e| e.entity_id.clone()).collect());
        for i in 0..80 {
            home.entities.push(EntityRec {
                entity_id: format!("light.extra_{i}"),
                name: format!("Extra Langname Gerät Nummer {i}"),
                domain: "light".into(),
                platform: None,
                area: Some("wohnzimmer".into()),
                aliases: vec![format!("alias lang {i}")],
                tags: Vec::new(),
            });
            home.assist.as_mut().unwrap().insert(format!("light.extra_{i}"));
        }
        let out = build_asr_boost(&home, cat(), &[], "de", 40);
        assert!(estimate_tokens(&out.prompt) <= 40, "prompt tokens {}", estimate_tokens(&out.prompt));
        assert!(out.dropped > 0);
        assert!(out.prompt.contains("Wohnzimmer"), "{:?}", out.prompt);
        assert_eq!(out.schema_version, "1");
        assert_eq!(out.language, "de");
        assert_eq!(out.max_tokens, 40);
    }

    #[test]
    fn areas_and_fuzzy_names_rank_before_long_entity_names() {
        let home = HomeGraph {
            floors: vec![FloorRec {
                floor_id: "og".into(),
                name: "Obergeschoss".into(),
                aliases: vec!["OG".into()],
                level: Some(1),
            }],
            areas: vec![AreaRec {
                area_id: "studio".into(),
                name: "Studio".into(),
                aliases: vec![],
                floor_id: Some("og".into()),
            }],
            entities: vec![
                EntityRec {
                    entity_id: "cover.studio_vorhang".into(),
                    name: "Vorhang".into(),
                    domain: "cover".into(),
                    platform: None,
                    area: Some("studio".into()),
                    aliases: vec!["Studio-Vorhang".into()],
                    tags: Vec::new(),
                },
                EntityRec {
                    entity_id: "light.sehr_langer_name".into(),
                    name: "Sehr Langer Alias Name Stehlampe Wohnbereich".into(),
                    domain: "light".into(),
                    platform: None,
                    area: Some("studio".into()),
                    aliases: vec![],
                    tags: Vec::new(),
                },
            ],
            assist: Some(["cover.studio_vorhang".into(), "light.sehr_langer_name".into()].into()),
            ..Default::default()
        };
        let out = build_asr_boost(&home, cat(), &[], "de", 12);
        let texts: Vec<&str> = out.terms.iter().map(|t| t.text.as_str()).collect();
        assert!(texts.contains(&"Studio"), "{texts:?}");
        assert!(texts.contains(&"Vorhang"), "{texts:?}");
        let studio = texts.iter().position(|t| *t == "Studio").unwrap();
        let vorhang = texts.iter().position(|t| *t == "Vorhang").unwrap();
        assert!(studio < vorhang, "{texts:?}");
        let vorhang_term = out.terms.iter().find(|t| t.text == "Vorhang").unwrap();
        assert_eq!(vorhang_term.tier, 3);
        assert_eq!(vorhang_term.entity_id.as_deref(), Some("cover.studio_vorhang"));
    }

    #[test]
    fn skips_infra_fillers_and_unexposed() {
        let mut home = default_home();
        home.assist = Some(["light.wohnzimmer".into()].into());
        home.entities.push(EntityRec {
            entity_id: "light.hidden".into(),
            name: "Geheimlampe".into(),
            domain: "light".into(),
            platform: None,
            area: Some("wohnzimmer".into()),
            aliases: vec![],
            tags: Vec::new(),
        });
        home.entities.push(EntityRec {
            entity_id: "light.infra_led".into(),
            name: "Voice LED".into(),
            domain: "light".into(),
            platform: Some("esphome".into()),
            area: Some("wohnzimmer".into()),
            aliases: vec![],
            tags: vec!["infra".into()],
        });
        let custom = vec![CustomSentence {
            phrase: "bitte die Papiertonne rausstellen".into(),
            intent: "HassTurnOn".into(),
            slots: HashMap::new(),
        }];
        let out = build_asr_boost(&home, cat(), &custom, "de", 200);
        let texts: BTreeSet<&str> = out.terms.iter().map(|t| t.text.as_str()).collect();
        assert!(!texts.contains("Geheimlampe"), "{texts:?}");
        assert!(!texts.contains("Voice LED"), "{texts:?}");
        assert!(!texts.contains("bitte"), "{texts:?}");
        assert!(texts.iter().any(|t| t.eq_ignore_ascii_case("Papiertonne") || t.contains("Papiertonne")), "{texts:?}");
    }

    #[test]
    fn custom_anchors_are_tier_four() {
        let home = HomeGraph::default();
        let custom = vec![CustomSentence {
            phrase: "Mülltonne raus".into(),
            intent: "HassTurnOn".into(),
            slots: HashMap::new(),
        }];
        let out = build_asr_boost(&home, cat(), &custom, "de", 50);
        let muell = out.terms.iter().find(|t| fold_umlaut(&t.text).contains("mulltonne") || t.text.contains("Mülltonne"));
        assert!(muell.is_some(), "{:?}", out.terms);
        assert_eq!(muell.unwrap().tier, 4);
    }

    #[test]
    fn estimate_tokens_counts_words() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("Vorhang"), 2); // 7 chars → 2
        assert!(estimate_tokens("Wohnzimmer Studio Stehlampe") >= 3);
    }

    #[test]
    fn fuzzy_examples_cover_vorhang_vs_vorrang() {
        let home = HomeGraph {
            entities: vec![EntityRec {
                entity_id: "cover.vorhang".into(),
                name: "Vorhang".into(),
                domain: "cover".into(),
                platform: None,
                area: None,
                aliases: vec![],
                tags: Vec::new(),
            }],
            assist: Some(["cover.vorhang".into()].into()),
            ..Default::default()
        };
        let out = build_asr_boost(&home, cat(), &[], "de", 20);
        assert!(out.terms.iter().any(|t| t.text == "Vorhang"), "{:?}", out.terms);
        assert!(!out.prompt.split_whitespace().any(|w| w == "Vorrang"));
    }
}
