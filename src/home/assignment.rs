use crate::home::expose::assist_visible;
use crate::home::gaps::needs_mapping;
use crate::io::bundle::BundleEntry;
use crate::lang::Catalog;
use crate::parse::normalize::compact;
use crate::types::{AreaRec, EntityRec, HomeGraph};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize)]
pub struct SuggestedArea {
    pub area_id: String,
    pub name: String,
    pub score: u32,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignmentRow {
    pub entity_id: String,
    pub name: String,
    pub domain: String,
    pub area: Option<String>,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
    pub confidence: Confidence,
    pub suggested_area: Option<SuggestedArea>,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Counts {
    pub all: usize,
    pub assist: usize,
    pub rooms: usize,
    pub leftover: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub bundle: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Coverage {
    pub all: usize,
    pub assist: usize,
    pub high: usize,
    pub leftover: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoomReadiness {
    pub area_id: String,
    pub name: String,
    pub count: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
    pub inbox: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrafficPoint {
    pub day: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TrafficRecent {
    pub id: String,
    pub ts_ms: u64,
    pub source: String,
    pub language: Option<String>,
    pub text: String,
    pub speech: String,
    pub intents: Vec<String>,
    pub clarify: bool,
    pub chat: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Traffic {
    pub total: usize,
    pub by_source: BTreeMap<String, usize>,
    pub by_intent: BTreeMap<String, usize>,
    pub by_day: Vec<TrafficPoint>,
    pub clarify: usize,
    pub chat: usize,
    pub empty: usize,
    pub recent: Vec<TrafficRecent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DomainCount {
    pub domain: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FloorEntry {
    pub floor_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Dashboard {
    pub counts: Counts,
    pub coverage: Coverage,
    pub domains: Vec<DomainCount>,
    pub rooms: Vec<RoomReadiness>,
    pub floors: Vec<FloorEntry>,
    pub assignment: Vec<AssignmentRow>,
    pub traffic: Traffic,
}

pub fn build_dashboard(
    home: &HomeGraph,
    bundle: &[BundleEntry],
    dismissed: &[String],
    live_traffic: Traffic,
    catalog: &Catalog,
) -> Dashboard {
    let assignment: Vec<_> = home
        .entities
        .iter()
        .filter(|entity| assist_visible(entity, home))
        .map(|entity| assignment_row(entity, home, dismissed, catalog))
        .collect();
    let high = assignment.iter().filter(|row| row.confidence == Confidence::High).count();
    let medium = assignment.iter().filter(|row| row.confidence == Confidence::Medium).count();
    let low = assignment.iter().filter(|row| row.confidence == Confidence::Low).count();
    let leftover = medium + low;
    Dashboard {
        counts: Counts {
            all: home.entities.len(),
            assist: assignment.len(),
            rooms: home.areas.len(),
            leftover,
            high,
            medium,
            low,
            bundle: bundle.len(),
        },
        coverage: Coverage { all: home.entities.len(), assist: assignment.len(), high, leftover },
        domains: domains(&assignment),
        rooms: rooms(home, &assignment),
        floors: floors(home),
        assignment,
        traffic: if live_traffic.total > 0 { live_traffic } else { traffic(bundle) },
    }
}

pub fn assignment_row(entity: &EntityRec, home: &HomeGraph, dismissed: &[String], catalog: &Catalog) -> AssignmentRow {
    let confidence = confidence(entity, home, catalog);
    let mut reasons = Vec::new();
    if entity.area.is_none() {
        reasons.push("missing_area".into());
    } else if confidence == Confidence::Medium {
        reasons.push("weak_name".into());
    } else {
        reasons.push("ready".into());
    }
    let suggested_area = (!dismissed.iter().any(|id| id == &entity.entity_id)).then(|| suggested_area(entity, home)).flatten();
    AssignmentRow {
        entity_id: entity.entity_id.clone(),
        name: entity.name.clone(),
        domain: entity.domain.clone(),
        area: entity.area.clone(),
        aliases: entity.aliases.clone(),
        tags: entity.tags.clone(),
        confidence,
        suggested_area,
        reasons,
    }
}

pub fn confidence(entity: &EntityRec, home: &HomeGraph, catalog: &Catalog) -> Confidence {
    if entity.area.is_none() {
        Confidence::Low
    } else if needs_mapping(entity, home, catalog) {
        Confidence::Medium
    } else {
        Confidence::High
    }
}

pub fn suggested_area(entity: &EntityRec, home: &HomeGraph) -> Option<SuggestedArea> {
    let hay = entity_tokens(entity);
    let mut best: Option<SuggestedArea> = None;
    for area in &home.areas {
        let (score, reasons) = score_area(&hay, entity, area);
        if score < 2 {
            continue;
        }
        let candidate = SuggestedArea { area_id: area.area_id.clone(), name: area.name.clone(), score, reasons };
        if best.as_ref().is_none_or(|current| candidate.score > current.score) {
            best = Some(candidate);
        }
    }
    best
}

fn entity_tokens(entity: &EntityRec) -> Vec<String> {
    let mut out = split_words(&entity.entity_id);
    out.extend(split_words(&entity.name));
    for alias in &entity.aliases {
        out.extend(split_words(alias));
    }
    out.sort();
    out.dedup();
    out
}

fn split_words(raw: &str) -> Vec<String> {
    raw.split(|c: char| !c.is_ascii_alphanumeric())
        .map(compact)
        .filter(|part| part.len() > 2 && !part.chars().all(|c| c.is_ascii_digit()))
        .collect()
}

fn score_area(tokens: &[String], entity: &EntityRec, area: &AreaRec) -> (u32, Vec<String>) {
    let mut score = 0;
    let mut reasons = Vec::new();
    let area_words =
        std::iter::once(area.area_id.as_str()).chain(std::iter::once(area.name.as_str())).chain(area.aliases.iter().map(String::as_str));
    for raw in area_words {
        let word = compact(raw);
        if word.len() <= 2 {
            continue;
        }
        if tokens.iter().any(|token| token == &word) {
            score += if entity.entity_id.contains(&area.area_id) { 3 } else { 2 };
            reasons.push(format!("match:{word}"));
        }
        if compact(&entity.name).contains(&word) {
            score += 1;
        }
    }
    reasons.sort();
    reasons.dedup();
    (score, reasons)
}

fn domains(rows: &[AssignmentRow]) -> Vec<DomainCount> {
    let mut counts = BTreeMap::<String, usize>::new();
    for row in rows {
        *counts.entry(row.domain.clone()).or_default() += 1;
    }
    counts.into_iter().map(|(domain, count)| DomainCount { domain, count }).collect()
}

fn rooms(home: &HomeGraph, rows: &[AssignmentRow]) -> Vec<RoomReadiness> {
    let mut out: Vec<_> = home
        .areas
        .iter()
        .map(|area| {
            let here: Vec<_> = rows.iter().filter(|row| row.area.as_deref() == Some(area.area_id.as_str())).collect();
            let high = here.iter().filter(|row| row.confidence == Confidence::High).count();
            let medium = here.iter().filter(|row| row.confidence == Confidence::Medium).count();
            let low = here.iter().filter(|row| row.confidence == Confidence::Low).count();
            RoomReadiness {
                area_id: area.area_id.clone(),
                name: area.name.clone(),
                count: here.len(),
                high,
                medium,
                low,
                inbox: medium + low,
            }
        })
        .collect();
    out.sort_by(|a, b| b.inbox.cmp(&a.inbox).then_with(|| a.name.cmp(&b.name)));
    out
}

fn floors(home: &HomeGraph) -> Vec<FloorEntry> {
    let mut floors: Vec<_> = home.floors.iter().filter(|floor| !floor.floor_id.is_empty()).collect();
    floors.sort_by(|a, b| {
        match (a.level, b.level) {
            (Some(left), Some(right)) => left.cmp(&right),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
        .then_with(|| a.name.cmp(&b.name))
        .then_with(|| a.floor_id.cmp(&b.floor_id))
    });
    floors.dedup_by(|a, b| a.floor_id == b.floor_id);
    floors
        .into_iter()
        .map(|floor| FloorEntry {
            floor_id: floor.floor_id.clone(),
            name: if floor.name.is_empty() { floor.floor_id.clone() } else { floor.name.clone() },
        })
        .collect()
}

fn traffic(bundle: &[BundleEntry]) -> Traffic {
    let mut by_source = BTreeMap::new();
    let mut by_intent = BTreeMap::new();
    let mut by_day = BTreeMap::new();
    let mut clarify = 0;
    let mut chat = 0;
    let mut empty = 0;
    for entry in bundle {
        *by_source.entry(entry.source.clone()).or_default() += 1;
        *by_day.entry(day_bucket(entry.ts_ms)).or_default() += 1;
        if entry.response.clarify {
            clarify += 1;
        }
        if entry.response.chat {
            chat += 1;
        }
        if entry.response.intents.is_empty() {
            empty += 1;
        }
        for intent in &entry.response.intents {
            *by_intent.entry(intent.name.clone()).or_default() += 1;
        }
    }
    let mut recent: Vec<_> = bundle.iter().rev().take(15).map(recent).collect();
    recent.reverse();
    Traffic {
        total: bundle.len(),
        by_source,
        by_intent,
        by_day: by_day.into_iter().map(|(day, count)| TrafficPoint { day, count }).collect(),
        clarify,
        chat,
        empty,
        recent,
    }
}

fn day_bucket(ts_ms: u64) -> String {
    format!("d{}", ts_ms / 86_400_000)
}

fn recent(entry: &BundleEntry) -> TrafficRecent {
    TrafficRecent {
        id: entry.id.clone(),
        ts_ms: entry.ts_ms,
        source: entry.source.clone(),
        language: entry.language.clone(),
        text: entry.request.text.clone(),
        speech: entry.response.speech.clone(),
        intents: entry.response.intents.iter().map(|intent| intent.name.clone()).collect(),
        clarify: entry.response.clarify,
        chat: entry.response.chat,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AreaRec, EntityRec, FloorRec, HomeGraph};

    fn ent(id: &str, name: &str, area: Option<&str>) -> EntityRec {
        EntityRec {
            entity_id: id.into(),
            name: name.into(),
            domain: "light".into(),
            platform: None,
            area: area.map(str::to_string),
            aliases: Vec::new(),
            tags: Vec::new(),
        }
    }

    fn home() -> HomeGraph {
        HomeGraph {
            areas: vec![
                AreaRec { area_id: "schlafzimmer".into(), name: "Schlafzimmer".into(), aliases: vec!["bedroom".into()], floor_id: None },
                AreaRec { area_id: "wohnzimmer".into(), name: "Wohnzimmer".into(), aliases: vec!["living".into()], floor_id: None },
            ],
            entities: vec![
                ent("light.schlafzimmer_kugel", "Kugel", None),
                ent("light.hue_play_1", "Hue play 1", Some("wohnzimmer")),
                ent("light.schlafzimmer_licht", "Schlafzimmer Licht", Some("schlafzimmer")),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn suggests_room_from_entity_id() {
        let suggestion = suggested_area(&home().entities[0], &home()).unwrap();
        assert_eq!(suggestion.area_id, "schlafzimmer");
        assert!(suggestion.score >= 3, "{suggestion:?}");
    }

    #[test]
    fn confidence_tracks_mapping_need() {
        let home = home();
        assert_eq!(confidence(&home.entities[0], &home, crate::lang::catalog()), Confidence::Low);
        assert_eq!(confidence(&home.entities[1], &home, crate::lang::catalog()), Confidence::Medium);
        assert_eq!(confidence(&home.entities[2], &home, crate::lang::catalog()), Confidence::High);
    }

    #[test]
    fn dashboard_counts_visible_entities() {
        let home = home();
        let dash = build_dashboard(&home, &[], &[], Traffic::default(), crate::lang::catalog());
        assert_eq!(dash.counts.assist, 3);
        assert_eq!(dash.counts.high + dash.counts.medium + dash.counts.low, dash.counts.assist);
        assert_eq!(dash.coverage.leftover, dash.counts.medium + dash.counts.low);
        assert!(dash.floors.is_empty());
    }

    #[test]
    fn dashboard_lists_floor_ids() {
        let mut home = home();
        home.floors = vec![
            FloorRec { floor_id: "upper".into(), name: "Obergeschoss".into(), aliases: vec!["og".into()], level: Some(1) },
            FloorRec { floor_id: String::new(), name: "Ghost".into(), aliases: Vec::new(), level: None },
            FloorRec { floor_id: "ground".into(), name: "Erdgeschoss".into(), aliases: Vec::new(), level: Some(0) },
        ];
        let dash = build_dashboard(&home, &[], &[], Traffic::default(), crate::lang::catalog());
        assert_eq!(dash.floors.iter().map(|floor| floor.floor_id.as_str()).collect::<Vec<_>>(), ["ground", "upper"]);
        assert_eq!(dash.floors[0].name, "Erdgeschoss");
    }
}
