use crate::home::expose::assist_visible;
use crate::lang::catalog;
use crate::parse::fuzzy::{evidence, Profile};
use crate::parse::infer::mentions_fixture_noun;
use crate::parse::media::is_media_move_or_play;
use crate::parse::normalize::{compact, fold_umlaut};
use crate::types::HomeGraph;

pub(crate) fn named_scene_or_script(tokens: &[String], home: &HomeGraph) -> Option<String> {
    if is_media_move_or_play(tokens) || catalog().any(tokens, catalog().media_nouns()) {
        return None;
    }
    let mentioned = tokens.iter().any(|t| catalog().scene_nouns().contains(t.as_str()) || catalog().script_words().contains(t.as_str()));
    if !mentioned && (catalog().any(tokens, catalog().light_nouns()) || mentions_fixture_noun(tokens)) {
        return None;
    }
    let mut hits: Vec<String> = home
        .entities
        .iter()
        .filter(|e| assist_visible(e, home))
        .filter(|e| matches!(e.domain.as_str(), "scene" | "script"))
        .filter(|e| {
            let tail = e.entity_id.rsplit('.').next().unwrap_or("");
            tokens.iter().any(|token| token == tail || scene_token(token) == tail)
                || scene_label_whole(tokens, tail)
                || scene_name_hit(tokens, &e.name, home)
                || e.aliases.iter().any(|n| scene_label_whole(tokens, n) || scene_name_hit(tokens, n, home))
        })
        .map(|e| e.entity_id.clone())
        .collect();
    if hits.len() > 1 {
        hits.retain(|id| scene_compact_hit(id, tokens, home));
    }
    let named = mentioned || catalog().any(tokens, catalog().scene_named());
    let strong = hits.iter().any(|id| scene_compact_hit(id, tokens, home));
    (hits.len() == 1 && (named || strong || tokens.iter().any(|t| t.len() > 5))).then_some(hits.pop()).flatten()
}

fn scene_label_whole(tokens: &[String], label: &str) -> bool {
    let label = compact(label);
    let parts: Vec<String> = tokens.iter().map(|token| compact(token)).filter(|token| !token.is_empty()).collect();
    if parts.iter().any(|token| token == &label) {
        return label.len() > 3 || (label.len() >= 2 && !short_scene_alias_is_generic(&label));
    }
    label.len() > 3 && (2..=parts.len()).any(|width| parts.windows(width).any(|window| window.join("") == label))
}

fn short_scene_alias_is_generic(label: &str) -> bool {
    catalog().verb(label).is_some()
        || catalog().generic().contains(&label)
        || catalog().weak_scene().contains(label)
        || catalog().light_nouns().contains(label)
        || matches!(label, "all" | "alle" | "alles" | "aus" | "off" | "on" | "an")
}

fn scene_compact_hit(id: &str, tokens: &[String], home: &HomeGraph) -> bool {
    let tail = id.rsplit('.').next().unwrap_or("");
    scene_label_whole(tokens, tail)
        || home.entities.iter().any(|entity| {
            entity.entity_id == id
                && (scene_name_hit(tokens, &entity.name, home)
                    || entity.aliases.iter().any(|alias| scene_label_whole(tokens, alias) || scene_name_hit(tokens, alias, home)))
        })
}

fn scene_token(token: &str) -> String {
    let mapped = catalog().scene_token(token);
    if mapped == token {
        fold_umlaut(token)
    } else {
        mapped
    }
}

fn scene_name_hit(tokens: &[String], name: &str, home: &HomeGraph) -> bool {
    let parts: Vec<String> = fold_umlaut(name)
        .split_whitespace()
        .map(scene_token)
        .filter(|p| p.len() > 3 && !catalog().weak_scene().contains(p.as_str()) && scene_distinctive(p, home))
        .collect();
    if parts.is_empty() {
        return false;
    }
    let mapped: Vec<String> = tokens.iter().map(|t| scene_token(t)).collect();
    let allow_fuzzy = tokens.iter().any(|token| catalog().verb(token).is_some());
    let mut repairs = 0;
    parts.iter().all(|part| {
        if mapped.iter().any(|token| token == part) {
            return true;
        }
        if !allow_fuzzy {
            return false;
        }
        let fuzzy = mapped
            .iter()
            .filter(|token| token.len() >= 4)
            .filter_map(|token| evidence(token, part, Profile::Target))
            .max_by(|left, right| left.score.partial_cmp(&right.score).unwrap_or(std::cmp::Ordering::Equal));
        if fuzzy.is_some() {
            repairs += 1;
        }
        fuzzy.is_some() && repairs <= 1
    })
}

fn scene_distinctive(part: &str, home: &HomeGraph) -> bool {
    if catalog().light_nouns().contains(part) || catalog().generic().contains(&part) {
        return false;
    }
    let folded = compact(part);
    !folded.is_empty() && !home.areas.iter().any(|a| compact(&a.area_id) == folded || compact(&a.name) == folded)
}
