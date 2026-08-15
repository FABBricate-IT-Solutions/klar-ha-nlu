use crate::lang::catalog;
use crate::parse::resolve::mentions_home;
use crate::session::Session;
use crate::types::HomeGraph;

pub fn wants_llm(tokens: &[String], home: &HomeGraph) -> bool {
    if tokens.is_empty() || looks_like_home(tokens, home) {
        return false;
    }
    is_casual(tokens) || is_special(tokens) || is_open_question(tokens)
}

pub fn is_news(tokens: &[String], home: &HomeGraph) -> bool {
    if tokens.is_empty() || looks_like_home(tokens, home) {
        return false;
    }
    catalog().any(tokens, &catalog().chat_news)
}

pub fn is_news_dismiss(tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let cat = catalog();
    if cat.any(tokens, &cat.chat_news) {
        return false;
    }
    let closing = cat.any(tokens, &cat.chat_news_dismiss) || cat.any(tokens, &cat.chat_thanks);
    if !closing {
        return false;
    }
    tokens.iter().all(|t| {
        cat.chat_news_dismiss.contains(t.as_str())
            || cat.chat_thanks.contains(t.as_str())
            || cat.fillers.contains(t.as_str())
            || cat.particles.contains(t.as_str())
            || cat.affirm.contains(t.as_str())
    })
}

pub fn briefing_followup(tokens: &[String], home: &HomeGraph, session: &Session) -> bool {
    session.briefing && !tokens.is_empty() && !looks_like_home(tokens, home) && !is_news_dismiss(tokens)
}

pub(crate) fn looks_like_home(tokens: &[String], home: &HomeGraph) -> bool {
    mentions_home(tokens, home)
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
    use crate::home::default_home;

    fn toks(text: &str) -> Vec<String> {
        crate::parse::normalize::tokenize(text)
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

    #[test]
    fn news_nouns_are_news_not_home() {
        let home = default_home();
        for text in
            ["Was sind die aktuellen Nachrichten", "Was sind die aktuellen News", "aktuelle Schlagzeilen", "What is the latest news"]
        {
            assert!(is_news(&toks(text), &home), "{text}");
            assert!(!looks_like_home(&toks(text), &home), "{text}");
        }
        assert!(!is_news(&toks("Licht im Wohnzimmer an"), &home));
        assert!(!is_news(&toks("Was ist die Hauptstadt von Frankreich"), &home));
        assert!(!is_news(&toks("Wie ist das Wetter"), &home));
    }

    #[test]
    fn news_dismiss_is_short_close() {
        assert!(is_news_dismiss(&toks("nein")));
        assert!(is_news_dismiss(&toks("nein danke")));
        assert!(is_news_dismiss(&toks("das reicht")));
        assert!(is_news_dismiss(&toks("danke")));
        assert!(!is_news_dismiss(&toks("nein die erste")));
        assert!(!is_news_dismiss(&toks("mehr zur ersten Meldung")));
    }
}
