use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRec {
    pub entity_id: String,
    pub name: String,
    pub domain: String,
    #[serde(default)]
    pub platform: Option<String>,
    pub area: Option<String>,
    pub aliases: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorRec {
    pub floor_id: String,
    pub name: String,
    pub aliases: Vec<String>,
    #[serde(default)]
    pub level: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaRec {
    pub area_id: String,
    pub name: String,
    pub aliases: Vec<String>,
    #[serde(default)]
    pub floor_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HomePolicy {
    #[serde(default)]
    pub infra_id: Vec<String>,
    #[serde(default)]
    pub infra_name: Vec<String>,
    #[serde(default)]
    pub timer_hints: HashMap<i32, String>,
    #[serde(default)]
    pub preferred_climate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HomeGraph {
    pub entities: Vec<EntityRec>,
    pub areas: Vec<AreaRec>,
    #[serde(default)]
    pub floors: Vec<FloorRec>,
    #[serde(default)]
    pub scene_members: HashMap<String, Vec<String>>,
    /// `None` means no Assist expose list is available. `Some` limits the visible IDs.
    #[serde(default)]
    pub assist: Option<HashSet<String>>,
    #[serde(default)]
    pub policy: HomePolicy,
}

impl HomeGraph {
    pub fn areas_on_floor<'a>(&'a self, floor_id: &'a str) -> impl Iterator<Item = &'a AreaRec> + 'a {
        self.areas.iter().filter(move |area| area.floor_id.as_deref() == Some(floor_id))
    }

    pub fn floor(&self, floor_id: &str) -> Option<&FloorRec> {
        self.floors.iter().find(|floor| floor.floor_id == floor_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSentence {
    pub phrase: String,
    pub intent: String,
    pub slots: HashMap<String, String>,
}
