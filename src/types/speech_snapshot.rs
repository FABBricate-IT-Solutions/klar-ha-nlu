//! HA post-execute snapshot. Engine interpolates; Python builds from live state.

use super::UnitSystem;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SNAPSHOT_SCHEMA: &str = "1";
pub const MAX_SNAPSHOT_ENTITIES: usize = 32;
pub const MAX_SNAPSHOT_EVENTS: usize = 16;
pub const MAX_SNAPSHOT_QUEUE: usize = 8;
pub const MAX_ATTR_CHARS: usize = 256;
pub const MAX_NAME_CHARS: usize = 128;

const ALLOWED_ATTRS: &[&str] = &[
    "current_temperature",
    "temperature",
    "temperature_unit",
    "unit_of_measurement",
    "hvac_action",
    "hvac_mode",
    "volume_level",
    "is_volume_muted",
    "media_title",
    "media_artist",
    "media_album_name",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeechSnapshot {
    #[serde(default)]
    pub schema_version: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub unit_system: UnitSystem,
    #[serde(default)]
    pub now: String,
    pub intent: SpeechIntent,
    pub outcome: String,
    #[serde(default)]
    pub entities: Vec<SpeechEntity>,
    #[serde(default)]
    pub calendar_events: Vec<SpeechCalendarEvent>,
    #[serde(default)]
    pub media_queue: Vec<SpeechQueueItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeechIntent {
    pub name: String,
    #[serde(default)]
    pub slots: Vec<SpeechSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeechSlot {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpeechEntity {
    pub entity_id: String,
    pub name: String,
    pub domain: String,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_class: Option<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeechCalendarEvent {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub start: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeechQueueItem {
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeechRenderOut {
    pub speech: String,
    pub quiet_ack: bool,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotError {
    Schema,
    Outcome,
    Language,
    Now,
}

impl SpeechSnapshot {
    pub fn sanitize(mut self) -> Result<Self, SnapshotError> {
        if self.schema_version != SNAPSHOT_SCHEMA {
            return Err(SnapshotError::Schema);
        }
        if !matches!(self.outcome.as_str(), "success" | "partial" | "error") {
            return Err(SnapshotError::Outcome);
        }
        if self.language.trim().is_empty() || self.language.chars().count() > 32 || self.language.chars().any(char::is_control) {
            return Err(SnapshotError::Language);
        }
        if self.now.trim().is_empty() || self.now.chars().count() > 64 {
            return Err(SnapshotError::Now);
        }
        truncate_chars(&mut self.personality, 32);
        truncate_chars(&mut self.intent.name, 64);
        self.intent.slots.truncate(16);
        for slot in &mut self.intent.slots {
            truncate_chars(&mut slot.name, 64);
            truncate_chars(&mut slot.value, MAX_NAME_CHARS);
        }
        self.entities.truncate(MAX_SNAPSHOT_ENTITIES);
        for entity in &mut self.entities {
            truncate_chars(&mut entity.entity_id, MAX_NAME_CHARS);
            truncate_chars(&mut entity.name, MAX_NAME_CHARS);
            truncate_chars(&mut entity.domain, 32);
            truncate_chars(&mut entity.state, 64);
            if let Some(area) = &mut entity.area {
                truncate_chars(area, MAX_NAME_CHARS);
            }
            if let Some(area_name) = &mut entity.area_name {
                truncate_chars(area_name, MAX_NAME_CHARS);
            }
            entity.attributes.retain(|key, _| ALLOWED_ATTRS.contains(&key.as_str()));
            for value in entity.attributes.values_mut() {
                cap_attr(value);
            }
        }
        self.calendar_events.truncate(MAX_SNAPSHOT_EVENTS);
        for event in &mut self.calendar_events {
            truncate_chars(&mut event.summary, MAX_NAME_CHARS);
            truncate_chars(&mut event.start, 64);
        }
        self.media_queue.truncate(MAX_SNAPSHOT_QUEUE);
        for item in &mut self.media_queue {
            truncate_chars(&mut item.title, MAX_NAME_CHARS);
        }
        Ok(self)
    }

    pub fn unrendered(self) -> SpeechRenderOut {
        SpeechRenderOut { speech: String::new(), quiet_ack: false, source: "unrendered" }
    }
}

fn cap_attr(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(text) => truncate_chars(text, MAX_ATTR_CHARS),
        serde_json::Value::Number(_) | serde_json::Value::Bool(_) | serde_json::Value::Null => {}
        other => *other = serde_json::Value::Null,
    }
}

fn truncate_chars(value: &mut String, max: usize) {
    if value.chars().count() <= max {
        return;
    }
    *value = value.chars().take(max).collect();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> SpeechSnapshot {
        SpeechSnapshot {
            schema_version: "1".into(),
            language: "de".into(),
            personality: "default".into(),
            unit_system: UnitSystem::Metric,
            now: "2026-09-05T19:22:00+02:00".into(),
            intent: SpeechIntent { name: "HassTurnOn".into(), slots: vec![SpeechSlot { name: "area".into(), value: "wohnzimmer".into() }] },
            outcome: "success".into(),
            entities: vec![SpeechEntity {
                entity_id: "light.wohnzimmer".into(),
                name: "Wohnzimmer".into(),
                domain: "light".into(),
                state: "on".into(),
                area: Some("wohnzimmer".into()),
                area_name: Some("Wohnzimmer".into()),
                device_class: None,
                attributes: BTreeMap::from([
                    ("media_title".into(), serde_json::Value::String("ok".into())),
                    ("secret".into(), serde_json::Value::String("drop-me".into())),
                ]),
            }],
            calendar_events: Vec::new(),
            media_queue: Vec::new(),
        }
    }

    #[test]
    fn drops_unknown_attributes_and_rejects_bad_schema() {
        let clean = sample().sanitize().unwrap();
        assert!(clean.entities[0].attributes.contains_key("media_title"));
        assert!(!clean.entities[0].attributes.contains_key("secret"));
        assert_eq!(clean.unrendered().source, "unrendered");
        let mut bad = sample();
        bad.schema_version = "2".into();
        assert_eq!(bad.sanitize(), Err(SnapshotError::Schema));
        let mut missing_now = sample();
        missing_now.now.clear();
        assert_eq!(missing_now.sanitize(), Err(SnapshotError::Now));
        let missing_schema: SpeechSnapshot = serde_json::from_value(serde_json::json!({
            "language": "de",
            "now": "2026-09-05T19:22:00+02:00",
            "intent": {"name": "HassTurnOn"},
            "outcome": "success"
        }))
        .unwrap();
        assert_eq!(missing_schema.sanitize(), Err(SnapshotError::Schema));
        let mut over = sample();
        over.entities = (0..40)
            .map(|i| SpeechEntity {
                entity_id: format!("light.n{i}"),
                name: "n".into(),
                domain: "light".into(),
                state: "on".into(),
                area: None,
                area_name: None,
                device_class: None,
                attributes: BTreeMap::new(),
            })
            .collect();
        assert_eq!(over.sanitize().unwrap().entities.len(), MAX_SNAPSHOT_ENTITIES);
        let mut bad_outcome = sample();
        bad_outcome.outcome = "ok".into();
        assert_eq!(bad_outcome.sanitize(), Err(SnapshotError::Outcome));
    }
}
