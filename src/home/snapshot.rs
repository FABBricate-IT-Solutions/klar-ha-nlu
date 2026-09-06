use crate::home::registry::keep_domain;
use crate::home::registry_yaml::{merge_area_aliases, merge_floor_aliases};
use crate::home::roles::expand_entity_tags;
use crate::types::{AreaRec, EntityRec, FloorRec, HomeGraph};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

pub const HOME_SCHEMA_VERSION: &str = "1";
pub const MAX_SNAPSHOT_ENTITIES: usize = 4096;
pub const MAX_SNAPSHOT_DEVICES: usize = 2048;
pub const MAX_SNAPSHOT_AREAS: usize = 256;
pub const MAX_SNAPSHOT_FLOORS: usize = 64;
pub const MAX_SNAPSHOT_LABELS: usize = 256;
pub const MAX_SNAPSHOT_ASSIST: usize = 4096;
pub const MAX_SNAPSHOT_ALIASES: usize = 32;
pub const MAX_ID_CHARS: usize = 128;
pub const MAX_NAME_CHARS: usize = 256;
pub const MAX_ALIAS_CHARS: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    Schema,
    Caps,
    Malformed(&'static str),
}

impl SnapshotError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Schema => "unsupported home snapshot schema",
            Self::Caps => "home snapshot exceeds collection caps",
            Self::Malformed(detail) => detail,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HomeSnapshot {
    pub schema_version: String,
    #[serde(default)]
    pub entities: Vec<SnapshotEntity>,
    #[serde(default)]
    pub devices: Vec<SnapshotDevice>,
    #[serde(default)]
    pub areas: Vec<SnapshotArea>,
    #[serde(default)]
    pub floors: Vec<SnapshotFloor>,
    #[serde(default)]
    pub labels: Vec<SnapshotLabel>,
    #[serde(default)]
    pub assist: Option<Vec<String>>,
    #[serde(default)]
    pub registered_intents: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotEntity {
    pub entity_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub original_name: Option<String>,
    #[serde(default)]
    pub has_entity_name: bool,
    #[serde(default)]
    pub area_id: Option<String>,
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub disabled: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotDevice {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub name_by_user: Option<String>,
    #[serde(default)]
    pub area_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotArea {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub floor_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotFloor {
    pub floor_id: String,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub level: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotLabel {
    pub label_id: String,
    pub name: String,
}

pub fn ingest(snapshot: HomeSnapshot) -> Result<HomeGraph, SnapshotError> {
    validate(&snapshot)?;
    let devices = device_names(&snapshot.devices);
    let label_names = snapshot.labels.iter().map(|label| (label.label_id.clone(), label.name.clone())).collect::<HashMap<_, _>>();
    let floors = snapshot
        .floors
        .into_iter()
        .map(|floor| FloorRec {
            aliases: merge_floor_aliases(&floor.floor_id, &floor.name, floor.level, floor.aliases),
            floor_id: floor.floor_id,
            name: floor.name,
            level: floor.level,
        })
        .collect::<Vec<_>>();
    let floor_ids = floors.iter().map(|floor| floor.floor_id.as_str()).collect::<HashSet<_>>();
    let areas = snapshot
        .areas
        .into_iter()
        .map(|area| AreaRec {
            area_id: area.id.clone(),
            name: area.name.clone(),
            aliases: merge_area_aliases(&area.id, &area.name, area.aliases),
            floor_id: area.floor_id.filter(|id| floor_ids.contains(id.as_str())),
        })
        .collect::<Vec<_>>();
    let area_ids = areas.iter().map(|area| area.area_id.as_str()).collect::<HashSet<_>>();
    let assist_ids: HashSet<String> =
        snapshot.assist.as_ref().map(|ids| ids.iter().filter(|id| id.contains('.')).cloned().collect()).unwrap_or_default();
    let entities = snapshot
        .entities
        .into_iter()
        .filter(|entity| !entity.disabled && (keep_domain(&entity.entity_id) || assist_ids.contains(&entity.entity_id)))
        .map(|entity| {
            let domain = entity.entity_id.split('.').next().unwrap_or("").to_string();
            let device = entity.device_id.as_deref().and_then(|id| devices.get(id));
            let area = entity.area_id.filter(|id| area_ids.contains(id.as_str())).or_else(|| device.and_then(|info| info.1.clone()));
            EntityRec {
                name: display_name(
                    &entity.entity_id,
                    entity.name,
                    entity.original_name,
                    entity.has_entity_name,
                    device.map(|info| info.0.as_str()),
                ),
                entity_id: entity.entity_id,
                domain,
                platform: entity.platform,
                area,
                aliases: entity.aliases,
                tags: expand_entity_tags(entity.labels, &label_names),
            }
        })
        .collect();
    let assist = snapshot.assist.map(|ids| ids.into_iter().filter(|id| id.contains('.')).collect());
    let registered_intents = snapshot.registered_intents.into_iter().filter(|name| !name.is_empty() && name.len() <= 64).take(64).collect();
    Ok(HomeGraph { entities, areas, floors, assist, registered_intents, ..HomeGraph::default() })
}

fn validate(snapshot: &HomeSnapshot) -> Result<(), SnapshotError> {
    if snapshot.schema_version != HOME_SCHEMA_VERSION {
        return Err(SnapshotError::Schema);
    }
    if snapshot.entities.len() > MAX_SNAPSHOT_ENTITIES
        || snapshot.devices.len() > MAX_SNAPSHOT_DEVICES
        || snapshot.areas.len() > MAX_SNAPSHOT_AREAS
        || snapshot.floors.len() > MAX_SNAPSHOT_FLOORS
        || snapshot.labels.len() > MAX_SNAPSHOT_LABELS
        || snapshot.assist.as_ref().is_some_and(|ids| ids.len() > MAX_SNAPSHOT_ASSIST)
    {
        return Err(SnapshotError::Caps);
    }
    for entity in &snapshot.entities {
        valid_entity_id(&entity.entity_id)?;
        optional_name(entity.name.as_deref(), "entity name")?;
        optional_name(entity.original_name.as_deref(), "entity original_name")?;
        optional_id(entity.area_id.as_deref(), "entity area_id")?;
        optional_id(entity.device_id.as_deref(), "entity device_id")?;
        optional_name(entity.platform.as_deref(), "entity platform")?;
        valid_aliases(&entity.aliases)?;
        valid_aliases(&entity.labels)?;
    }
    for device in &snapshot.devices {
        valid_id(&device.id, "device id")?;
        optional_name(device.name.as_deref(), "device name")?;
        optional_name(device.name_by_user.as_deref(), "device name_by_user")?;
        optional_id(device.area_id.as_deref(), "device area_id")?;
    }
    for area in &snapshot.areas {
        valid_id(&area.id, "area id")?;
        valid_name(&area.name, "area name")?;
        optional_id(area.floor_id.as_deref(), "area floor_id")?;
        valid_aliases(&area.aliases)?;
    }
    for floor in &snapshot.floors {
        valid_id(&floor.floor_id, "floor id")?;
        valid_name(&floor.name, "floor name")?;
        valid_aliases(&floor.aliases)?;
    }
    for label in &snapshot.labels {
        valid_id(&label.label_id, "label id")?;
        valid_name(&label.name, "label name")?;
    }
    if let Some(assist) = &snapshot.assist {
        for entity_id in assist {
            valid_entity_id(entity_id)?;
        }
    }
    Ok(())
}

fn device_names(devices: &[SnapshotDevice]) -> HashMap<String, (String, Option<String>)> {
    devices
        .iter()
        .filter_map(|device| {
            let name = clean(device.name_by_user.as_deref()).or_else(|| clean(device.name.as_deref()))?;
            Some((device.id.clone(), (name, device.area_id.clone().filter(|id| !id.is_empty()))))
        })
        .collect()
}

fn display_name(entity_id: &str, name: Option<String>, original: Option<String>, has_entity_name: bool, device: Option<&str>) -> String {
    if let Some(name) = clean(name.as_deref()).filter(|value| !looks_like_entity_id(value)) {
        return name;
    }
    let original = clean(original.as_deref()).filter(|value| !looks_like_entity_id(value));
    if has_entity_name || original.is_none() {
        if let Some(device) = device {
            return match original {
                Some(original)
                    if !device.eq_ignore_ascii_case(&original) && !device.to_ascii_lowercase().contains(&original.to_ascii_lowercase()) =>
                {
                    format!("{device} {original}")
                }
                _ => device.to_string(),
            };
        }
    }
    original.unwrap_or_else(|| entity_id.rsplit('.').next().unwrap_or(entity_id).replace('_', " "))
}

fn looks_like_entity_id(value: &str) -> bool {
    value.contains('.') && !value.contains(' ') && value.split('.').count() == 2
}

fn clean(value: Option<&str>) -> Option<String> {
    let cleaned = value?.split_whitespace().collect::<Vec<_>>().join(" ");
    (!cleaned.is_empty()).then_some(cleaned)
}

fn valid_entity_id(value: &str) -> Result<(), SnapshotError> {
    let mut parts = value.split('.');
    if matches!((parts.next(), parts.next(), parts.next()), (Some(domain), Some(object), None) if !domain.is_empty() && !object.is_empty())
        && value.len() <= MAX_ID_CHARS
        && !value.chars().any(char::is_control)
    {
        return Ok(());
    }
    Err(SnapshotError::Malformed("invalid entity_id"))
}

fn valid_id(value: &str, _field: &'static str) -> Result<(), SnapshotError> {
    if value.is_empty() || value.len() > MAX_ID_CHARS || value.chars().any(char::is_control) {
        return Err(SnapshotError::Malformed("invalid id"));
    }
    Ok(())
}

fn valid_name(value: &str, _field: &'static str) -> Result<(), SnapshotError> {
    if value.is_empty() || value.chars().count() > MAX_NAME_CHARS || value.chars().any(|ch| ch.is_control() && !ch.is_whitespace()) {
        return Err(SnapshotError::Malformed("invalid name"));
    }
    Ok(())
}

fn optional_id(value: Option<&str>, field: &'static str) -> Result<(), SnapshotError> {
    value.map(|id| valid_id(id, field)).transpose().map(|_| ())
}

fn optional_name(value: Option<&str>, field: &'static str) -> Result<(), SnapshotError> {
    value.map(|name| valid_name(name, field)).transpose().map(|_| ())
}

fn valid_aliases(values: &[String]) -> Result<(), SnapshotError> {
    if values.len() > MAX_SNAPSHOT_ALIASES {
        return Err(SnapshotError::Caps);
    }
    if values.iter().any(|value| value.is_empty() || value.chars().count() > MAX_ALIAS_CHARS || value.chars().any(char::is_control)) {
        return Err(SnapshotError::Malformed("invalid alias"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> HomeSnapshot {
        HomeSnapshot {
            schema_version: HOME_SCHEMA_VERSION.into(),
            entities: vec![SnapshotEntity {
                entity_id: "light.living".into(),
                name: Some("Living".into()),
                original_name: None,
                has_entity_name: false,
                area_id: Some("living".into()),
                device_id: None,
                platform: Some("hue".into()),
                aliases: vec!["decke".into()],
                labels: vec!["Licht".into()],
                disabled: false,
            }],
            devices: Vec::new(),
            areas: vec![SnapshotArea {
                id: "living".into(),
                name: "Wohnzimmer".into(),
                aliases: vec!["wohnzimmer".into()],
                floor_id: Some("upper".into()),
            }],
            floors: vec![SnapshotFloor { floor_id: "upper".into(), name: "Upper Floor".into(), aliases: Vec::new(), level: Some(1) }],
            labels: vec![SnapshotLabel { label_id: "lbl_1".into(), name: "Licht".into() }],
            assist: Some(vec!["light.living".into()]),
            registered_intents: Vec::new(),
        }
    }

    #[test]
    fn rejects_unknown_snapshot_fields() {
        let err = serde_json::from_value::<HomeSnapshot>(serde_json::json!({
            "schema_version": HOME_SCHEMA_VERSION,
            "entities": [],
            "extra": true
        }))
        .expect_err("unknown field");
        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn ingest_keeps_floors_and_assist() {
        let home = ingest(snapshot()).expect("snapshot");
        assert_eq!(home.floors.len(), 1);
        assert_eq!(home.floors[0].floor_id, "upper");
        assert!(home.floors[0].aliases.iter().any(|alias| alias == "upstairs"));
        assert_eq!(home.areas[0].floor_id.as_deref(), Some("upper"));
        assert_eq!(home.entities[0].entity_id, "light.living");
        assert_eq!(home.assist.as_ref().map(HashSet::len), Some(1));
    }

    #[test]
    fn rejects_bad_schema_and_caps() {
        let mut bad = snapshot();
        bad.schema_version = "9".into();
        assert_eq!(ingest(bad).unwrap_err(), SnapshotError::Schema);
        let mut huge = snapshot();
        huge.entities = (0..=MAX_SNAPSHOT_ENTITIES)
            .map(|index| SnapshotEntity {
                entity_id: format!("light.lamp{index}"),
                name: Some("Lamp".into()),
                original_name: None,
                has_entity_name: false,
                area_id: None,
                device_id: None,
                platform: None,
                aliases: Vec::new(),
                labels: Vec::new(),
                disabled: false,
            })
            .collect();
        assert_eq!(ingest(huge).unwrap_err(), SnapshotError::Caps);
    }

    #[test]
    fn rejects_malformed_ids_without_panic() {
        let mut bad = snapshot();
        bad.entities[0].entity_id = "../etc/passwd".into();
        assert_eq!(ingest(bad).unwrap_err(), SnapshotError::Malformed("invalid entity_id"));
        let mut missing = snapshot();
        missing.floors[0].floor_id.clear();
        assert_eq!(ingest(missing).unwrap_err(), SnapshotError::Malformed("invalid id"));
    }

    #[test]
    fn drops_unknown_floor_links_and_disabled_entities() {
        let mut raw = snapshot();
        raw.areas[0].floor_id = Some("ghost".into());
        raw.entities.push(SnapshotEntity {
            entity_id: "light.hidden".into(),
            name: Some("Hidden".into()),
            original_name: None,
            has_entity_name: false,
            area_id: Some("living".into()),
            device_id: None,
            platform: None,
            aliases: Vec::new(),
            labels: Vec::new(),
            disabled: true,
        });
        let home = ingest(raw).expect("snapshot");
        assert!(home.areas[0].floor_id.is_none());
        assert_eq!(home.entities.len(), 1);
    }

    #[test]
    fn ingest_keeps_weather_and_assist_only_domains() {
        let mut raw = snapshot();
        raw.entities.push(SnapshotEntity {
            entity_id: "weather.openweathermap".into(),
            name: Some("OpenWeatherMap".into()),
            original_name: None,
            has_entity_name: false,
            area_id: None,
            device_id: None,
            platform: Some("openweathermap".into()),
            aliases: vec!["Wetter".into()],
            labels: Vec::new(),
            disabled: false,
        });
        raw.entities.push(SnapshotEntity {
            entity_id: "sensor.openweathermap_temperatur".into(),
            name: Some("Temperatur".into()),
            original_name: None,
            has_entity_name: false,
            area_id: None,
            device_id: None,
            platform: Some("openweathermap".into()),
            aliases: Vec::new(),
            labels: Vec::new(),
            disabled: false,
        });
        raw.entities.push(SnapshotEntity {
            entity_id: "sensor.hidden_temp".into(),
            name: Some("Hidden".into()),
            original_name: None,
            has_entity_name: false,
            area_id: None,
            device_id: None,
            platform: None,
            aliases: Vec::new(),
            labels: Vec::new(),
            disabled: false,
        });
        raw.assist = Some(vec!["light.living".into(), "weather.openweathermap".into(), "sensor.openweathermap_temperatur".into()]);
        let home = ingest(raw).expect("snapshot");
        let ids: HashSet<_> = home.entities.iter().map(|entity| entity.entity_id.as_str()).collect();
        assert!(ids.contains("weather.openweathermap"));
        assert!(ids.contains("sensor.openweathermap_temperatur"));
        assert!(!ids.contains("sensor.hidden_temp"));
    }
}
