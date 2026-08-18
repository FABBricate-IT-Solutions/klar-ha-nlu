use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::lang::catalog;
use crate::parse::fuzzy::{select_unique, Evidence, Profile};
use crate::parse::infer::fixture_matches;
use crate::parse::normalize::{compact, fold_umlaut};
use crate::parse::resolve::{token_eq, token_hit};
use crate::types::{AreaRec, EntityRec, HomeGraph};

pub(crate) fn fuzzy_areas(tokens: &[String], areas: &[AreaRec]) -> Option<Vec<String>> {
    if !tokens.iter().any(|token| catalog().verb(token).is_some()) {
        return None;
    }
    let labels: Vec<(String, String)> = areas
        .iter()
        .flat_map(|area| {
            std::iter::once(compact(&area.name))
                .chain(std::iter::once(compact(&area.area_id)))
                .chain(area.aliases.iter().map(|alias| compact(alias)))
                .map(|label| (area.area_id.clone(), label))
        })
        .filter(|(_, label)| label.len() >= 6)
        .collect();
    let mut hits: Vec<(String, Evidence)> = Vec::new();
    let max_width = tokens.len().min(3);
    for width in 1..=max_width {
        for window in tokens.windows(width) {
            if window.iter().all(|token| catalog().generic.contains(&token.as_str())) {
                continue;
            }
            let observed = window.join("");
            let Some(hit) = select_unique(&observed, labels.iter().map(|(id, label)| (id.as_str(), label.as_str())), Profile::Target)
            else {
                continue;
            };
            if let Some(existing) = hits.iter_mut().find(|(id, _)| id == hit.key) {
                if hit.evidence.score > existing.1.score {
                    existing.1 = hit.evidence;
                }
            } else {
                hits.push((hit.key.to_string(), hit.evidence));
            }
        }
    }
    hits.sort_by(|left, right| right.1.score.partial_cmp(&left.1.score).unwrap_or(std::cmp::Ordering::Equal));
    let winner = hits.first()?;
    (hits.len() == 1).then(|| vec![winner.0.clone()])
}

pub(crate) fn collect_named_devices(tokens: &[String], home: &HomeGraph) -> Option<Vec<EntityRec>> {
    let cat = catalog();
    let mut found = Vec::new();
    for token in tokens {
        let generic_lamp = cat.light_nouns.contains(token.as_str()) || cat.light_singular.contains(token.as_str());
        let named = cat.named_device.contains(token.as_str()) && !generic_lamp;
        let ceiling = cat.ceiling.contains(token.as_str());
        let island = cat.island.contains(token.as_str());
        let bedside = cat.bedside.contains(token.as_str());
        let lamp = token == "lamp" || cat.lamp_fixture.contains(token.as_str());
        if !named && !ceiling && !island && !bedside && !lamp {
            continue;
        }
        for entity in &home.entities {
            if !assist_visible(entity, home) || entity.domain != "light" || is_infra(entity) {
                continue;
            }
            if found.iter().any(|have: &EntityRec| have.entity_id == entity.entity_id) {
                continue;
            }
            let alias_hit = entity.aliases.iter().any(|alias| {
                let folded = fold_umlaut(alias);
                !catalog().is_conj(&folded) && token_eq(token, &folded)
            });
            if token_hit(tokens, &fold_umlaut(&entity.name))
                || alias_hit
                || (ceiling && fixture_matches(entity, "ceiling"))
                || (island && fixture_matches(entity, "island"))
                || (bedside && fixture_matches(entity, token))
                || (lamp && fixture_matches(entity, "lamp"))
                || (named && fixture_matches(entity, token))
            {
                found.push(entity.clone());
            }
        }
    }
    (!found.is_empty()).then_some(found)
}
