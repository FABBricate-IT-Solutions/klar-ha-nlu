use crate::parse::normalize::fold_umlaut;
use crate::types::{AreaRec, EntityRec, FloorRec, HomeGraph, HomePolicy};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct HomeFile {
    #[serde(default)]
    floors: Vec<YamlFloor>,
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
    #[serde(default)]
    policy: HomePolicy,
}

#[derive(Deserialize)]
struct YamlFloor {
    id: String,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    level: Option<i32>,
}

#[derive(Deserialize)]
struct YamlArea {
    id: String,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default, alias = "floor")]
    floor_id: Option<String>,
}

#[derive(Deserialize)]
struct YamlDevice {
    id: String,
    #[serde(default)]
    area_id: Option<String>,
    name: String,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    platform: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
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
    let floors: Vec<FloorRec> = file
        .floors
        .into_iter()
        .map(|floor| FloorRec {
            floor_id: floor.id.clone(),
            name: floor.name.clone(),
            aliases: merge_floor_aliases(&floor.id, &floor.name, floor.level, floor.aliases),
            level: floor.level,
        })
        .collect();
    let areas = file
        .areas
        .into_iter()
        .map(|a| AreaRec {
            area_id: a.id.clone(),
            name: a.name.clone(),
            aliases: merge_area_aliases(&a.id, &a.name, a.aliases),
            floor_id: a.floor_id.filter(|id| !id.is_empty()),
        })
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
            EntityRec { entity_id: d.id, name: d.name, domain, platform: d.platform, area: d.area_id, aliases, tags: d.tags }
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
            platform: None,
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
            platform: None,
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
                platform: None,
                area: None,
                aliases: vec![fold_umlaut(&item.name)],
                tags: Vec::new(),
            });
        }
    }
    Ok(HomeGraph { entities, areas, floors, scene_members, assist: None, policy: file.policy, registered_intents: Vec::new() })
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
    "und",
    "or",
    "oder",
    "auch",
    "too",
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

pub(crate) fn merge_floor_aliases(floor_id: &str, name: &str, level: Option<i32>, existing: Vec<String>) -> Vec<String> {
    let mut aliases = existing;
    aliases.push(floor_id.to_string());
    aliases.push(fold_umlaut(name));
    aliases.extend(extra_floor_aliases(floor_id, name, level));
    for part in name.split_whitespace() {
        let p = fold_umlaut(part);
        if p.len() > 3 && !GENERIC_NAME.contains(&p.as_str()) {
            aliases.push(p);
        }
    }
    aliases.sort();
    aliases.dedup();
    aliases
}

fn extra_floor_aliases(floor_id: &str, name: &str, level: Option<i32>) -> Vec<String> {
    let blob = format!("{} {}", fold_umlaut(floor_id), fold_umlaut(name));
    let mut extra = Vec::new();
    if level == Some(0) || blob.contains("ground") || blob.contains("erdgeschoss") || blob.contains("eg") {
        extra.extend(["erdgeschoss".into(), "downstairs".into(), "groundfloor".into(), "ground".into(), "eg".into()]);
    }
    if level == Some(1) || blob.contains("upper") || blob.contains("obergeschoss") || blob.contains("upstairs") || blob.contains("og") {
        extra.extend(["obergeschoss".into(), "upstairs".into(), "upperfloor".into(), "upper".into(), "og".into()]);
    }
    if level.is_some_and(|value| value < 0) || blob.contains("keller") || blob.contains("basement") || blob.contains("untergeschoss") {
        extra.extend(["keller".into(), "basement".into(), "untergeschoss".into()]);
    }
    extra
}

pub(crate) fn merge_area_aliases(area_id: &str, name: &str, existing: Vec<String>) -> Vec<String> {
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
        "arbeitszimmer" | "office" => vec!["office".into(), "study".into(), "buero".into(), "arbeitszimmer".into()],
        "garden" | "garten" => vec!["garden".into(), "garten".into(), "yard".into(), "balkon".into()],
        "basement" | "keller" => vec!["basement".into(), "keller".into(), "cellar".into()],
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
            extra.extend(["garagentuer".into(), "garagentor".into(), "garageneingang".into()]);
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
    use std::path::PathBuf;

    #[test]
    fn yaml_keeps_music_assistant_platform() {
        let dir = tempfile();
        let path = dir.join("home.yaml");
        std::fs::write(
            &path,
            "areas:\n  - {id: living, name: Living}\ndevices:\n  - {id: media_player.box, area_id: living, name: Box, platform: music_assistant, tags: [Musik]}\n",
        )
        .expect("write");
        let home = load_home_config(&path).expect("load");
        let player = home.entities.iter().find(|entity| entity.entity_id == "media_player.box").expect("player");
        assert_eq!(player.platform.as_deref(), Some("music_assistant"));
        assert_eq!(player.tags, vec!["Musik"]);
    }

    fn tempfile() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("klar-home-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }
}
