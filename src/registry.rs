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
    let label_names = crate::roles::load_label_names(&config_dir.join(".storage/core.label_registry"));
    let mut entities = match read_entities(&entity_path, &device_areas, &label_names) {
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
        if (slug == area.area_id || slug.starts_with(&format!("{}_", area.area_id)))
            && best.is_none_or(|cur| area.area_id.len() > cur.len())
        {
            best = Some(area.area_id.as_str());
        }
    }
    best.map(str::to_string)
}

fn read_entities(
    path: &Path,
    device_areas: &HashMap<String, String>,
    label_names: &HashMap<String, String>,
) -> Result<Vec<EntityRec>, String> {
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
            EntityRec {
                entity_id: e.entity_id,
                name,
                domain,
                area,
                aliases: e.aliases,
                tags: crate::roles::expand_entity_tags(e.labels, label_names),
            }
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

pub use crate::registry_yaml::load_home_config;
use crate::registry_yaml::merge_area_aliases;

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
