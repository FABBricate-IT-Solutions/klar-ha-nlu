//! Building test homes and reading back what Home Assistant would do.
use std::collections::{BTreeSet, HashSet};

use klar_nlu::nlu::parse_with_policies;
use klar_nlu::session::Session;
use klar_nlu::types::{AreaRec, EntityRec, HomeGraph, Intent, ParseOutcome, PolicyRule, Settings, SpeechBank};

pub fn area(id: &str, name: &str) -> AreaRec {
    AreaRec { area_id: id.into(), name: name.into(), aliases: vec![], floor_id: None }
}

pub fn ent(id: &str, name: &str, area: &str) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: id.split_once('.').map(|(domain, _)| domain).unwrap_or("").into(),
        platform: None,
        area: (!area.is_empty()).then(|| area.into()),
        aliases: vec![],
        tags: vec![],
    }
}

/// Exposes everything to Assist except `hidden`, as a tidy real home does.
pub fn home(areas: Vec<AreaRec>, entities: Vec<EntityRec>, hidden: &[&str]) -> HomeGraph {
    let assist: HashSet<String> = entities.iter().map(|e| e.entity_id.clone()).filter(|id| !hidden.contains(&id.as_str())).collect();
    HomeGraph { areas, entities, assist: Some(assist), ..HomeGraph::default() }
}

fn domain_of(intent: &Intent) -> &str {
    intent.slot("domain").or_else(|| intent.slot("entity_id").and_then(|id| id.split_once('.').map(|(d, _)| d))).unwrap_or("")
}

/// "Intent entity_id value" for every device Home Assistant would act on. Room-wide steps
/// reach every exposed device of that domain in the room, as Home Assistant's own intents do.
pub fn effects(out: &ParseOutcome, home: &HomeGraph) -> BTreeSet<String> {
    let exposed = |e: &EntityRec| home.assist.as_ref().is_none_or(|ids| ids.contains(&e.entity_id));
    let mut set = BTreeSet::new();
    for intent in out.plan.as_ref().map(|plan| plan.intents()).unwrap_or_default() {
        let value = ["brightness", "color", "position", "temperature", "percentage", "volume_level", "volume_step"]
            .iter()
            .find_map(|key| intent.slot(key).map(|v| format!(" {key}={v}")))
            .unwrap_or_default();
        let targets = home.entities.iter().filter(|e| match intent.slot("entity_id") {
            Some(id) => e.entity_id == id,
            None => exposed(e) && e.area.as_deref() == intent.slot("area") && e.domain == domain_of(&intent),
        });
        for target in targets {
            set.insert(format!("{} {}{}", intent.name, target.entity_id, value));
        }
    }
    set
}

pub fn describe(out: &ParseOutcome) -> String {
    let plan: Vec<String> = out
        .plan
        .as_ref()
        .map(|plan| plan.intents())
        .unwrap_or_default()
        .iter()
        .map(|i| format!("{}{:?}", i.name, i.slots.iter().map(|s| format!("{}={}", s.name, s.value)).collect::<Vec<_>>()))
        .collect();
    format!("{:?} {plan:?}", out.decision)
}

/// One conversation: the same session across turns, optionally with user policy rules.
pub struct Talk<'a> {
    pub home: &'a HomeGraph,
    pub session: Session,
    pub settings: Settings,
    pub rules: Vec<PolicyRule>,
}

impl<'a> Talk<'a> {
    pub fn new(lang: &str, home: &'a HomeGraph) -> Self {
        Self { home, session: Session::new(), settings: Settings::pinned(lang), rules: Vec::new() }
    }

    pub fn without_confirm(mut self) -> Self {
        self.settings.confirm_risky_actions = false;
        self
    }

    pub fn say(&mut self, text: &str) -> ParseOutcome {
        parse_with_policies(text, self.home, &mut self.session, &[], &self.settings, &self.rules, &SpeechBank::default())
    }
}

pub fn run(lang: &str, home: &HomeGraph, text: &str) -> ParseOutcome {
    Talk::new(lang, home).without_confirm().say(text)
}
