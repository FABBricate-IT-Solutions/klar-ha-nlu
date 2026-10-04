//! Ranked ASR bias terms for wyoming-faster-whisper / openai `prompt`.
//! Only hard, ASR-confusable names — not common rooms, verbs, or pack lexicon.
//! Never touches audio or Whisper/Parakeet itself.

use crate::home::expose::assist_visible;
use crate::home::policy::{is_infra, is_nlu_ignored};
use crate::lang::Catalog;
use crate::parse::normalize::fold_umlaut;
use crate::types::{CustomSentence, HomeGraph};
use serde::Serialize;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

pub const ASR_BOOST_SCHEMA: &str = "1";
pub const DEFAULT_MAX_TOKENS: usize = 200;
pub const MAX_MAX_TOKENS: usize = 223;

const MIN_TERM_CHARS: usize = 3;
const MAX_TERM_CHARS: usize = 40;
/// Prefer short confusable names, but ignore 1–2 letter noise (`pc`, `ac`).
const FUZZY_SHORT_CHARS: usize = 6;
const MIN_FUZZY_SHORT_CHARS: usize = 4;

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
    /// Lower fills the prompt first (fuzzy entities → custom anchors).
    sort_group: u8,
}

/// Build a capped ASR boost prompt. Pure: no I/O, no network.
///
/// Only difficult names: short / umlaut / cover-curtain confusables and
/// distinctive custom-phrase anchors. Common places, pack verbs, and
/// generic device words (`Licht`, `an`, …) are intentionally omitted so
/// they cannot bias everyday commands.
pub fn build_asr_boost(home: &HomeGraph, catalog: &Catalog, custom: &[CustomSentence], language: &str, max_tokens: usize) -> AsrBoostOut {
    let budget = max_tokens.clamp(1, MAX_MAX_TOKENS);
    let mut seen_fold = HashSet::new();
    let mut candidates: Vec<Candidate> = Vec::new();

    push_hard_entities(home, &mut candidates, &mut seen_fold, catalog);
    push_custom_anchors(custom, catalog, &mut candidates, &mut seen_fold);

    candidates.sort_by(|a, b| {
        a.sort_group
            .cmp(&b.sort_group)
            .then_with(|| a.tier.cmp(&b.tier))
            .then_with(|| {
                // Prefer short confusable entity names; longer custom anchors next.
                if a.sort_group == 1 {
                    a.text.len().cmp(&b.text.len())
                } else {
                    b.text.len().cmp(&a.text.len())
                }
            })
            .then_with(|| a.text.to_lowercase().cmp(&b.text.to_lowercase()))
    });

    let mut terms = Vec::new();
    let mut used_tokens = 0usize;
    let mut dropped = 0usize;
    fill_terms(&candidates, budget, &mut terms, &mut used_tokens, &mut dropped);

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

fn push_hard_entities(home: &HomeGraph, out: &mut Vec<Candidate>, seen: &mut HashSet<String>, catalog: &Catalog) {
    let places = place_folds(home);
    for entity in &home.entities {
        if !assist_visible(entity, home) || is_infra(entity) || is_nlu_ignored(entity) {
            continue;
        }
        let names: Vec<&str> = std::iter::once(entity.name.as_str()).chain(entity.aliases.iter().map(String::as_str)).collect();
        for name in names {
            // Prefer individual hard tokens ("Rollo", "Kugel") over "Rollo Wohnzimmer" / "Küche Licht".
            for token in tokenize_phrase(name) {
                if !is_boost_token(&token, catalog, &places) {
                    continue;
                }
                let tier = if is_fuzzy_prone(&token, catalog) || is_cover_curtain_noun(&token, catalog) {
                    3
                } else {
                    2
                };
                accept(out, seen, &token, tier, Some(entity.entity_id.clone()), 1);
            }
        }
    }
}

fn place_folds(home: &HomeGraph) -> HashSet<String> {
    let mut places = HashSet::new();
    for floor in &home.floors {
        places.insert(fold_umlaut(&floor.name));
        for alias in &floor.aliases {
            places.insert(fold_umlaut(alias));
        }
    }
    for area in &home.areas {
        places.insert(fold_umlaut(&area.name));
        for alias in &area.aliases {
            places.insert(fold_umlaut(alias));
        }
    }
    places
}

fn push_custom_anchors(custom: &[CustomSentence], catalog: &Catalog, out: &mut Vec<Candidate>, seen: &mut HashSet<String>) {
    for row in custom {
        for token in tokenize_phrase(&row.phrase) {
            if token.chars().count() < 5 {
                continue;
            }
            if is_generic_lexicon(&token, catalog) {
                continue;
            }
            accept(out, seen, &token, 4, None, 2);
        }
    }
}

fn fill_terms(candidates: &[Candidate], budget: usize, terms: &mut Vec<AsrBoostTerm>, used_tokens: &mut usize, dropped: &mut usize) {
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
    if !(MIN_TERM_CHARS..=MAX_TERM_CHARS).contains(&chars) {
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

/// Single token worth biasing (never common rooms / Licht / an / aus).
fn is_boost_token(text: &str, catalog: &Catalog, places: &HashSet<String>) -> bool {
    let folded = fold_umlaut(text);
    if folded.chars().count() < MIN_TERM_CHARS {
        return false;
    }
    if places.contains(&folded) || is_generic_lexicon(text, catalog) || is_noise_token(text) {
        return false;
    }
    if is_cover_curtain_noun(text, catalog) {
        return true;
    }
    if has_non_ascii_letter(text) || has_digit(text) || text.contains('-') {
        return true;
    }
    // Short distinctive nicknames: Kugel, Rollo — not kitchen/light lexicon.
    is_fuzzy_prone(text, catalog)
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
    if (MIN_FUZZY_SHORT_CHARS..=FUZZY_SHORT_CHARS).contains(&chars) {
        return true;
    }
    is_cover_curtain_noun(text, catalog)
}

fn is_cover_curtain_noun(text: &str, catalog: &Catalog) -> bool {
    let needle = fold_umlaut(text);
    catalog.curtain_nouns().iter().any(|w| fold_umlaut(w) == needle) || catalog.cover_nouns().iter().any(|w| fold_umlaut(w) == needle)
}

fn is_noise_token(text: &str) -> bool {
    matches!(
        fold_umlaut(text).as_str(),
        "all"
            | "alle"
            | "ac"
            | "pc"
            | "tv"
            | "led"
            | "amp"
            | "usb"
            | "rgb"
            | "og"
            | "eg"
            | "kg"
            | "ml"
            | "front"
            | "door"
            | "night"
            | "movie"
            | "decke"
            | "termin"
    )
}

/// Everyday command lexicon — never boost these (they poison "Licht aus" / "schalte ein").
fn is_generic_lexicon(text: &str, catalog: &Catalog) -> bool {
    let folded = fold_umlaut(text);
    let lower = text.to_ascii_lowercase();
    if catalog.is_filler(&folded)
        || catalog.is_filler(&lower)
        || catalog.is_particle(&folded)
        || catalog.is_conj(&folded)
        || catalog.open_words().contains(folded.as_str())
        || catalog.close_words().contains(folded.as_str())
        || catalog.verb(&folded).is_some()
        || catalog.verb(&lower).is_some()
    {
        return true;
    }
    catalog
        .light_singular()
        .iter()
        .chain(catalog.light_plural().iter())
        .chain(catalog.light_nouns().iter())
        .chain(catalog.fan_nouns().iter())
        .chain(catalog.media_nouns().iter())
        .chain(catalog.climate_nouns().iter())
        .chain(catalog.door_nouns().iter())
        .chain(catalog.lock_nouns().iter())
        .chain(catalog.vacuum_nouns().iter())
        .chain(catalog.calendar_nouns().iter())
        .any(|w| fold_umlaut(w) == folded)
}

fn has_non_ascii_letter(text: &str) -> bool {
    text.chars().any(|c| c.is_alphabetic() && !c.is_ascii())
}

fn has_digit(text: &str) -> bool {
    text.chars().any(|c| c.is_ascii_digit())
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
    fn budget_drops_overflow_and_keeps_hard_names() {
        let mut home = HomeGraph {
            entities: vec![
                EntityRec {
                    entity_id: "light.kugel".into(),
                    name: "Kugel".into(),
                    domain: "light".into(),
                    platform: None,
                    area: None,
                    aliases: vec![],
                    tags: Vec::new(),
                },
                EntityRec {
                    entity_id: "cover.rollo".into(),
                    name: "Rollo".into(),
                    domain: "cover".into(),
                    platform: None,
                    area: None,
                    aliases: vec![],
                    tags: Vec::new(),
                },
            ],
            assist: Some(["light.kugel".into(), "cover.rollo".into()].into()),
            ..Default::default()
        };
        for i in 0..80 {
            let id = format!("cover.extra_{i}");
            home.entities.push(EntityRec {
                entity_id: id.clone(),
                name: format!("Vx{i:02}"),
                domain: "cover".into(),
                platform: None,
                area: None,
                aliases: vec![],
                tags: Vec::new(),
            });
            home.assist.as_mut().unwrap().insert(id);
        }
        let out = build_asr_boost(&home, cat(), &[], "de", 12);
        assert!(estimate_tokens(&out.prompt) <= 12, "prompt tokens {}", estimate_tokens(&out.prompt));
        assert!(out.dropped > 0);
        assert!(
            out.prompt.contains("Kugel") || out.prompt.contains("Rollo") || out.prompt.contains("Vx"),
            "{:?}",
            out.prompt
        );
        assert!(!out.prompt.split_whitespace().any(|w| w == "Wohnzimmer"), "places must not leak: {}", out.prompt);
        assert_eq!(out.schema_version, "1");
        assert_eq!(out.language, "de");
        assert_eq!(out.max_tokens, 12);
    }

    #[test]
    fn only_hard_entity_names_not_easy_places_or_lights() {
        let home = HomeGraph {
            floors: vec![FloorRec { floor_id: "og".into(), name: "Obergeschoss".into(), aliases: vec!["OG".into()], level: Some(1) }],
            areas: vec![AreaRec { area_id: "studio".into(), name: "Studio".into(), aliases: vec![], floor_id: Some("og".into()) }],
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
                    entity_id: "light.wohnzimmer".into(),
                    name: "Wohnzimmer Licht".into(),
                    domain: "light".into(),
                    platform: None,
                    area: Some("studio".into()),
                    aliases: vec!["Licht".into()],
                    tags: Vec::new(),
                },
            ],
            assist: Some(["cover.studio_vorhang".into(), "light.wohnzimmer".into()].into()),
            ..Default::default()
        };
        let out = build_asr_boost(&home, cat(), &[], "de", 40);
        let texts: Vec<&str> = out.terms.iter().map(|t| t.text.as_str()).collect();
        assert!(texts.contains(&"Vorhang"), "{texts:?}");
        assert!(texts.iter().any(|t| t.contains("Studio-Vorhang") || *t == "Studio-Vorhang"), "{texts:?}");
        assert!(!texts.iter().any(|t| *t == "Studio" || *t == "Obergeschoss"), "places leaked: {texts:?}");
        assert!(!texts.iter().any(|t| t.eq_ignore_ascii_case("licht")), "generic light leaked: {texts:?}");
        assert!(!texts.iter().any(|t| *t == "Wohnzimmer Licht"), "easy light name leaked: {texts:?}");
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
        let custom =
            vec![CustomSentence { phrase: "bitte die Papiertonne rausstellen".into(), intent: "HassTurnOn".into(), slots: HashMap::new() }];
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
        let custom = vec![CustomSentence { phrase: "Mülltonne raus".into(), intent: "HassTurnOn".into(), slots: HashMap::new() }];
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

    #[test]
    fn empty_home_emits_no_pack_lexicon() {
        let out = build_asr_boost(&HomeGraph::default(), cat(), &[], "de", 40);
        assert!(out.terms.is_empty(), "expected empty hard-name prompt, got {:?}", out.terms);
        assert!(out.prompt.is_empty());
    }
}
