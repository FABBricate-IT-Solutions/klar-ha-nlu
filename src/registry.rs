use crate::normalize::fold_umlaut;
use crate::types::{AreaRec, EntityRec, HomeGraph};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct EntityStorage {
    data: EntityData,
}

#[derive(Deserialize)]
struct EntityData {
    entities: Vec<RawEntity>,
}

#[derive(Deserialize)]
struct RawEntity {
    entity_id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    original_name: Option<String>,
    #[serde(default)]
    area_id: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    disabled_by: Option<String>,
    #[serde(default)]
    device_id: Option<String>,
}

#[derive(Deserialize)]
struct DeviceStorage {
    data: DeviceData,
}

#[derive(Deserialize)]
struct DeviceData {
    devices: Vec<RawDevice>,
}

#[derive(Deserialize)]
struct RawDevice {
    id: String,
    #[serde(default)]
    area_id: Option<String>,
}

#[derive(Deserialize)]
struct AreaStorage {
    data: AreaData,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum AreaData {
    Wrapped { areas: Vec<RawArea> },
    Flat(Vec<RawArea>),
}

#[derive(Deserialize)]
struct RawArea {
    id: Option<String>,
    area_id: Option<String>,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
}

pub fn load_home(config_dir: &Path, fallback: HomeGraph) -> HomeGraph {
    let entity_path = config_dir.join(".storage/core.entity_registry");
    let area_path = config_dir.join(".storage/core.area_registry");
    if !entity_path.exists() {
        tracing::info!("kein Entity-Registry unter {}, nutze Default-Wohnung", entity_path.display());
        return fallback;
    }

    let device_areas = read_device_areas(&config_dir.join(".storage/core.device_registry"));
    let mut entities = match read_entities(&entity_path, &device_areas) {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!("Entity-Registry unlesbar: {err}");
            return fallback;
        }
    };
    let areas = read_areas(&area_path).unwrap_or_else(|_| fallback.areas.clone());
    if entities.is_empty() {
        return fallback;
    }
    for ent in &mut entities {
        if ent.area.is_none() {
            ent.area = infer_area(&ent.entity_id, &areas);
        }
    }
    tracing::info!("{} Entitäten, {} Räume aus Home Assistant geladen", entities.len(), areas.len());
    HomeGraph { entities, areas, assist: crate::expose::load_assist(config_dir), ..fallback }
}

fn read_device_areas(path: &Path) -> HashMap<String, String> {
    let Ok(raw) = fs::read_to_string(path) else {
        return HashMap::new();
    };
    let Ok(parsed) = serde_json::from_str::<DeviceStorage>(&raw) else {
        return HashMap::new();
    };
    parsed.data.devices.into_iter().filter_map(|d| d.area_id.map(|area| (d.id, area))).collect()
}

fn infer_area(entity_id: &str, areas: &[AreaRec]) -> Option<String> {
    let slug = entity_id.split_once('.')?.1;
    let mut best: Option<&str> = None;
    for area in areas {
        if slug == area.area_id || slug.starts_with(&format!("{}_", area.area_id)) {
            if best.is_none_or(|cur| area.area_id.len() > cur.len()) {
                best = Some(area.area_id.as_str());
            }
        }
    }
    best.map(str::to_string)
}

fn read_entities(path: &Path, device_areas: &HashMap<String, String>) -> Result<Vec<EntityRec>, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed: EntityStorage = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    Ok(parsed
        .data
        .entities
        .into_iter()
        .filter(|e| e.disabled_by.is_none())
        .filter(|e| keep_domain(&e.entity_id))
        .map(|e| {
            let domain = e.entity_id.split('.').next().unwrap_or("").to_string();
            let name = e.name.or(e.original_name).unwrap_or_else(|| e.entity_id.clone());
            let area = e.area_id.or_else(|| e.device_id.as_ref().and_then(|id| device_areas.get(id).cloned()));
            EntityRec { entity_id: e.entity_id, name, domain, area, aliases: e.aliases, tags: e.labels }
        })
        .collect())
}

fn read_areas(path: &Path) -> Result<Vec<AreaRec>, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let parsed: AreaStorage = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let areas = match parsed.data {
        AreaData::Wrapped { areas } => areas,
        AreaData::Flat(areas) => areas,
    };
    Ok(areas
        .into_iter()
        .map(|a| {
            let area_id = a.id.or(a.area_id).unwrap_or_else(|| fold_umlaut(&a.name));
            AreaRec { area_id: area_id.clone(), name: a.name.clone(), aliases: merge_area_aliases(&area_id, &a.name, a.aliases) }
        })
        .collect())
}

fn keep_domain(entity_id: &str) -> bool {
    matches!(
        entity_id.split('.').next(),
        Some("light")
            | Some("switch")
            | Some("climate")
            | Some("fan")
            | Some("cover")
            | Some("scene")
            | Some("script")
            | Some("vacuum")
            | Some("media_player")
            | Some("lock")
            | Some("timer")
            | Some("todo")
    )
}

#[derive(Deserialize)]
struct HomeFile {
    #[serde(default)]
    areas: Vec<YamlArea>,
    #[serde(default)]
    devices: Vec<YamlDevice>,
    #[serde(default)]
    scenes: Vec<YamlScene>,
    #[serde(default)]
    scripts: Vec<YamlScript>,
    #[serde(default)]
    timers: Vec<YamlNamed>,
    #[serde(default)]
    lists: Vec<YamlNamed>,
}

#[derive(Deserialize)]
struct YamlArea {
    id: String,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
}

#[derive(Deserialize)]
struct YamlDevice {
    id: String,
    #[serde(default)]
    area_id: Option<String>,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
}

#[derive(Deserialize)]
struct YamlNamed {
    id: String,
    name: String,
}

#[derive(Deserialize)]
struct YamlScene {
    id: String,
    name: String,
    #[serde(default)]
    entities: serde_yaml::Value,
}

#[derive(Deserialize)]
struct YamlScript {
    id: String,
    name: String,
    #[serde(default)]
    actions: serde_yaml::Value,
}

/// Load a `home_config.yaml` (areas + devices + scenes).
pub fn load_home_config(path: &Path) -> Result<HomeGraph, String> {
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let file: HomeFile = serde_yaml::from_str(&raw).map_err(|e| e.to_string())?;
    let areas = file
        .areas
        .into_iter()
        .map(|a| AreaRec { area_id: a.id.clone(), name: a.name.clone(), aliases: merge_area_aliases(&a.id, &a.name, a.aliases) })
        .collect();
    let mut entities: Vec<EntityRec> = file
        .devices
        .into_iter()
        .map(|d| {
            let domain = d.id.split('.').next().unwrap_or("").to_string();
            let mut aliases = d.aliases;
            aliases.push(fold_umlaut(&d.name));
            for part in d.name.split_whitespace() {
                let p = fold_umlaut(part);
                if p.len() > 2 && !GENERIC_NAME.contains(&p.as_str()) {
                    aliases.push(p);
                }
            }
            aliases.extend(extra_device_aliases(&d.id, &d.name, &domain));
            EntityRec { entity_id: d.id, name: d.name, domain, area: d.area_id, aliases, tags: Vec::new() }
        })
        .collect();
    let mut scene_members: HashMap<String, Vec<String>> = HashMap::new();
    for item in file.scenes {
        let id = if item.id.contains('.') { item.id.clone() } else { format!("scene.{}", item.id) };
        scene_members.insert(id.clone(), yaml_entity_ids(&item.entities));
        entities.push(EntityRec {
            entity_id: id,
            name: item.name.clone(),
            domain: "scene".into(),
            area: None,
            aliases: vec![fold_umlaut(&item.name)],
            tags: Vec::new(),
        });
    }
    for item in file.scripts {
        let id = if item.id.contains('.') { item.id.clone() } else { format!("script.{}", item.id) };
        scene_members.insert(id.clone(), yaml_entity_ids(&item.actions));
        entities.push(EntityRec {
            entity_id: id,
            name: item.name.clone(),
            domain: "script".into(),
            area: None,
            aliases: vec![fold_umlaut(&item.name)],
            tags: Vec::new(),
        });
    }
    for (domain, items) in [("timer", file.timers), ("todo", file.lists)] {
        for item in items {
            let id = if item.id.contains('.') { item.id.clone() } else { format!("{domain}.{}", item.id) };
            entities.push(EntityRec {
                entity_id: id,
                name: item.name.clone(),
                domain: domain.to_string(),
                area: None,
                aliases: vec![fold_umlaut(&item.name)],
                tags: Vec::new(),
            });
        }
    }
    Ok(HomeGraph { entities, areas, scene_members, assist: None })
}

fn yaml_entity_ids(value: &serde_yaml::Value) -> Vec<String> {
    let mut out = Vec::new();
    collect_entity_ids(value, &mut out);
    out.sort();
    out.dedup();
    out
}

fn collect_entity_ids(value: &serde_yaml::Value, out: &mut Vec<String>) {
    match value {
        serde_yaml::Value::String(s) if s.contains('.') => out.push(s.clone()),
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map {
                if let serde_yaml::Value::String(key) = k {
                    if key.contains('.') {
                        out.push(key.clone());
                    }
                    if key == "entity_id" {
                        collect_entity_ids(v, out);
                    }
                }
                collect_entity_ids(v, out);
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for v in seq {
                collect_entity_ids(v, out);
            }
        }
        _ => {}
    }
}

const GENERIC_NAME: &[&str] = &[
    "light",
    "lights",
    "room",
    "floor",
    "the",
    "and",
    "licht",
    "zimmer",
    "bedroom",
    "bedrooms",
    "kinderzimmer",
    "bath",
    "bathroom",
    "main",
    "ceiling",
    "master",
    "timer",
    "living",
    "dining",
    "kitchen",
    "family",
    "laundry",
    "hallway",
    "entryway",
    "powder",
    "ground",
    "upper",
];

fn merge_area_aliases(area_id: &str, name: &str, existing: Vec<String>) -> Vec<String> {
    let mut aliases = existing;
    aliases.push(area_id.to_string());
    aliases.push(fold_umlaut(name));
    for part in name.split_whitespace() {
        let p = fold_umlaut(part);
        if p.len() > 3 && !GENERIC_NAME.contains(&p.as_str()) {
            aliases.push(p);
        }
    }
    aliases.extend(extra_area_aliases(area_id));
    aliases.sort();
    aliases.dedup();
    aliases
}

fn extra_area_aliases(id: &str) -> Vec<String> {
    match id {
        "entryway" => vec!["foyer".into(), "entrance".into(), "eingang".into(), "diele".into()],
        "living" | "wohnzimmer" => vec!["lounge".into(), "wohnzimmer".into(), "wohnraum".into(), "living".into(), "livingroom".into()],
        "family_room" => vec!["den".into(), "familienzimmer".into(), "family".into()],
        "powder_room" => vec!["powder".into(), "gaestewc".into(), "guestwc".into()],
        "laundry" => vec!["laundryroom".into(), "waschkueche".into()],
        "master_bedroom" | "schlafzimmer" => vec!["elternschlafzimmer".into(), "master".into(), "bedroom".into(), "schlafzimmer".into()],
        "main_bath" | "badezimmer" => vec!["bathroom".into(), "badezimmer".into(), "bad".into(), "bath".into()],
        "hallway" | "flur" => vec!["hall".into(), "corridor".into(), "flur".into(), "hallway".into(), "diele".into()],
        "wohnung" => vec!["ueberall".into(), "everywhere".into(), "all".into(), "home".into(), "house".into(), "apartment".into()],
        "arbeitszimmer" => vec!["office".into(), "study".into(), "buero".into(), "arbeitszimmer".into()],
        "esszimmer" => vec!["dining".into(), "diningroom".into(), "esszimmer".into()],
        "kuche" | "kueche" => vec!["kitchen".into(), "kuche".into(), "kueche".into()],
        "balkon" => vec!["balcony".into(), "terrace".into(), "balkon".into(), "terrasse".into()],
        _ => Vec::new(),
    }
}

fn extra_device_aliases(id: &str, name: &str, domain: &str) -> Vec<String> {
    let folded = fold_umlaut(name);
    let mut extra = Vec::new();
    if id.contains("alle") || folded.contains("alle lichter") || folded.starts_with("all light") {
        extra.extend(["alle".into(), "all".into(), "ueberall".into(), "everywhere".into()]);
    }
    if folded == "deckenlampe" || id.contains("deckenlampe") {
        extra.extend(["decke".into(), "deckenlampe".into()]);
    }
    if folded.contains("klima") {
        extra.extend(["klima".into(), "ac".into(), "klimaanlage".into()]);
    }
    if folded.split_whitespace().any(|p| p == "pc") || id.contains("pc") {
        extra.push("pc".into());
    }
    if domain == "vacuum" {
        extra.extend(["staubsauger".into(), "sauger".into(), "saugroboter".into(), "vacuum".into()]);
    }
    if domain == "lock" {
        extra.push("schloss".into());
        if id.contains("front") || folded.contains("front") || folded.contains("haustuer") {
            extra.push("haustuer".into());
        }
        if id.contains("garage") || folded.contains("garage") {
            extra.extend(["garagentuer".into(), "garageneingang".into()]);
        }
    }
    if id.contains("dryer") || folded == "dryer" || folded == "trockner" {
        extra.extend(["dryer".into(), "trockner".into()]);
    }
    if id.contains("washing") || folded.contains("waschmaschine") || folded.contains("washing") {
        extra.extend(["washer".into(), "waschmaschine".into(), "washing".into()]);
    }
    extra
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::default_home;

    #[test]
    fn german_rooms_get_english_aliases() {
        let office = merge_area_aliases("arbeitszimmer", "Arbeitszimmer", vec![]);
        assert!(office.iter().any(|a| a == "office"), "{office:?}");
        assert!(office.iter().any(|a| a == "study"), "{office:?}");
        let living = merge_area_aliases("wohnzimmer", "Wohnzimmer", vec![]);
        assert!(living.iter().any(|a| a == "living"), "{living:?}");
        let bed = merge_area_aliases("schlafzimmer", "Schlafzimmer", vec![]);
        assert!(bed.iter().any(|a| a == "bedroom"), "{bed:?}");
    }

    #[test]
    fn hue_ids_inherit_room_from_entity_id() {
        let areas = default_home().areas;
        assert_eq!(infer_area("light.schlafzimmer", &areas).as_deref(), Some("schlafzimmer"));
        assert_eq!(infer_area("light.schlafzimmer_ambilight", &areas).as_deref(), Some("schlafzimmer"));
        assert_eq!(infer_area("light.hue_color_lamp_2", &areas), None);
    }
}
