use crate::compound::area_slots;
use crate::gaps::assist_visible;
use crate::lang::catalog;
use crate::lexicon::Action;
use crate::parse_help::{color_word, wants_all_lights};
use crate::types::{HomeGraph, Intent};

fn whole_home_lights(home: &HomeGraph) -> Option<&crate::types::EntityRec> {
    home.entities.iter().find(|e| {
        e.domain == "light"
            && (e.entity_id.contains("alle") || e.aliases.iter().any(|a| matches!(a.as_str(), "all" | "alle" | "everywhere" | "ueberall")))
    })
}

pub(crate) fn all_lights_clause(
    tokens: &[String],
    home: &HomeGraph,
    action: Action,
    number: Option<i32>,
    areas: &[String],
) -> Option<ClauseOut> {
    if !wants_all_lights(tokens) {
        return None;
    }
    let home_wide = whole_home_lights(home);
    let rooms: Vec<&String> = areas.iter().filter(|a| home_wide.is_none_or(|e| e.area.as_deref() != Some(a.as_str()))).collect();
    if rooms.is_empty() {
        let e = home_wide?;
        return Some(ClauseOut::Intents(vec![fill_intent(action, tokens, number, Some(&e.entity_id), e.area.as_deref(), Some("light"))]));
    }
    let intents: Vec<Intent> = rooms
        .into_iter()
        .filter_map(|area| {
            let (id, slot, dom) = area_slots(action, area, Some("light"), home, tokens);
            let intent = fill_intent(action, tokens, number, id.as_deref(), slot.as_deref(), dom.as_deref());
            (intent.name != "Unknown").then_some(intent)
        })
        .collect();
    (!intents.is_empty()).then_some(ClauseOut::Intents(intents))
}

pub(crate) fn fill_intent(
    action: Action,
    tokens: &[String],
    number: Option<i32>,
    entity_id: Option<&str>,
    area: Option<&str>,
    domain: Option<&str>,
) -> Intent {
    let mut intent = intent_from_action(action, tokens);
    if let Some(id) = entity_id {
        intent = intent.with("entity_id", id);
    } else if let Some(a) = area {
        intent = intent.with("area", a);
    }
    if entity_id.is_none() {
        if let Some(d) = domain {
            if !matches!(action, Action::TimerStart | Action::TimerAdd | Action::TimerCancel | Action::TimerPause) {
                intent = intent.with("domain", d);
            }
        }
    }
    match action {
        Action::SetLight | Action::On => {
            if entity_id.is_some_and(|id| id.starts_with("switch.")) && matches!(action, Action::SetLight) {
                intent.name = "HassTurnOn".into();
            } else if domain == Some("climate") || entity_id.is_some_and(|id| id.starts_with("climate.")) {
                if let Some(n) = number {
                    intent.name = "HassClimateSetTemperature".into();
                    intent = intent.with("temperature", n.to_string());
                }
            } else if let Some(c) = color_word(tokens) {
                intent.name = "HassLightSet".into();
                intent = intent.with("color", c);
            } else if let Some(n) = number {
                if matches!(action, Action::SetLight)
                    || domain == Some("light")
                    || catalog().any(tokens, &catalog().light_nouns)
                    || catalog().any(tokens, &catalog().ceiling)
                {
                    intent.name = "HassLightSet".into();
                    intent = intent.with("brightness", n.to_string());
                }
            }
        }
        Action::SetTemp => {
            if let Some(n) = number {
                intent = intent.with("temperature", n.to_string());
            }
        }
        Action::FanSpeed => {
            if let Some(n) = number {
                intent = intent.with("percentage", n.to_string());
            }
        }
        Action::CoverSet => {
            if let Some(n) = number {
                intent = intent.with("position", n.to_string());
            }
        }
        Action::TimerStart | Action::TimerAdd => {
            if let Some(n) = number {
                intent = intent.with(timer_unit(tokens), n.to_string());
            }
        }
        Action::ListAdd | Action::ListComplete => {
            if let Some(item) = list_item(tokens) {
                intent = intent.with("item", item);
            }
            if entity_id.is_none_or(|id| !id.starts_with("todo."))
                && (catalog().any(tokens, &catalog().list_nouns) || catalog().any(tokens, &catalog().shopping_names))
            {
                intent = intent.with("name", "shopping_list");
            }
        }
        _ => {}
    }
    intent
}

fn timer_unit(tokens: &[String]) -> &'static str {
    if catalog().any(tokens, &catalog().hours) {
        "hours"
    } else if catalog().any(tokens, &catalog().seconds) {
        "seconds"
    } else {
        "minutes"
    }
}

fn list_item(tokens: &[String]) -> Option<String> {
    let cat = catalog();
    let words: Vec<&str> = tokens
        .iter()
        .map(String::as_str)
        .filter(|t| !cat.list_skip.contains(t) && !cat.list_nouns.contains(t) && !cat.shopping_names.contains(t))
        .collect();
    if words.is_empty() {
        None
    } else {
        Some(words.join(" "))
    }
}

pub(crate) fn intent_from_action(action: Action, tokens: &[String]) -> Intent {
    match action {
        Action::On => Intent::new("HassTurnOn"),
        Action::Off => Intent::new("HassTurnOff"),
        Action::Toggle => Intent::new("HassToggle"),
        Action::SetLight => Intent::new("HassLightSet"),
        Action::SetTemp => Intent::new("HassClimateSetTemperature"),
        Action::GetState => {
            if catalog().any(tokens, &catalog().temp_query) {
                Intent::new("HassGetState").with("device_class", "temperature")
            } else {
                Intent::new("HassGetState")
            }
        }
        Action::MediaPause => Intent::new("HassMediaPause"),
        Action::MediaPlay => Intent::new("HassTurnOn").with("domain", "media_player"),
        Action::MediaNext => Intent::new("HassMediaNext"),
        Action::MediaMute => Intent::new("HassMediaPlayerMute"),
        Action::FanSpeed => Intent::new("HassFanSetSpeed"),
        Action::VacuumStart => Intent::new("HassVacuumStart"),
        Action::VacuumDock => Intent::new("HassVacuumReturnToBase"),
        Action::Scene => Intent::new("HassTurnOn").with("domain", "scene"),
        Action::CoverOpen => Intent::new("HassTurnOn").with("domain", "cover"),
        Action::CoverClose => Intent::new("HassTurnOff").with("domain", "cover"),
        Action::CoverSet => Intent::new("HassSetPosition").with("domain", "cover"),
        Action::Lock => Intent::new("HassTurnOn").with("domain", "lock"),
        Action::Unlock => Intent::new("HassTurnOff").with("domain", "lock"),
        Action::TimerStart => Intent::new("HassStartTimer"),
        Action::TimerAdd => Intent::new("HassIncreaseTimer"),
        Action::TimerCancel => Intent::new("HassCancelTimer"),
        Action::TimerPause => Intent::new("HassPauseTimer"),
        Action::ListAdd => Intent::new("HassListAddItem"),
        Action::ListComplete => Intent::new("HassListCompleteItem"),
        Action::ClarifyWrong => Intent::new("Unknown"),
    }
}

pub(crate) fn intent_with_entity(mut intent: Intent, entity_id: &str) -> Intent {
    if intent.slot("entity_id").is_none() {
        intent = intent.with("entity_id", entity_id);
    }
    intent
}

pub(crate) fn pick_singular_lamp(tokens: &[String], home: &HomeGraph, areas: &[String]) -> Option<String> {
    if !catalog().wants_singular_lamp(tokens) {
        return None;
    }
    let lamps: Vec<&str> = home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| {
            e.domain == "light"
                && e.area.as_ref().is_some_and(|a| areas.contains(a))
                && (e.entity_id.contains("lamp") || e.aliases.iter().any(|a| a.contains("lamp") || a.contains("lampe")))
        })
        .map(|e| e.entity_id.as_str())
        .collect();
    (lamps.len() == 1).then(|| lamps[0].to_string())
}

pub(crate) enum ClauseOut {
    Intents(Vec<Intent>),
    Clarify(Vec<String>, Intent),
}

pub(crate) fn timer_clause(
    tokens: &[String],
    home: &HomeGraph,
    action: Action,
    number: Option<i32>,
    domain: Option<&str>,
) -> Option<ClauseOut> {
    if domain != Some("timer") {
        return None;
    }
    let mut intents: Vec<Intent> = crate::resolve::pick_timers(tokens, home)
        .iter()
        .map(|id| fill_intent(action, tokens, number, Some(id), None, Some("timer")))
        .collect();
    let start = matches!(action, Action::TimerStart | Action::TimerAdd) && number.is_some();
    if intents.is_empty() && (start || matches!(action, Action::TimerCancel | Action::TimerPause)) {
        intents.push(fill_intent(action, tokens, number, None, None, None));
    }
    intents.retain(|i| i.name != "Unknown");
    (!intents.is_empty()).then_some(ClauseOut::Intents(intents))
}

pub(crate) fn laundry_switch_clause(
    tokens: &[String],
    home: &HomeGraph,
    action: Action,
    number: Option<i32>,
    domain: Option<&str>,
) -> Option<ClauseOut> {
    if domain != Some("switch") || !matches!(action, Action::On | Action::Off | Action::Toggle) {
        return None;
    }
    if !catalog().any(tokens, &catalog().laundry_area) {
        return None;
    }
    if catalog().any(tokens, &catalog().laundry_machines) {
        return None;
    }
    let areas = crate::home_policy::laundry_areas(home);
    let area = areas.first()?.as_str();
    let switches: Vec<String> = home
        .entities
        .iter()
        .filter(|e| e.domain == "switch" && e.area.as_deref().is_some_and(|a| areas.iter().any(|id| id == a)))
        .map(|e| e.entity_id.clone())
        .collect();
    if switches.len() < 2 {
        return None;
    }
    let plural = catalog().any(tokens, &catalog().switch_plural);
    let start = catalog().any(tokens, &catalog().start_words);
    let one = (tokens.iter().any(|t| t == "switch") && !plural) || start;
    if !one && !plural {
        return Some(ClauseOut::Clarify(switches, intent_from_action(action, tokens).with("area", area).with("domain", "switch")));
    }
    let id = one.then(|| switches.iter().find(|id| catalog().laundry_machines.iter().any(|m| id.contains(m))).cloned()).flatten();
    Some(ClauseOut::Intents(vec![fill_intent(action, tokens, number, id.as_deref(), Some(area), Some("switch"))]))
}
