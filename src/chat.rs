use crate::lang::catalog;
use crate::normalize::compact;
use crate::resolve::query_grounded;
use crate::session::Session;
use crate::types::HomeGraph;

pub fn wants_llm(tokens: &[String], home: &HomeGraph) -> bool {
    if tokens.is_empty() || looks_like_home(tokens, home) {
        return false;
    }
    is_casual(tokens) || is_special(tokens) || is_open_question(tokens)
}

fn looks_like_home(tokens: &[String], home: &HomeGraph) -> bool {
    let cat = catalog();
    if cat.any(tokens, &cat.light_nouns)
        || cat.any(tokens, &cat.climate_nouns)
        || cat.any(tokens, &cat.cover_nouns)
        || cat.any(tokens, &cat.fan_nouns)
        || cat.any(tokens, &cat.lock_nouns)
        || cat.any(tokens, &cat.vacuum_nouns)
        || cat.any(tokens, &cat.media_nouns)
        || cat.any(tokens, &cat.timer_nouns)
        || cat.any(tokens, &cat.list_nouns)
        || cat.any(tokens, &cat.scene_nouns)
        || cat.any(tokens, &cat.named_device)
        || cat.any(tokens, &cat.temp_query)
        || cat.any(tokens, &cat.on_words)
        || cat.any(tokens, &cat.off_words)
        || cat.any(tokens, &cat.laundry_machines)
        || cat.any(tokens, &cat.status_words)
    {
        return true;
    }
    if mentions_area(tokens, home) {
        return true;
    }
    query_grounded(tokens, home, false, &Session::new())
}

fn mentions_area(tokens: &[String], home: &HomeGraph) -> bool {
    home.areas.iter().any(|area| {
        let mut names = std::iter::once(compact(&area.area_id))
            .chain(std::iter::once(compact(&area.name)))
            .chain(area.aliases.iter().map(|alias| compact(alias)));
        names.any(|name| !name.is_empty() && tokens.iter().any(|t| t == &name))
    })
}

fn is_casual(tokens: &[String]) -> bool {
    let cat = catalog();
    cat.any(tokens, &cat.chat_greet)
        || cat.any(tokens, &cat.chat_thanks)
        || cat.any(tokens, &cat.chat_feeling)
        || cat.any(tokens, &cat.chat_identity)
}

fn is_special(tokens: &[String]) -> bool {
    let cat = catalog();
    (cat.any(tokens, &cat.chat_tell)
        && (cat.any(tokens, &cat.chat_yarn) || tokens.iter().any(|t| t.contains("witz") || t.contains("joke"))))
        || cat.any(tokens, &cat.chat_yarn)
        || cat.any(tokens, &cat.chat_world)
        || cat.any(tokens, &cat.chat_advice)
}

fn is_open_question(tokens: &[String]) -> bool {
    let cat = catalog();
    tokens.first().is_some_and(|t| cat.is_question_start(t))
        || tokens.iter().any(|t| cat.is_question_word(t) || cat.chat_open.contains(t.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::default_home;

    fn toks(text: &str) -> Vec<String> {
        crate::normalize::tokenize(text)
    }

    #[test]
    fn chat_examples_go_to_llm() {
        let home = default_home();
        for text in [
            "Erzähle eine Geschichte",
            "Erzähle einen Witz",
            "Wie geht es dir",
            "Guten Morgen",
            "Danke",
            "Was ist die Hauptstadt von Frankreich",
            "Wie ist das Wetter",
            "Was soll ich kochen",
            "Wer bist du",
            "Unterhalte mich",
            "Tell a joke",
            "How are you",
        ] {
            assert!(wants_llm(&toks(text), &home), "{text}");
        }
    }

    #[test]
    fn home_stays_on_nlu() {
        let home = default_home();
        for text in [
            "Licht im Wohnzimmer an",
            "Wie ist der Status der Küche",
            "Wie warm ist es im Schlafzimmer",
            "mach es aus",
            "Timer eine Minute",
            "Wo ist R2D2",
        ] {
            assert!(!wants_llm(&toks(text), &home), "{text}");
        }
    }

    #[test]
    fn garbage_is_not_chat() {
        let home = default_home();
        assert!(!wants_llm(&toks("asdfghjkl qwerty"), &home));
    }
}
