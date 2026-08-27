use crate::home::expose::assist_visible;
use crate::home::roles::is_light_like;
use crate::lang::catalog;
use crate::parse::normalize::{compact, fold_umlaut};
use crate::parse::resolve::token_hit;
use crate::types::HomeGraph;

pub(crate) fn query_grounded(tokens: &[String], home: &HomeGraph, has_target: bool) -> bool {
    has_target || mentions_home(tokens, home)
}

pub(crate) fn mentions_home(tokens: &[String], home: &HomeGraph) -> bool {
    let cat = catalog();
    if cat.any(tokens, cat.temp_query())
        || cat.any(tokens, cat.light_nouns())
        || cat.any(tokens, cat.climate_nouns())
        || cat.any(tokens, cat.cover_nouns())
        || cat.any(tokens, cat.fan_nouns())
        || cat.any(tokens, cat.lock_nouns())
        || cat.any(tokens, cat.vacuum_nouns())
        || cat.any(tokens, cat.media_nouns())
        || cat.any(tokens, cat.timer_nouns())
        || cat.any(tokens, cat.list_nouns())
        || crate::parse::calendar::mentions_calendar(tokens)
        || cat.any(tokens, cat.scene_nouns())
        || cat.any(tokens, cat.named_device())
        || cat.any(tokens, cat.on_words())
        || cat.any(tokens, cat.off_words())
        || cat.any(tokens, cat.laundry_machines())
        || cat.any(tokens, cat.status_words())
    {
        return true;
    }
    if home.entities.iter().any(crate::home::roles::is_music_player)
        && (tokens.windows(2).any(|w| matches!((w[0].as_str(), w[1].as_str()), ("was", "laeuft") | ("was", "spielt")))
            || tokens.windows(3).any(|w| matches!((w[0].as_str(), w[1].as_str(), w[2].as_str()), ("what", "s", "playing")))
            || tokens.iter().any(|t| matches!(t.as_str(), "queue" | "warteschlange")))
    {
        return true;
    }
    if home.areas.iter().any(|area| {
        std::iter::once(compact(&area.area_id))
            .chain(std::iter::once(compact(&area.name)))
            .chain(area.aliases.iter().map(|alias| compact(alias)))
            .any(|name| !name.is_empty() && tokens.iter().any(|token| token == &name))
    }) || home.floors.iter().any(|floor| {
        std::iter::once(compact(&floor.floor_id))
            .chain(std::iter::once(compact(&floor.name)))
            .chain(floor.aliases.iter().map(|alias| compact(alias)))
            .any(|name| !name.is_empty() && tokens.iter().any(|token| token == &name || token_hit(tokens, &name)))
    }) {
        return true;
    }
    home.entities.iter().filter(|entity| assist_visible(entity, home)).any(|entity| {
        let name = fold_umlaut(&entity.name);
        tokens.iter().any(|token| {
            token.len() > 3
                && !cat.is_question_start(token)
                && !cat.is_question_word(token)
                && (name.split([' ', '_']).any(|part| part == token) || entity.aliases.iter().any(|alias| alias == token))
        })
    })
}

pub(crate) fn light_rooms_for_clarify(home: &HomeGraph) -> Vec<String> {
    home.areas
        .iter()
        .filter(|area| !crate::home::policy::is_whole_home(area))
        .filter(|area| {
            home.entities.iter().any(|entity| {
                assist_visible(entity, home) && is_light_like(entity, catalog()) && entity.area.as_deref() == Some(area.area_id.as_str())
            })
        })
        .map(|area| area.area_id.clone())
        .collect()
}
