use crate::lang::catalog;
use crate::parse::news::is_news;
use crate::parse::resolve::mentions_home;
use crate::types::HomeGraph;

pub fn wants_llm(tokens: &[String], home: &HomeGraph) -> bool {
    if tokens.is_empty() || looks_like_home(tokens, home) {
        return false;
    }
    is_casual(tokens) || is_special(tokens)
}

pub fn is_ood(tokens: &[String], home: &HomeGraph) -> bool {
    if tokens.is_empty() || looks_like_home(tokens, home) || wants_llm(tokens, home) || is_news(tokens, home) || has_home_domain_cue(tokens)
    {
        return false;
    }
    is_world_or_advice(tokens) || is_open_question(tokens)
}

fn has_home_domain_cue(tokens: &[String]) -> bool {
    let cat = catalog();
    cat.any(tokens, cat.media_nouns())
        || cat.any(tokens, cat.timer_nouns())
        || cat.any(tokens, cat.list_nouns())
        || crate::parse::calendar::mentions_calendar(tokens)
        || cat.any(tokens, cat.light_nouns())
        || cat.any(tokens, cat.cover_nouns())
        || cat.any(tokens, cat.lock_nouns())
        || cat.any(tokens, cat.climate_nouns())
        || cat.any(tokens, cat.tv_words())
        || tokens.iter().any(|token| {
            matches!(token.as_str(), "queue" | "playing" | "volume" | "lautstarke" | "wiedergabe" | "prozent" | "percent" | "percentage")
        })
}

pub(crate) fn looks_like_home(tokens: &[String], home: &HomeGraph) -> bool {
    mentions_home(tokens, home)
}

fn is_casual(tokens: &[String]) -> bool {
    let cat = catalog();
    cat.any(tokens, cat.chat_greet())
        || cat.any(tokens, cat.chat_thanks())
        || cat.any(tokens, cat.chat_feeling())
        || cat.any(tokens, cat.chat_identity())
        || is_identity_question(tokens)
        || is_wellbeing_question(tokens)
}

fn is_identity_question(tokens: &[String]) -> bool {
    let who = tokens.iter().any(|token| matches!(token.as_str(), "wer" | "who"));
    let self_ref = tokens.iter().any(|token| matches!(token.as_str(), "du" | "you" | "bist" | "are"));
    who && self_ref
}

fn is_wellbeing_question(tokens: &[String]) -> bool {
    if is_world_or_advice(tokens) {
        return false;
    }
    let how = tokens.iter().any(|token| matches!(token.as_str(), "how" | "wie"));
    let feeling = tokens.iter().any(|token| matches!(token.as_str(), "are" | "you" | "geht" | "gehts" | "feeling" | "mood"));
    how && feeling
}

fn is_special(tokens: &[String]) -> bool {
    let cat = catalog();
    (cat.any(tokens, cat.chat_tell())
        && (cat.any(tokens, cat.chat_yarn()) || tokens.iter().any(|t| t.contains("witz") || t.contains("joke"))))
        || cat.any(tokens, cat.chat_yarn())
}

fn is_world_or_advice(tokens: &[String]) -> bool {
    let cat = catalog();
    cat.any(tokens, cat.chat_world()) || cat.any(tokens, cat.chat_advice())
}

fn is_open_question(tokens: &[String]) -> bool {
    let cat = catalog();
    tokens.first().is_some_and(|t| cat.is_question_start(t))
        || tokens.iter().any(|t| cat.is_question_word(t) || cat.chat_open().contains(t.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::default_home;
    use crate::parse::news::briefing_followup;
    use crate::session::Session;

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
            "Wer bist du",
            "Unterhalte mich",
            "Tell a joke",
            "How are you",
        ] {
            assert!(wants_llm(&toks(text), &home), "{text}");
            assert!(!is_ood(&toks(text), &home), "{text}");
        }
    }

    #[test]
    fn ood_world_questions_are_not_llm() {
        let home = default_home();
        for text in [
            "Wie ist das Wetter",
            "Was ist die Hauptstadt von Frankreich",
            "Was soll ich kochen",
            "What's the weather",
            "What is the capital of France",
        ] {
            assert!(!wants_llm(&toks(text), &home), "{text}");
            assert!(is_ood(&toks(text), &home), "{text}");
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
    fn llm_chat_keeps_story_replies() {
        let home = default_home();
        let mut session = Session::new();
        session.last_heard = Some(crate::session::LastHeard {
            text: "erzähl eine geschichte".into(),
            decision: "chat".into(),
            speech: String::new(),
            reason: None,
            area: Some("wohnzimmer".into()),
            names: Vec::new(),
        });
        for text in ["über einen Elefanten", "länger", "egal", "science fiction"] {
            assert!(briefing_followup(&toks(text), &home, &session), "{text}");
        }
        for text in ["Licht im Wohnzimmer aus", "mach sie aus"] {
            assert!(!briefing_followup(&toks(text), &home, &session), "{text}");
        }
        session.briefing = true;
        assert!(!briefing_followup(&toks("egal"), &home, &session));
    }
}
