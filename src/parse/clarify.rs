use crate::lang::catalog;
use crate::parse::infer::fixture_aliases;
use crate::parse::normalize::tokenize;
use crate::session::Session;
use crate::types::HomeGraph;

/// Which offered option the reply names. Options are matched word for word: "office"
/// picks the office, "off" does not, and a reply that fits two options picks none.
/// "the first" picks the first; a plain "yes" picks nothing, the question was either-or.
pub(crate) fn pick_clarification(tokens: &[String], session: &Session, home: &HomeGraph) -> Option<String> {
    let pending = &session.pending_clarify()?.options;
    if pending.is_empty() {
        return None;
    }
    let cat = catalog();
    if tokens.iter().any(|t| cat.clarify_pick().contains(t.as_str()) && !cat.is_affirm(t)) {
        return pending.first().cloned();
    }
    let mut best: Option<(usize, &String)> = None;
    for id in pending {
        let score = clarify_option_score(id, tokens, home);
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

fn clarify_option_score(id: &str, tokens: &[String], home: &HomeGraph) -> usize {
    if let Some(area) = home.areas.iter().find(|area| area.area_id == id) {
        let mut score = phrase_score(tokens, &area.name);
        for alias in &area.aliases {
            score = score.max(phrase_score(tokens, alias));
        }
        score = score.max(floor_score(area, tokens, home));
        return score.max(id_tail_score(id, tokens));
    }
    if let Some(entity) = home.entities.iter().find(|entity| entity.entity_id == id) {
        let mut score = phrase_score(tokens, &entity.name);
        for alias in &entity.aliases {
            score = score.max(phrase_score(tokens, alias));
        }
        return score.max(id_tail_score(id, tokens));
    }
    id_tail_score(id, tokens)
}

fn floor_score(area: &crate::types::AreaRec, tokens: &[String], home: &HomeGraph) -> usize {
    let Some(floor_id) = area.floor_id.as_deref() else {
        return 0;
    };
    let Some(floor) = home.floors.iter().find(|floor| floor.floor_id == floor_id) else {
        return 0;
    };
    let mut score = phrase_score(tokens, &floor.name).max(phrase_score(tokens, &floor.floor_id.replace('_', " ")));
    for alias in &floor.aliases {
        score = score.max(phrase_score(tokens, alias)).max(short_alias_score(tokens, alias));
    }
    score
}

/// A floor alias may be as short as "eg". It counts when it is a word of the reply.
fn short_alias_score(tokens: &[String], alias: &str) -> usize {
    let words = tokenize(alias);
    if stands_in(tokens, &words) {
        words.iter().map(String::len).sum::<usize>().max(1) * 4
    } else {
        0
    }
}

/// The label stands in the reply as whole words, in order. Filler words inside the
/// label ("the den") may be missing, and a compound may be written as one word.
fn phrase_score(tokens: &[String], phrase: &str) -> usize {
    let words = tokenize(phrase);
    let weight = words.iter().map(String::len).sum::<usize>();
    if weight < 3 {
        return 0;
    }
    let spoken: Vec<String> = words.iter().filter(|word| !catalog().is_filler(word)).cloned().collect();
    let joined = words.concat();
    let named = stands_in(tokens, &words) || stands_in(tokens, &spoken) || tokens.iter().any(|token| *token == joined);
    if named {
        weight * 4
    } else {
        0
    }
}

fn stands_in(tokens: &[String], words: &[String]) -> bool {
    !words.is_empty() && tokens.windows(words.len()).any(|window| window == words)
}

fn id_tail_score(id: &str, tokens: &[String]) -> usize {
    let tail = tokenize(&id.rsplit('.').next().unwrap_or(id).replace('_', " "));
    if stands_in(tokens, &tail) {
        return 1;
    }
    let hit =
        tokens.iter().any(|token| fixture_aliases(token).iter().any(|alias| alias.len() > 2 && tail.iter().any(|part| part == alias)));
    usize::from(hit)
}
