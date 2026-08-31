use crate::lang::catalog;
use crate::parse::action::{detect_actions, Action};
use crate::parse::normalize::fold_umlaut;
use crate::types::HomeGraph;

/// Split a token stream into clauses. "Wohnzimmer und Küche" stays one clause
/// unless a new action verb appears after the conjunction.
pub fn split_clauses(tokens: &[String], home: &HomeGraph) -> Vec<Vec<String>> {
    if let Some(at) = protected_media_followup(tokens, home) {
        let mut clauses = vec![tokens[..at].to_vec()];
        clauses.extend(split_clauses(&tokens[at + 1..], home));
        return clauses;
    }
    if protected_media_status(tokens) || phrase_spans_conj(tokens, home) {
        return vec![tokens.to_vec()];
    }
    if let Some(at) = split_two_targets(tokens) {
        let mut left = tokens[..at].to_vec();
        let mut right = tokens[at + 1..].to_vec();
        if detect_actions(&right).is_empty() {
            for (i, _) in detect_actions(tokens) {
                if i < at {
                    right.insert(0, tokens[i].clone());
                }
            }
        }
        // "Kugel und Decke aus" / "raito ribingu to kitchin keshite":
        // trailing particle or sole off/on verb belongs to both sides.
        if let Some(last) = tokens.last() {
            if is_shared_tail(last) && !left.iter().any(|token| is_shared_tail(token)) {
                left.push(last.clone());
            }
        }
        if !left.is_empty() && !right.is_empty() {
            return vec![left, right];
        }
    }
    let actions = detect_actions(tokens);
    if actions.len() <= 1 {
        return vec![tokens.to_vec()];
    }

    let mut cuts = Vec::new();
    for window in actions.windows(2) {
        let (i0, a0) = window[0];
        let (i1, a1) = window[1];
        if i1 <= i0 {
            continue;
        }
        let between = &tokens[i0 + 1..i1];
        let has_conj = between.iter().any(|t| is_conj(t));
        // Same verb twice ("mach … an") is one clause. A new verb, or
        // the same verb after "und", starts another.
        if a0 == a1 {
            continue;
        }
        if is_same_command(a0, a1) && !has_conj {
            continue;
        }
        if a0 == a1 && is_particle(&tokens[i1]) || trailing_power_particle(tokens, i1, a0, a1, has_conj) {
            continue;
        }
        if has_conj || (is_new_action_span(a1) && a1 != a0) {
            if let Some(cut) = between.iter().rposition(|t| is_conj(t)) {
                cuts.push(i0 + 1 + cut + 1);
            } else {
                cuts.push(i1);
            }
        }
    }

    if cuts.is_empty() {
        return vec![tokens.to_vec()];
    }

    let mut clauses = Vec::new();
    let mut start = 0;
    for cut in cuts {
        if cut > start {
            clauses.push(tokens[start..cut].to_vec());
        }
        start = cut;
    }
    if start < tokens.len() {
        clauses.push(tokens[start..].to_vec());
    }
    clauses.retain(|c| !c.is_empty());
    if clauses.is_empty() {
        vec![tokens.to_vec()]
    } else {
        clauses
    }
}

fn protected_media_followup(tokens: &[String], home: &HomeGraph) -> Option<usize> {
    tokens.iter().enumerate().find_map(|(index, token)| {
        if !is_conj(token) || conj_covered_by_name(tokens, home, index) || !protected_media_status(&tokens[..index]) {
            return None;
        }
        let right = &tokens[index + 1..];
        let actions = detect_actions(right);
        let new_command = actions.iter().any(|(_, action)| *action != Action::GetState) || explicit_query_start(right);
        new_command.then_some(index)
    })
}

fn explicit_query_start(tokens: &[String]) -> bool {
    tokens.first().is_some_and(|token| matches!(token.as_str(), "was" | "wie" | "ist" | "sind" | "what" | "whats" | "is" | "are" | "how"))
}

fn protected_media_status(tokens: &[String]) -> bool {
    let question = tokens.iter().any(|token| matches!(token.as_str(), "was" | "wie" | "what" | "whats"));
    let next = tokens.iter().any(|token| matches!(token.as_str(), "next" | "naechster" | "naechste" | "naechstes"));
    let media = tokens
        .iter()
        .any(|token| matches!(token.as_str(), "queue" | "warteschlange" | "music" | "musik" | "song" | "track" | "lied" | "titel"));
    has_phrase(tokens, &["kommt", "als", "naechstes"]) || has_phrase(tokens, &["wie", "laut"]) || (question && next && media)
}

fn has_phrase(tokens: &[String], phrase: &[&str]) -> bool {
    tokens.windows(phrase.len()).any(|window| window.iter().map(String::as_str).eq(phrase.iter().copied()))
}

fn phrase_spans_conj(tokens: &[String], home: &HomeGraph) -> bool {
    let Some(at) = tokens.iter().position(|t| is_conj(t)) else {
        return false;
    };
    conj_covered_by_name(tokens, home, at)
}

fn conj_covered_by_name(tokens: &[String], home: &HomeGraph, at: usize) -> bool {
    home.entities.iter().any(|ent| {
        name_covers_conj(&ent.name, tokens, at)
            || ent.aliases.iter().any(|alias| name_covers_conj(alias, tokens, at))
            || name_covers_conj(ent.entity_id.rsplit('.').next().unwrap_or(&ent.entity_id), tokens, at)
    }) || home.areas.iter().any(|area| {
        name_covers_conj(&area.name, tokens, at)
            || name_covers_conj(&area.area_id, tokens, at)
            || area.aliases.iter().any(|alias| name_covers_conj(alias, tokens, at))
    })
}

fn name_covers_conj(name: &str, tokens: &[String], at: usize) -> bool {
    let parts: Vec<String> =
        fold_umlaut(name).split(|c: char| !c.is_alphanumeric()).filter(|part| !part.is_empty()).map(str::to_string).collect();
    if parts.len() < 3 || !parts.iter().any(|p| is_conj(p)) {
        return false;
    }
    tokens
        .windows(parts.len())
        .enumerate()
        .any(|(start, window)| (start..start + parts.len()).contains(&at) && window.iter().zip(&parts).all(|(token, part)| token == part))
}

fn split_two_targets(tokens: &[String]) -> Option<usize> {
    let at = tokens.iter().position(|t| is_conj(t))?;
    let left = &tokens[..at];
    let right = &tokens[at + 1..];
    let right_head = right.split(|t| is_conj(t)).next().unwrap_or(right);
    if device_side(left)
        && device_side(right_head)
        && !right.iter().skip(right_head.len()).any(|t| is_conj(t))
        && !both_covers(left, right_head)
    {
        Some(at)
    } else {
        None
    }
}

fn both_covers(left: &[String], right: &[String]) -> bool {
    let cat = catalog();
    let cover = |tokens: &[String]| cat.any(tokens, cat.cover_nouns()) || cat.any(tokens, cat.curtain_nouns());
    cover(left) && cover(right)
}

fn device_side(tokens: &[String]) -> bool {
    let cat = catalog();
    cat.any(tokens, cat.device_side())
        || cat.any(tokens, cat.named_device())
        || cat.any(tokens, cat.ceiling())
        || cat.any(tokens, cat.cover_nouns())
        || cat.any(tokens, cat.fan_nouns())
        || cat.any(tokens, cat.island())
        || crate::parse::infer::mentions_lamp_fixture(tokens)
        || cat.any(tokens, cat.laundry_machines())
        || tokens.iter().any(|token| matches!(token.as_str(), "dryer" | "washer"))
}

fn is_shared_tail(token: &str) -> bool {
    is_particle(token)
        || matches!(catalog().verb(token), Some(crate::lang::VerbKind::Off | crate::lang::VerbKind::On | crate::lang::VerbKind::OnParticle))
}

fn is_conj(t: &str) -> bool {
    catalog().is_conj(t)
}

fn is_new_action_span(action: Action) -> bool {
    !matches!(action, Action::GetState)
}

fn is_particle(token: &str) -> bool {
    catalog().is_particle(token)
}

fn trailing_power_particle(tokens: &[String], i1: usize, a0: Action, a1: Action, has_conj: bool) -> bool {
    if i1 + 1 != tokens.len() || !is_particle(&tokens[i1]) {
        return false;
    }
    if matches!((a0, a1), (Action::On, Action::Off) | (Action::Off, Action::On)) {
        return !has_conj;
    }
    matches!(a0, Action::GetState)
        && matches!(a1, Action::On | Action::Off)
        && catalog().any(tokens, catalog().climate_nouns())
        && !tokens.iter().any(|token| catalog().is_question_word(token) || catalog().is_question_start(token))
}

fn is_same_command(a: Action, b: Action) -> bool {
    matches!(
        (a, b),
        (Action::On | Action::Off, Action::FanSpeed)
            | (Action::FanSpeed, Action::On | Action::Off)
            | (Action::On | Action::Off, Action::SetLight)
            | (Action::SetLight, Action::On | Action::Off)
            | (Action::CoverOpen | Action::CoverClose, Action::CoverSet)
            | (Action::CoverSet, Action::CoverOpen | Action::CoverClose)
            | (Action::Lock | Action::Unlock, Action::On | Action::Off)
            | (Action::On | Action::Off, Action::Lock | Action::Unlock)
            | (Action::Lock, Action::Unlock)
            | (Action::Unlock, Action::Lock)
            | (Action::SetTemp, Action::SetLight)
            | (Action::SetLight, Action::SetTemp)
            | (Action::GetState, Action::FanSpeed | Action::SetLight | Action::SetTemp)
            | (Action::FanSpeed | Action::SetLight | Action::SetTemp, Action::GetState)
            | (
                Action::GetState,
                Action::On | Action::Off | Action::Lock | Action::Unlock | Action::CoverOpen | Action::CoverClose | Action::CoverSet
            )
            | (
                Action::On | Action::Off | Action::Lock | Action::Unlock | Action::CoverOpen | Action::CoverClose | Action::CoverSet,
                Action::GetState
            )
            | (Action::CoverOpen, Action::CoverClose)
            | (Action::CoverClose, Action::CoverOpen)
            | (
                Action::On | Action::Off | Action::SetLight | Action::MediaPause | Action::MediaPlay,
                Action::TimerStart | Action::TimerAdd | Action::TimerCancel | Action::TimerPause
            )
            | (
                Action::TimerStart | Action::TimerAdd | Action::TimerCancel | Action::TimerPause,
                Action::On | Action::Off | Action::SetLight | Action::MediaPause | Action::MediaPlay
            )
            | (Action::GetState, Action::TimerStart | Action::TimerAdd | Action::TimerCancel | Action::TimerPause)
            | (Action::TimerStart | Action::TimerAdd | Action::TimerCancel | Action::TimerPause, Action::GetState)
            | (Action::GetState, Action::VacuumDock | Action::VacuumStart)
            | (Action::VacuumDock | Action::VacuumStart, Action::GetState)
            | (Action::VacuumStart | Action::VacuumDock, Action::On | Action::Off)
            | (Action::On | Action::Off, Action::VacuumStart | Action::VacuumDock)
            | (Action::ListAdd | Action::ListComplete, Action::On | Action::Off | Action::SetLight)
            | (Action::On | Action::Off | Action::SetLight, Action::ListAdd | Action::ListComplete)
    )
}

pub(crate) fn wants_group_clarify(raw: &[String]) -> bool {
    catalog().wants_group_clarify(raw)
}

pub(crate) fn follow_fixture(tokens: &[String], home: &crate::types::HomeGraph, areas: &[String]) -> Option<String> {
    if areas.is_empty() {
        return None;
    }
    let lights: Vec<&str> = home
        .entities
        .iter()
        .filter(|e| e.domain == "light" && e.area.as_ref().is_some_and(|a| areas.contains(a)))
        .map(|e| e.entity_id.as_str())
        .collect();
    let find = |n: &str| lights.iter().find(|id| id.contains(n)).map(|s| (*s).to_string());
    let cat = catalog();
    if cat.any(tokens, cat.island()) {
        return find("island").or_else(|| find("insel"));
    }
    if cat.any(tokens, cat.ceiling()) {
        return find("ceiling").or_else(|| find("decke"));
    }
    if crate::parse::infer::mentions_lamp_fixture(tokens) {
        return find("bedside").or_else(|| find("lamp")).or_else(|| find("ceiling"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::split_clauses;
    use crate::parse::parse;
    use crate::session::Session;
    use crate::types::{AreaRec, EntityRec, FloorRec, HomeGraph, Settings};

    fn rec(id: &str, name: &str, domain: &str, area: &str, aliases: &[&str]) -> EntityRec {
        EntityRec {
            entity_id: id.into(),
            name: name.into(),
            domain: domain.into(),
            platform: None,
            area: Some(area.into()),
            aliases: aliases.iter().map(|value| (*value).into()).collect(),
            tags: Vec::new(),
        }
    }

    fn leftover_home() -> HomeGraph {
        HomeGraph {
            floors: vec![FloorRec {
                floor_id: "basement".into(),
                name: "Keller".into(),
                aliases: vec!["keller".into(), "basement".into()],
                level: Some(-1),
            }],
            areas: vec![
                AreaRec {
                    area_id: "living".into(),
                    name: "Wohnzimmer".into(),
                    aliases: vec!["wohnzimmer".into(), "living".into(), "ribingu".into()],
                    floor_id: Some("ground".into()),
                },
                AreaRec {
                    area_id: "office".into(),
                    name: "Büro".into(),
                    aliases: vec!["arbeitszimmer".into(), "office".into()],
                    floor_id: Some("ground".into()),
                },
                AreaRec {
                    area_id: "garden".into(),
                    name: "Garten".into(),
                    aliases: vec!["garten".into(), "garden".into()],
                    floor_id: Some("ground".into()),
                },
                AreaRec {
                    area_id: "basement".into(),
                    name: "Keller".into(),
                    aliases: vec!["keller".into(), "basement".into()],
                    floor_id: Some("basement".into()),
                },
                AreaRec {
                    area_id: "kitchen".into(),
                    name: "Küche".into(),
                    aliases: vec!["kuche".into(), "kitchen".into()],
                    floor_id: Some("ground".into()),
                },
            ],
            entities: vec![
                rec("cover.living_blinds", "Rollo", "cover", "living", &["rollo"]),
                rec("light.living_ceiling", "Wohnzimmer Decke", "light", "living", &["decke", "deckenlampe"]),
                rec("light.living_globe", "Kugel", "light", "living", &["kugel", "globe"]),
                rec("light.living_lamp", "Wohnzimmer Lampe", "light", "living", &["lampe", "lamp"]),
                rec("light.office_ceiling", "Büro Decke", "light", "office", &["decke"]),
                rec("light.garden", "Garten Licht", "light", "garden", &["garten licht"]),
                rec("light.basement", "Keller Licht", "light", "basement", &["keller licht"]),
                rec("light.kitchen", "Küche Licht", "light", "kitchen", &["kuche licht"]),
            ],
            ..HomeGraph::default()
        }
    }

    fn parse_de(sentence: &str, home: HomeGraph) -> crate::types::ParseResult {
        parse(sentence, &home, &mut Session::new(), &[], &Settings::pinned("de"))
    }

    #[test]
    fn trailing_particle_clause_cuts() {
        let _bound = crate::lang::bind_catalog(crate::lang::catalog_for(&["de".into(), "en".into()]));
        let home = leftover_home();
        let one = |words: &[&str]| words.iter().map(|word| (*word).to_string()).collect::<Vec<_>>();
        assert_eq!(split_clauses(&one(&["activate", "all", "off"]), &home).len(), 1);
        assert_eq!(split_clauses(&one(&["mach", "heizung", "schlafzimmer", "und", "wohnzimmer", "an"]), &home).len(), 1);
        assert_eq!(split_clauses(&one(&["wie", "warm", "ist", "wohnzimmer", "und", "mach", "lichter", "aus"]), &home).len(), 2);
        assert_eq!(split_clauses(&one(&["ist", "kinderzimmer", "an", "und", "mach", "lichter", "aus"]), &home).len(), 2);
    }

    #[test]
    fn cover_and_ceiling_split_on_und() {
        let tokens = ["mach", "rollo", "im", "wohnzimmer", "an", "und", "decke", "im", "wohnzimmer", "an"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        let clauses = split_clauses(&tokens, &leftover_home());
        assert_eq!(clauses.len(), 2, "{clauses:?}");
        let result = parse_de("mach rollo im wohnzimmer an und decke im wohnzimmer an", leftover_home());
        let ids: Vec<_> = result.intents.iter().filter_map(|intent| intent.slot("entity_id")).collect();
        assert!(ids.contains(&"cover.living_blinds"), "{result:?}");
        assert!(ids.contains(&"light.living_ceiling"), "{result:?}");
        assert!(!ids.contains(&"light.office_ceiling"), "{result:?}");
    }

    #[test]
    fn living_and_kitchen_off_stays_areas() {
        let result = parse_de("licht wohnzimmer und kuche aus", leftover_home());
        assert!(!result.clarify, "{result:?}");
        let hit = |needle: &str| {
            result
                .intents
                .iter()
                .any(|intent| intent.slot("area") == Some(needle) || intent.slot("entity_id").is_some_and(|id| id.contains(needle)))
        };
        assert!(hit("living") || hit("wohnzimmer"), "{result:?}");
        assert!(hit("kitchen") || hit("kuche"), "{result:?}");
        assert!(result.intents.iter().all(|intent| intent.name == "HassTurnOff"), "{result:?}");
    }

    #[test]
    fn basement_does_not_drop_garden() {
        let result = parse_de("licht keller und garten an", leftover_home());
        let hit = |needle: &str| {
            result
                .intents
                .iter()
                .any(|intent| intent.slot("area") == Some(needle) || intent.slot("entity_id").is_some_and(|id| id.contains(needle)))
        };
        assert!(hit("garden") || hit("garten"), "{result:?}");
        assert!(hit("basement") || hit("keller"), "{result:?}");
    }

    #[test]
    fn globe_and_ceiling_share_trailing_off() {
        let result = parse_de("kugel und decke aus", leftover_home());
        let ids: Vec<_> = result.intents.iter().filter_map(|intent| intent.slot("entity_id")).collect();
        assert!(ids.contains(&"light.living_globe"), "{result:?}");
        assert!(ids.contains(&"light.living_ceiling"), "{result:?}");
        assert!(result.intents.iter().all(|intent| intent.name == "HassTurnOff"), "{result:?}");
    }

    #[test]
    fn except_office_ceiling_skips_office_area() {
        let result = parse_de("alle lichter aus ausser decke im arbeitszimmer", leftover_home());
        assert!(!result.intents.iter().any(|intent| intent.slot("area") == Some("office")), "{result:?}");
        assert!(!result.intents.iter().any(|intent| intent.slot("entity_id") == Some("light.office_ceiling")), "{result:?}");
    }

    #[test]
    fn except_lamp_token_skips_living_lamp() {
        let result = parse("keshite 全部 raito 以外 lamp ribingu", &leftover_home(), &mut Session::new(), &[], &Settings::pinned("ja"));
        assert!(!result.intents.iter().any(|intent| intent.slot("entity_id") == Some("light.living_lamp")), "{result:?}");
        assert!(
            result
                .intents
                .iter()
                .any(|intent| intent.slot("entity_id") == Some("light.living_ceiling") || intent.slot("area") == Some("living")),
            "{result:?}"
        );
    }

    #[test]
    fn except_garden_turns_off_others() {
        let result = parse("keshite 全部 raito 以外 garden", &leftover_home(), &mut Session::new(), &[], &Settings::pinned("ja"));
        assert!(!result.clarify, "{result:?}");
        assert!(result.intents.iter().any(|intent| intent.name == "HassTurnOff"), "{result:?}");
        assert!(
            !result.intents.iter().any(|intent| intent.slot("area") == Some("garden") || intent.slot("entity_id") == Some("light.garden")),
            "{result:?}"
        );
        assert!(
            result
                .intents
                .iter()
                .any(|intent| intent.slot("area") == Some("living") || intent.slot("entity_id").is_some_and(|id| id.contains("living"))),
            "{result:?}"
        );
    }
}
