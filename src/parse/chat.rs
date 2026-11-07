use crate::lang::catalog;
use crate::parse::resolve::mentions_home;
use crate::session::Session;
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

pub fn is_news(tokens: &[String], home: &HomeGraph) -> bool {
    if tokens.is_empty() || looks_like_home(tokens, home) {
        return false;
    }
    catalog().any(tokens, catalog().chat_news()) || tokens.iter().any(|token| compound_stem(token).is_some())
}

const TOPIC_GLUE: &[&str] = &[
    "was",
    "what",
    "whats",
    "gibt",
    "gib",
    "sind",
    "ist",
    "is",
    "are",
    "there",
    "any",
    "aktuelle",
    "aktuellen",
    "aktueller",
    "aktuelles",
    "latest",
    "current",
    "heute",
    "today",
    "some",
    "give",
    "get",
    "lass",
    "let",
    "ueber",
    "about",
    "on",
    "aus",
    "from",
    "zum",
    "zur",
    "short",
    "neue",
    "neuer",
    "neuen",
    "tell",
    "hor",
    "hear",
    "sag",
    "say",
    "read",
    "the",
    "a",
    "an",
    "one",
    "ones",
    "eine",
    "einen",
    "einer",
    "einem",
    "eines",
];

/// Leftover words after the news cue. `sportnachrichten` yields `sport`.
pub fn news_topic(tokens: &[String]) -> Option<String> {
    if tokens.is_empty() {
        return None;
    }
    let marked = news_marks(tokens);
    let mut parts = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if marked[index] {
            continue;
        }
        if let Some(stem) = compound_stem(token) {
            if !topic_glue(&stem) {
                parts.push(stem);
            }
            continue;
        }
        if topic_glue(token) {
            continue;
        }
        parts.push(token.clone());
    }
    let topic = parts.join(" ");
    if topic.is_empty() {
        None
    } else {
        Some(topic)
    }
}

fn news_marks(tokens: &[String]) -> Vec<bool> {
    let news = catalog().chat_news();
    let mut hit = vec![false; tokens.len()];
    for phrase in news {
        let parts: Vec<&str> = phrase.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        if parts.len() == 1 {
            for (index, token) in tokens.iter().enumerate() {
                if token == parts[0] {
                    hit[index] = true;
                }
            }
            continue;
        }
        if tokens.len() < parts.len() {
            continue;
        }
        for start in 0..=tokens.len() - parts.len() {
            if tokens[start..start + parts.len()].iter().map(String::as_str).eq(parts.iter().copied()) {
                hit[start..start + parts.len()].fill(true);
            }
        }
    }
    hit
}

fn compound_stem(token: &str) -> Option<String> {
    let news = catalog().chat_news();
    let mut best: Option<&str> = None;
    for word in news {
        if word.contains(' ') || word.len() < 4 || token.len() < word.len() + 3 || !token.ends_with(word) {
            continue;
        }
        if best.is_none_or(|current| word.len() > current.len()) {
            best = Some(word);
        }
    }
    best.map(|word| token[..token.len() - word.len()].to_string())
}

fn topic_glue(token: &str) -> bool {
    let cat = catalog();
    TOPIC_GLUE.contains(&token)
        || FOLLOW_CUES.contains(&token)
        || cat.fillers().contains(token)
        || cat.particles().contains(token)
        || cat.question_words().contains(token)
        || cat.affirm().contains(token)
        || cat.chat_thanks().contains(token)
        || cat.conjunctions().contains(token)
        || cat.chat_news_dismiss().contains(token)
}

pub fn is_news_dismiss(tokens: &[String]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let cat = catalog();
    if cat.any(tokens, cat.chat_news()) {
        return false;
    }
    let closing = cat.any(tokens, cat.chat_news_dismiss()) || cat.any(tokens, cat.chat_thanks());
    if !closing {
        return false;
    }
    tokens.iter().all(|t| {
        cat.chat_news_dismiss().contains(t.as_str())
            || cat.chat_thanks().contains(t.as_str())
            || cat.fillers().contains(t.as_str())
            || cat.particles().contains(t.as_str())
            || cat.affirm().contains(t.as_str())
    })
}

/// „Mehr zur ersten Meldung“ names a story inside the briefing. It is not a new topic.
pub fn news_story_followup(tokens: &[String], home: &HomeGraph, session: &Session) -> bool {
    session.briefing
        && is_news(tokens, home)
        && tokens.iter().any(|token| FOLLOW_CUES.contains(&token.as_str()))
        && news_topic(tokens).is_none()
}

const FOLLOW_CUES: &[&str] = &[
    "mehr", "more", "erste", "ersten", "erster", "erstes", "first", "zweite", "zweiten", "zweiter", "zweites", "second", "dritte",
    "dritten", "dritter", "drittes", "third", "weitere", "weiteren", "weiterer", "weiteres", "another", "dazu", "letzte", "letzten",
    "letzter", "letztes", "last",
];

pub fn briefing_followup(tokens: &[String], home: &HomeGraph, session: &Session) -> bool {
    if !in_llm_turn(session) || tokens.is_empty() || looks_like_home(tokens, home) {
        return false;
    }
    if session.briefing && is_news_dismiss(tokens) {
        return false;
    }
    true
}

fn in_llm_turn(session: &Session) -> bool {
    session.briefing || session.last_heard.as_ref().is_some_and(|heard| heard.decision == "chat")
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
        for text in ["Was gibt es für Neuigkeiten", "was gibt es Neues", "what's new", "whats new"] {
            assert!(is_news(&toks(text), &home), "{text}");
            assert!(news_topic(&toks(text)).is_none(), "{text} {:?}", news_topic(&toks(text)));
        }
        assert_eq!(news_topic(&toks("Sportnachrichten")).as_deref(), Some("sport"));
        assert_eq!(news_topic(&toks("Nachrichten über Sport")).as_deref(), Some("sport"));
        assert_eq!(news_topic(&toks("news about sports")).as_deref(), Some("sports"));
        assert!(news_topic(&toks("mehr zur ersten Meldung")).is_none());
        assert!(news_topic(&toks("mehr über die letzte Meldung")).is_none());
        assert!(news_topic(&toks("letzte Nachrichten")).is_none());
    }

    #[test]
    fn generated_packs_use_news_words_not_the_off_verb() {
        let home = default_home();
        let _es = crate::lang::bind(&["es".into()]);
        assert!(is_news(&toks("noticias"), &home));
        assert!(!is_news(&toks("apaga"), &home));
        drop(_es);
        let _at = crate::lang::bind(&["de-AT".into()]);
        assert!(is_news(&toks("Nachrichten"), &home));
        assert!(!is_news(&toks("schalt"), &home));
        assert_eq!(news_topic(&toks("Sportnachrichten")).as_deref(), Some("sport"));
    }

    #[test]
    fn sport_news_sets_the_topic_on_the_outcome() {
        let home = default_home();
        let mut session = Session::new();
        let settings = crate::types::Settings::pinned("de");
        let sport = crate::nlu::parse("Sportnachrichten", &home, &mut session, &[], &settings);
        assert!(sport.briefing, "{}", sport.speech);
        assert_eq!(sport.news_topic.as_deref(), Some("sport"));
        assert_eq!(session.news_topic.as_deref(), Some("sport"));
        let about = crate::nlu::parse("Nachrichten über Sport", &home, &mut Session::new(), &[], &settings);
        assert_eq!(about.news_topic.as_deref(), Some("sport"));
        let plain = crate::nlu::parse("Was gibt es für Neuigkeiten", &home, &mut Session::new(), &[], &settings);
        assert!(plain.briefing, "{}", plain.speech);
        assert!(plain.news_topic.is_none(), "{:?}", plain.news_topic);
        let follow = crate::nlu::parse("erzähl mehr", &home, &mut session, &[], &settings);
        assert!(follow.briefing);
        assert_eq!(follow.news_topic.as_deref(), Some("sport"));
        let story = crate::nlu::parse("mehr zur ersten Meldung", &home, &mut session, &[], &settings);
        assert!(story.briefing, "{}", story.speech);
        assert_eq!(story.news_topic.as_deref(), Some("sport"));
        assert_eq!(session.news_topic.as_deref(), Some("sport"));
        let last = crate::nlu::parse("mehr über die letzte Meldung", &home, &mut session, &[], &settings);
        assert!(last.briefing, "{}", last.speech);
        assert_eq!(last.news_topic.as_deref(), Some("sport"));
        assert_eq!(session.news_topic.as_deref(), Some("sport"));
        let fresh = crate::nlu::parse("Was gibt es für Neuigkeiten", &home, &mut session, &[], &settings);
        assert!(fresh.briefing, "{}", fresh.speech);
        assert!(fresh.news_topic.is_none(), "{:?}", fresh.news_topic);
        assert!(session.news_topic.is_none());
    }

    #[test]
    fn short_news_words_stay_out_of_everyday_speech() {
        let home = default_home();
        let _lv = crate::lang::bind(&["lv".into()]);
        assert!(is_news(&toks("zinas"), &home));
        assert!(!is_news(&toks("zina"), &home));
        drop(_lv);
        let _gl = crate::lang::bind(&["gl".into()]);
        assert!(is_news(&toks("novas"), &home));
        assert!(!is_news(&toks("nova"), &home));
        drop(_gl);
        let _cy = crate::lang::bind(&["cy".into()]);
        assert!(is_news(&toks("newyddion"), &home));
        assert!(!is_news(&toks("newydd"), &home));
        drop(_cy);
        let _el = crate::lang::bind(&["el".into()]);
        assert!(is_news(&toks("ειδήσεις"), &home));
        assert!(!is_news(&toks("νέα"), &home));
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
