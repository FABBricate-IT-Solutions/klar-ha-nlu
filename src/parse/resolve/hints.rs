//! Domain and timer/climate pick helpers for resolve.
use super::spans::{timer_noun_free, LinkedNames};
use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::home::roles::{hot_water_asked, is_hot_water, matches_domain};
use crate::lang::catalog;
use crate::parse::action::{has_light_noun, is_garage_cover, is_query_token};
use crate::parse::normalize::is_time_unit;
use crate::types::HomeGraph;

pub fn domain_hint(tokens: &[String]) -> Option<&'static str> {
    domain_hint_with(tokens, &LinkedNames::default())
}

pub(crate) fn domain_hint_with(tokens: &[String], linked: &LinkedNames) -> Option<&'static str> {
    let cat = &catalog();
    if timer_noun_free(tokens, linked) {
        return Some("timer");
    }
    // Generic "device/appliance" in the laundry room is washer-vs-dryer, not lights.
    if cat.any(tokens, cat.laundry_area())
        && cat.any(tokens, cat.extra_device_nouns())
        && !has_light_noun(tokens)
        && !cat.any(tokens, cat.laundry_machines())
        && !cat.any(tokens, cat.skip_light())
    {
        return Some("switch");
    }
    for (index, t) in tokens.iter().enumerate() {
        // A domain word inside a linked device name ("timer" in "Timer Lamp") is not a hint.
        if linked.consumed.contains(&index) {
            continue;
        }
        if *t == "hue" {
            return Some("light");
        }
        if cat.door_nouns().contains(t.as_str()) {
            if cat.any(tokens, cat.sensor_words()) {
                return Some("binary_sensor");
            }
            if tokens.iter().any(|x| cat.lock_verbs().contains(x.as_str())) {
                return Some("lock");
            }
            if cat.any(tokens, cat.entry_words()) {
                return Some("lock");
            }
            if is_garage_cover(tokens) {
                return Some("cover");
            }
            return Some("lock");
        }
        if cat.window_words().contains(t.as_str()) {
            if cat.any(tokens, cat.sensor_words()) {
                return Some("binary_sensor");
            }
            return Some("cover");
        }
        let Some(domain) = cat.domain_map.get(t.as_str()).copied() else {
            continue;
        };
        if domain == "switch" {
            let skip_laundry = cat.laundry_area().contains(t.as_str())
                && (has_light_noun(tokens)
                    || crate::parse::numbers::first_number(tokens).is_some()
                    || is_query_token(tokens)
                    || tokens.iter().any(|x| catalog().color(x).is_some()));
            let skip_bare_machine = cat.bare_switch().contains(t.as_str()) && !cat.any(tokens, cat.laundry_hint());
            if skip_laundry || skip_bare_machine {
                continue;
            }
        }
        return Some(domain);
    }
    // A climate noun without an entry of its own in the map ("Thermostate") names the heating too.
    if cat.any(tokens, cat.climate_nouns()) {
        return Some("climate");
    }
    cat.fuzzy_domain(tokens)
}

pub(crate) fn pick_timers(tokens: &[String], home: &HomeGraph) -> Vec<String> {
    let ids: Vec<String> =
        home.entities.iter().filter(|e| assist_visible(e, home) && e.domain == "timer").map(|e| e.entity_id.clone()).collect();
    let want = |n: &str| ids.iter().filter(|id| id.contains(n)).cloned().collect::<Vec<_>>();
    if tokens.iter().any(|t| catalog().is_all(t)) && !tokens.iter().any(|t| is_time_unit(t)) {
        return ids.into_iter().filter(|id| !id.contains("abstract")).collect();
    }
    if catalog().any(tokens, catalog().oven()) {
        return want("oven");
    }
    if catalog().any(tokens, catalog().laundry_timer())
        || crate::home::policy::timer_hint(home, crate::parse::numbers::first_number(tokens)) == Some("laundry")
    {
        return want("laundry");
    }
    ids.iter().find(|id| id.contains("abstract")).cloned().into_iter().collect()
}

pub(crate) fn unique_in_area(home: &HomeGraph, area: &str, domain: &str, tokens: &[String]) -> Option<String> {
    let kind = (domain == "climate").then(|| crate::home::roles::wanted_climate_kind(tokens, &catalog())).flatten();
    let water = hot_water_asked(tokens);
    let hits: Vec<&str> = home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| matches_domain(e, domain, &catalog()) && !is_infra(e) && e.area.as_deref() == Some(area))
        .filter(|e| kind.is_none_or(|want| crate::home::roles::climate_kind(e, &catalog()) == Some(want)))
        .filter(|e| domain != "climate" || is_hot_water(e) == water)
        .map(|e| e.entity_id.as_str())
        .collect();
    (hits.len() == 1).then(|| hits[0].to_string())
}

pub(crate) fn climates_of_kind(home: &HomeGraph, tokens: &[String]) -> Vec<String> {
    let kind = crate::home::roles::wanted_climate_kind(tokens, &catalog());
    let water = hot_water_asked(tokens);
    home.entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| matches_domain(e, "climate", &catalog()) && !is_infra(e))
        .filter(|e| kind.is_none_or(|want| crate::home::roles::climate_kind(e, &catalog()) == Some(want)))
        .filter(|e| is_hot_water(e) == water)
        .map(|e| e.entity_id.clone())
        .collect()
}
