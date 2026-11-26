use crate::home::paths::{read_to_string_confined, write_atomic_confined};
use crate::home::registry::keep_domain;
use crate::home::registry_yaml::{drop_borrowed_area_names, merge_area_aliases, merge_floor_aliases};
use crate::home::roles::expand_entity_tags;
use crate::types::{AreaRec, EntityRec, FloorRec, HomeGraph};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub const HOME_SNAPSHOT_FILE: &str = "home_snapshot.json";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// The user's device class, or the integration's when the user set none.
    #[serde(default)]
    pub device_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotArea {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub floor_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotFloor {
    pub floor_id: String,
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub level: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotLabel {
    pub label_id: String,
    pub name: String,
}

pub fn ingest(mut snapshot: HomeSnapshot) -> Result<HomeGraph, SnapshotError> {
    if snapshot.schema_version != HOME_SCHEMA_VERSION {
        return Err(SnapshotError::Schema);
    }
    // D-15: missing assist must not stay `None` (that frees everything, including entities
    // added later). An omitted list means "every entity kept in this snapshot"; an explicit
    // empty list means nothing is exposed.
    let assist_omitted = snapshot.assist.is_none();
    // R-10: sanitize (clip aliases, reject bad ids), filter domains/Assist, then cap.
    let assist_ids: HashSet<String> =
        snapshot.assist.as_ref().map(|ids| ids.iter().filter(|id| id.contains('.')).cloned().collect()).unwrap_or_default();
    snapshot.entities = snapshot
        .entities
        .into_iter()
        .map(sanitize_entity)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|entity| !entity.disabled && (keep_domain(&entity.entity_id) || assist_ids.contains(&entity.entity_id)))
        .take(MAX_SNAPSHOT_ENTITIES)
        .collect();
    // D-15: an empty push must not wipe a running home.
    if snapshot.entities.is_empty() {
        return Err(SnapshotError::Malformed("empty home snapshot"));
    }
    snapshot.devices = snapshot.devices.into_iter().take(MAX_SNAPSHOT_DEVICES).map(sanitize_device).collect::<Result<Vec<_>, _>>()?;
    snapshot.areas = snapshot.areas.into_iter().take(MAX_SNAPSHOT_AREAS).map(sanitize_area).collect::<Result<Vec<_>, _>>()?;
    snapshot.floors = snapshot.floors.into_iter().take(MAX_SNAPSHOT_FLOORS).map(sanitize_floor).collect::<Result<Vec<_>, _>>()?;
    snapshot.labels = snapshot.labels.into_iter().take(MAX_SNAPSHOT_LABELS).map(sanitize_label).collect::<Result<Vec<_>, _>>()?;
    reject_duplicate_ids(&snapshot)?;
    if let Some(assist) = snapshot.assist.as_mut() {
        *assist = std::mem::take(assist).into_iter().filter(|id| id.contains('.')).take(MAX_SNAPSHOT_ASSIST).collect();
        for entity_id in assist.iter() {
            valid_entity_id(entity_id)?;
        }
    } else if assist_omitted {
        snapshot.assist = Some(snapshot.entities.iter().map(|entity| entity.entity_id.clone()).take(MAX_SNAPSHOT_ASSIST).collect());
    }
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
    let mut areas = snapshot
        .areas
        .into_iter()
        .map(|area| AreaRec {
            area_id: area.id.clone(),
            name: area.name.clone(),
            aliases: merge_area_aliases(&area.id, &area.name, area.aliases),
            floor_id: area.floor_id.filter(|id| floor_ids.contains(id.as_str())),
        })
        .collect::<Vec<_>>();
    drop_borrowed_area_names(&mut areas);
    let area_ids = areas.iter().map(|area| area.area_id.as_str()).collect::<HashSet<_>>();
    let device_classes = snapshot
        .entities
        .iter()
        .filter_map(|entity| Some((entity.entity_id.clone(), clean(entity.device_class.as_deref())?.to_lowercase())))
        .collect();
    let entities = snapshot
        .entities
        .into_iter()
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
    let assist = snapshot.assist.map(|ids| ids.into_iter().collect());
    let registered_intents = snapshot.registered_intents.into_iter().filter(|name| !name.is_empty() && name.len() <= 64).take(64).collect();
    let mut graph = HomeGraph { entities, areas, floors, assist, registered_intents, device_classes, ..HomeGraph::default() };
    crate::home::labels::rebuild_label_index(&mut graph);
    Ok(graph)
}

/// Persist the last accepted Home Assistant push so a restart does not fall back to an empty
/// or demo home while live sync is offline (D-17).
pub fn save_home_snapshot(dir: &Path, snapshot: &HomeSnapshot) -> std::io::Result<()> {
    let bytes = serde_json::to_vec(snapshot).map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
    write_atomic_confined(dir, HOME_SNAPSHOT_FILE, &bytes)
}

pub fn load_home_snapshot(dir: &Path) -> Option<HomeGraph> {
    let raw = match read_to_string_confined(dir, HOME_SNAPSHOT_FILE) {
        Ok(raw) => raw,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return None,
        Err(err) => {
            tracing::warn!("{HOME_SNAPSHOT_FILE} nicht lesbar: {err}");
            return None;
        }
    };
    let snapshot: HomeSnapshot = match serde_json::from_str(&raw) {
        Ok(snapshot) => snapshot,
        Err(err) => {
            tracing::warn!("{HOME_SNAPSHOT_FILE} ungültig: {err}");
            return None;
        }
    };
    match ingest(snapshot) {
        Ok(graph) => Some(graph),
        Err(err) => {
            tracing::warn!("{HOME_SNAPSHOT_FILE} abgelehnt: {}", err.as_str());
            None
        }
    }
}

fn reject_duplicate_ids(snapshot: &HomeSnapshot) -> Result<(), SnapshotError> {
    let mut entities = HashSet::new();
    for entity in &snapshot.entities {
        if !entities.insert(entity.entity_id.as_str()) {
            return Err(SnapshotError::Malformed("duplicate entity_id"));
        }
    }
    let mut devices = HashSet::new();
    for device in &snapshot.devices {
        if !devices.insert(device.id.as_str()) {
            return Err(SnapshotError::Malformed("duplicate device id"));
        }
    }
    let mut areas = HashSet::new();
    for area in &snapshot.areas {
        if !areas.insert(area.id.as_str()) {
            return Err(SnapshotError::Malformed("duplicate area id"));
        }
    }
    let mut floors = HashSet::new();
    for floor in &snapshot.floors {
        if !floors.insert(floor.floor_id.as_str()) {
            return Err(SnapshotError::Malformed("duplicate floor id"));
        }
    }
    let mut labels = HashSet::new();
    for label in &snapshot.labels {
        if !labels.insert(label.label_id.as_str()) {
            return Err(SnapshotError::Malformed("duplicate label id"));
        }
    }
    Ok(())
}

fn sanitize_entity(mut entity: SnapshotEntity) -> Result<SnapshotEntity, SnapshotError> {
    valid_entity_id(&entity.entity_id)?;
    optional_name(entity.name.as_deref(), "entity name")?;
    optional_name(entity.original_name.as_deref(), "entity original_name")?;
    optional_id(entity.area_id.as_deref(), "entity area_id")?;
    optional_id(entity.device_id.as_deref(), "entity device_id")?;
    optional_name(entity.platform.as_deref(), "entity platform")?;
    optional_id(entity.device_class.as_deref(), "entity device_class")?;
    entity.aliases = sanitize_aliases(entity.aliases);
    entity.labels = sanitize_aliases(entity.labels);
    Ok(entity)
}

fn sanitize_device(device: SnapshotDevice) -> Result<SnapshotDevice, SnapshotError> {
    valid_id(&device.id, "device id")?;
    optional_name(device.name.as_deref(), "device name")?;
    optional_name(device.name_by_user.as_deref(), "device name_by_user")?;
    optional_id(device.area_id.as_deref(), "device area_id")?;
    Ok(device)
}

fn sanitize_area(mut area: SnapshotArea) -> Result<SnapshotArea, SnapshotError> {
    valid_id(&area.id, "area id")?;
    valid_name(&area.name, "area name")?;
    optional_id(area.floor_id.as_deref(), "area floor_id")?;
    area.aliases = sanitize_aliases(area.aliases);
    Ok(area)
}

fn sanitize_floor(mut floor: SnapshotFloor) -> Result<SnapshotFloor, SnapshotError> {
    valid_id(&floor.floor_id, "floor id")?;
    valid_name(&floor.name, "floor name")?;
    floor.aliases = sanitize_aliases(floor.aliases);
    Ok(floor)
}

fn sanitize_label(label: SnapshotLabel) -> Result<SnapshotLabel, SnapshotError> {
    valid_id(&label.label_id, "label id")?;
    valid_name(&label.name, "label name")?;
    Ok(label)
}

fn sanitize_aliases(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .filter_map(|value| {
            let trimmed: String = value.chars().filter(|ch| !ch.is_control()).collect();
            let trimmed = trimmed.trim();
            if trimmed.is_empty() {
                return None;
            }
            let clipped: String = trimmed.chars().take(MAX_ALIAS_CHARS).collect();
            (!clipped.is_empty()).then_some(clipped)
        })
        .take(MAX_SNAPSHOT_ALIASES)
        .collect()
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

#[cfg(test)]
#[path = "snapshot_tests.rs"]
mod tests;
