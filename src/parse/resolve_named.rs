use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::lang::catalog;
use crate::parse::normalize::fold_umlaut;
use crate::parse::resolve::{fixture_matches, token_eq, token_hit};
use crate::types::{AreaRec, EntityRec, HomeGraph};
use strsim::normalized_levenshtein;

pub(crate) fn fuzzy_areas(tokens: &[String], areas: &[AreaRec]) -> Option<Vec<String>> {
    let mut best = 0.0_f64;
    let mut winner: Option<String> = None;
    let mut second = 0.0_f64;
    for area in areas {
        let names: Vec<String> = std::iter::once(fold_umlaut(&area.name))
            .chain(std::iter::once(area.area_id.clone()))
            .chain(area.aliases.iter().map(|alias| fold_umlaut(alias)))
            .collect();
        for name in names {
            if name.len() < 6 {
                continue;
            }
            for token in tokens {
                if catalog().generic.contains(&token.as_str()) || token.len() < 6 {
                    continue;
                }
                let score = normalized_levenshtein(token, &name);
                if score <= 0.88 {
                    continue;
                }
                if score > best {
                    second = best;
                    best = score;
                    winner = Some(area.area_id.clone());
                } else if score > second && winner.as_deref() != Some(area.area_id.as_str()) {
                    second = score;
                }
            }
        }
    }
    (best - second > 0.04).then_some(winner).flatten().map(|id| vec![id])
}

pub(crate) fn collect_named_devices(tokens: &[String], home: &HomeGraph) -> Option<Vec<EntityRec>> {
    let cat = catalog();
    let mut found = Vec::new();
    for token in tokens {
        let generic_lamp = matches!(token.as_str(), "lampe" | "lamp" | "leuchte");
        let named = cat.named_device.contains(token.as_str()) && !generic_lamp;
        let ceiling = cat.ceiling.contains(token.as_str());
        if !named && !ceiling {
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
                || (named && fixture_matches(entity, token))
            {
                found.push(entity.clone());
            }
        }
    }
    (!found.is_empty()).then_some(found)
}
