use crate::home::expose::assist_visible;
use crate::home::policy::{is_infra, is_infra_light, is_whole_home};
use crate::home::roles::is_light_like;
use crate::lang::catalog;
use crate::parse::action::Action;
use crate::parse::compound::area_slots;
use crate::parse::infer::{bind_domain, color_word, except_focus, except_tail, wants_all_lights};
use crate::parse::normalize::compact;
use crate::parse::resolve::resolve;
use crate::types::{EntityRec, HomeGraph, Intent};
use std::collections::HashSet;

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
    if except_tail(tokens).is_some() {
        return except_all_lights(tokens, home, action, number);
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

fn except_all_lights(tokens: &[String], home: &HomeGraph, action: Action, number: Option<i32>) -> Option<ClauseOut> {
    let skip_areas = excepted_areas(tokens, home);
    let skip = if skip_areas.is_empty() { excepted_light_ids(tokens, home) } else { HashSet::new() };
    let mut intents = Vec::new();
    for area in &home.areas {
        if is_whole_home(area) || skip_areas.contains(&area.area_id) {
            continue;
        }
        let lights = area_switchable_lights(home, &area.area_id);
        if lights.is_empty() {
            continue;
        }
        if lights.iter().any(|entity| skip.contains(&entity.entity_id)) {
            for entity in lights {
                if skip.contains(&entity.entity_id) {
                    continue;
                }
                intents.push(fill_intent(action, tokens, number, Some(&entity.entity_id), None, Some("light")));
            }
            continue;
        }
        let (id, slot, dom) = area_slots(action, &area.area_id, Some("light"), home, tokens);
        let intent = fill_intent(action, tokens, number, id.as_deref(), slot.as_deref(), dom.as_deref());
        if intent.name != "Unknown" {
            intents.push(intent);
        }
    }
    (!intents.is_empty()).then_some(ClauseOut::Intents(intents))
}

fn excepted_areas(tokens: &[String], home: &HomeGraph) -> HashSet<String> {
    let Some(focus) = except_focus(tokens) else {
        return HashSet::new();
    };
    let cat = catalog();
    if cat.any(&focus, &cat.named_device) || cat.any(&focus, &cat.light_nouns) {
        return HashSet::new();
    }
    resolve(&focus, home, Some("light")).areas.into_iter().collect()
}

fn excepted_light_ids(tokens: &[String], home: &HomeGraph) -> HashSet<String> {
    let focus = except_focus(tokens).or_else(|| except_tail(tokens).map(|tail| tail.to_vec()));
    let Some(focus) = focus else {
        return HashSet::new();
    };
    let resolved = resolve(&focus, home, Some("light"));
    let seed = resolved.entities.first().or_else(|| resolved.ambiguous.iter().min_by_key(|entity| compact(&entity.name).len()));
    let Some(seed) = seed else {
        return HashSet::new();
    };
    let name = compact(&seed.name);
    home.entities
        .iter()
        .filter(|entity| assist_visible(entity, home) && entity.domain == "light" && compact(&entity.name) == name)
        .map(|entity| entity.entity_id.clone())
        .collect()
}

fn area_switchable_lights<'a>(home: &'a HomeGraph, area: &str) -> Vec<&'a EntityRec> {
    home.entities
        .iter()
        .filter(|entity| {
            assist_visible(entity, home)
                && is_light_like(entity)
                && !is_infra(entity)
                && !is_infra_light(entity)
                && entity.area.as_deref() == Some(area)
                && !is_home_group_light(entity, home)
        })
        .collect()
}

fn is_home_group_light(entity: &EntityRec, home: &HomeGraph) -> bool {
    if entity.domain != "light" {
        return false;
    }
    let id = entity.entity_id.to_ascii_lowercase();
    id.contains("alle")
        || id.contains("_und_")
        || id.contains("_and_")
        || name_has_group_conj(&entity.name)
        || entity.area.as_ref().is_some_and(|area| home.areas.iter().any(|rec| rec.area_id == *area && is_whole_home(rec)))
}

fn name_has_group_conj(name: &str) -> bool {
    name.split(|c: char| !c.is_ascii_alphanumeric()).any(|part| matches!(compact(part).as_str(), "und" | "and"))
}

pub(crate) fn fill_intent(
    action: Action,
    tokens: &[String],
    number: Option<i32>,
    entity_id: Option<&str>,
    area: Option<&str>,
    domain: Option<&str>,
) -> Intent {
    let target = domain.or_else(|| entity_id.and_then(|id| id.split('.').next()));
    let action = bind_domain(action, tokens, number, target);
    let mut intent = intent_from_action(action, tokens);
    if let Some(id) = entity_id {
        intent = intent.with("entity_id", id);
    } else if let Some(a) = area {
        intent = intent.with("area", a);
    }
    if entity_id.is_none() {
        if let Some(d) = domain {
            if !matches!(
                action,
                Action::TimerStart | Action::TimerAdd | Action::TimerCancel | Action::TimerPause | Action::ListAdd | Action::ListComplete
            ) {
                intent = intent.with("domain", d);
            }
        }
    }
    match action {
        Action::SetLight => {
            if let Some(c) = color_word(tokens) {
                intent = intent.with("color", c);
            } else if let Some(n) = number {
                intent = intent.with("brightness", n.to_string());
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
            if let Some(item) = list_item(tokens, None) {
                intent = intent.with("item", item);
            }
            if entity_id.is_none() {
                intent = intent.with("name", "shopping_list");
            }
        }
        _ => {}
    }
    intent
}

pub(crate) fn fill_list_intent(action: Action, tokens: &[String], target: Option<&EntityRec>) -> Intent {
    let mut intent = intent_from_action(action, tokens);
    if let Some(entity) = target {
        intent = intent.with("entity_id", &entity.entity_id);
    } else {
        intent = intent.with("name", "shopping_list");
    }
    if let Some(item) = list_item(tokens, target) {
        intent = intent.with("item", item);
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

fn list_item(tokens: &[String], target: Option<&EntityRec>) -> Option<String> {
    let cat = catalog();
    let words: Vec<&str> = tokens
        .iter()
        .map(String::as_str)
        .filter(|token| {
            !cat.list_skip.contains(token)
                && !cat.list_nouns.contains(token)
                && !cat.shopping_names.contains(token)
                && !target.is_some_and(|entity| target_label_token(token, entity))
        })
        .collect();
    if words.is_empty() {
        None
    } else {
        Some(words.join(" "))
    }
}

fn target_label_token(token: &str, entity: &EntityRec) -> bool {
    let token = compact(token);
    std::iter::once(&entity.name)
        .chain(entity.aliases.iter())
        .chain(entity.tags.iter())
        .flat_map(|label| std::iter::once(compact(label)).chain(label.split(|character: char| !character.is_alphanumeric()).map(compact)))
        .filter(|label| !label.is_empty())
        .any(|label| token == label || catalog().list_nouns.iter().any(|suffix| token == format!("{label}{}", compact(suffix))))
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
                Intent::new("HassClimateGetTemperature")
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

#[derive(Debug, Clone)]
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
    let mut intents: Vec<Intent> = crate::parse::resolve::pick_timers(tokens, home)
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
    let areas = crate::home::policy::laundry_areas(home);
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

#[cfg(test)]
mod tests {
    use super::name_has_group_conj;
    use crate::parse::parse;
    use crate::session::Session;
    use crate::types::{EntityRec, HomeGraph, Settings};
    use std::collections::HashSet;

    #[test]
    fn group_conj_is_a_word_not_a_substring() {
        assert!(name_has_group_conj("Wohn und Esszimmer"));
        assert!(name_has_group_conj("Living and Dining"));
        assert!(!name_has_group_conj("Island Lights"));
        assert!(!name_has_group_conj("Insel Licht"));
        assert!(!name_has_group_conj("Standleuchte"));
    }

    #[test]
    fn configured_todo_names_aliases_and_labels_route_without_object_id_evidence() {
        for (sentence, entity) in [
            ("Füge Milch zur Aufgabenliste hinzu", todo("todo.internal_de", "Aufgaben", &[], &[])),
            ("Add milk to the House Tasks list", todo("todo.internal_en", "House Tasks", &[], &[])),
            ("Add milk to the Errands list", todo("todo.internal_alias", "Internal", &["Errands"], &[])),
            ("Add milk to the Weekend list", todo("todo.internal_label", "Internal", &[], &["Weekend"])),
        ] {
            let result = parse_with_home(sentence, HomeGraph { entities: vec![entity.clone()], ..HomeGraph::default() });
            assert!(!result.clarify, "{sentence}: {result:?}");
            assert_eq!(result.intents.len(), 1, "{sentence}: {result:?}");
            assert_eq!(result.intents[0].slot("entity_id"), Some(entity.entity_id.as_str()), "{sentence}: {result:?}");
            assert_eq!(
                result.intents[0].slot("item"),
                sentence.starts_with("Füge").then_some("milch").or(Some("milk"))
            );
        }
    }

    #[test]
    fn generic_shopping_does_not_guess_todo_and_hidden_or_ambiguous_names_do_not_execute() {
        let first = todo("todo.first", "Errands", &[], &[]);
        let second = todo("todo.second", "Errands", &[], &[]);
        let generic = parse_with_home(
            "Add milk to the shopping list",
            HomeGraph { entities: vec![first.clone(), second.clone()], ..HomeGraph::default() },
        );
        assert_eq!(generic.intents.len(), 1, "{generic:?}");
        assert_eq!(generic.intents[0].slot("name"), Some("shopping_list"));
        assert_eq!(generic.intents[0].slot("entity_id"), None);

        let ambiguous =
            parse_with_home("Add milk to the Errands list", HomeGraph { entities: vec![first.clone(), second], ..HomeGraph::default() });
        assert!(ambiguous.clarify, "{ambiguous:?}");
        assert!(ambiguous.intents.is_empty(), "{ambiguous:?}");

        let hidden = parse_with_home(
            "Add milk to the Errands list",
            HomeGraph { entities: vec![first], assist: Some(HashSet::new()), ..HomeGraph::default() },
        );
        assert!(!hidden.clarify, "{hidden:?}");
        assert!(hidden.intents.is_empty(), "{hidden:?}");
    }

    fn parse_with_home(sentence: &str, home: HomeGraph) -> crate::types::ParseResult {
        parse(sentence, &home, &mut Session::new(), &[], &Settings::default())
    }

    fn todo(id: &str, name: &str, aliases: &[&str], tags: &[&str]) -> EntityRec {
        EntityRec {
            entity_id: id.into(),
            name: name.into(),
            domain: "todo".into(),
            platform: None,
            area: None,
            aliases: aliases.iter().map(|value| (*value).into()).collect(),
            tags: tags.iter().map(|value| (*value).into()).collect(),
        }
    }
}
