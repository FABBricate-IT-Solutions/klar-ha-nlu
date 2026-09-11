use crate::lang::catalog;
use crate::parse::infer::fixture_aliases;
use crate::parse::normalize::{fold_umlaut, join_tokens};
use crate::session::Session;
use crate::types::HomeGraph;

pub(crate) fn pick_clarification(tokens: &[String], session: &Session, home: &HomeGraph) -> Option<String> {
    let pending = &session.pending_clarify()?.options;
    if pending.is_empty() {
        return None;
    }
    if tokens.iter().any(|t| catalog().clarify_pick().contains(t.as_str())) {
        return pending.first().cloned();
    }
    let blob = join_tokens(tokens);
    let mut best: Option<(usize, &String)> = None;
    for id in pending {
        let score = clarify_option_score(id, &blob, tokens, home);
        if score == 0 {
            continue;
        }
        match best {
            None => best = Some((score, id)),
            Some((prev, _)) if score > prev => best = Some((score, id)),
            Some((prev, _)) if score == prev => return None,
            Some(_) => {}
        }
    }
    best.map(|(_, id)| id.clone())
}

fn clarify_option_score(id: &str, blob: &str, tokens: &[String], home: &HomeGraph) -> usize {
    if let Some(area) = home.areas.iter().find(|area| area.area_id == id) {
        let mut score = phrase_score(blob, &area.name);
        for alias in &area.aliases {
            score = score.max(phrase_score(blob, alias));
        }
        return score.max(id_tail_score(id, blob, tokens));
    }
    if let Some(entity) = home.entities.iter().find(|entity| entity.entity_id == id) {
        let mut score = phrase_score(blob, &entity.name);
        for alias in &entity.aliases {
            score = score.max(phrase_score(blob, alias));
        }
        return score.max(id_tail_score(id, blob, tokens));
    }
    id_tail_score(id, blob, tokens)
}

fn phrase_score(blob: &str, phrase: &str) -> usize {
    let folded = fold_umlaut(phrase);
    if folded.len() < 3 {
        return 0;
    }
    if blob == folded || blob.split_whitespace().any(|word| word == folded) || blob.contains(&folded) {
        return folded.len() * 4;
    }
    0
}

fn id_tail_score(id: &str, blob: &str, tokens: &[String]) -> usize {
    let tail = id.rsplit('.').next().unwrap_or(id).replace('_', " ");
    let folded = fold_umlaut(&tail);
    if blob.contains(&folded) {
        return 1;
    }
    let hit = tokens.iter().any(|t| {
        fixture_aliases(t)
            .iter()
            .any(|a| (folded.contains(a) && a.len() > 2) || tail.split_whitespace().any(|p| a.contains(p) && p.len() > 2))
    });
    usize::from(hit)
}
