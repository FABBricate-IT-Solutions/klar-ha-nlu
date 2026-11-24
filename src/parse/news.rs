use std::collections::HashSet;

use crate::lang::catalog;
use crate::parse::calendar::agenda_verb;
use crate::parse::normalize::is_script_unit;
use crate::parse::resolve::mentions_home;
use crate::session::Session;
use crate::types::HomeGraph;

/// What a sentence with a news word asks for.
#[derive(Debug, PartialEq, Eq)]
pub enum NewsTurn {
    /// A new roundup, on a topic when one is named.
    Fresh(Option<String>),
    /// A question about the roundup that is open.
    Follow,
}

pub fn is_news(tokens: &[String], home: &HomeGraph) -> bool {
    let tokens = &regroup(tokens);
    !tokens.is_empty() && has_news_word(tokens) && !mentions_home(&calm(tokens, true), home)
}

pub fn news_turn(tokens: &[String], home: &HomeGraph, session: &Session) -> Option<NewsTurn> {
    if !is_news(tokens, home) {
        return None;
    }
    let tokens = &regroup(tokens);
    let topic = topic_of(tokens);
    if !session.briefing {
        return Some(NewsTurn::Fresh(topic));
    }
    let cat = catalog();
    if cat.any(tokens, cat.chat_news_story()) {
        return Some(NewsTurn::Follow);
    }
    if topic.is_some() {
        return Some(NewsTurn::Fresh(topic));
    }
    let marked = marks(tokens, cat.chat_news());
    let plain =
        tokens.iter().enumerate().all(|(index, token)| marked[index] || (noise(token) && !cat.chat_news_follow().contains(token.as_str())));
    Some(if plain { NewsTurn::Fresh(None) } else { NewsTurn::Follow })
}

/// A chat turn is open and the sentence is not about the home: it goes back to the model.
pub fn briefing_followup(tokens: &[String], home: &HomeGraph, session: &Session) -> bool {
    if !in_llm_turn(session) || tokens.is_empty() {
        return false;
    }
    let tokens = &regroup(tokens);
    let homeish = if session.briefing || has_news_word(tokens) {
        mentions_home(&calm(tokens, has_news_word(tokens)), home)
    } else {
        mentions_home(tokens, home)
    };
    !(homeish || session.briefing && is_news_dismiss(tokens))
}

fn in_llm_turn(session: &Session) -> bool {
    session.briefing || session.last_heard.as_ref().is_some_and(|heard| heard.decision == "chat")
}

pub fn is_news_dismiss(tokens: &[String]) -> bool {
    let cat = catalog();
    if tokens.is_empty() || cat.any(tokens, cat.chat_news()) {
        return false;
    }
    let mut closing = marks(tokens, cat.chat_news_dismiss());
    for (index, hit) in marks(tokens, cat.chat_thanks()).into_iter().enumerate() {
        closing[index] |= hit;
    }
    closing.contains(&true)
        && tokens.iter().enumerate().all(|(index, token)| {
            closing[index]
                || cat.fillers().contains(token.as_str())
                || cat.particles().contains(token.as_str())
                || cat.affirm().contains(token.as_str())
        })
}

/// What the news should be about. `sportnachrichten` yields `sport`, "news about Tesla" yields `tesla`.
fn topic_of(tokens: &[String]) -> Option<String> {
    let cat = catalog();
    let marked = marks(tokens, cat.chat_news());
    let free = |index: usize| !marked[index] && !noise(&tokens[index]);
    for (index, token) in tokens.iter().enumerate() {
        if let Some(stem) = compound_stem(token).filter(|stem| !marked[index] && !noise(stem)) {
            return Some(section(&stem));
        }
    }
    if let Some(topic) = marker_topic(tokens, &marked) {
        return Some(topic);
    }
    if let Some(query) = section_hit(tokens, &marked) {
        return Some(query.to_string());
    }
    if !cat.news_open {
        return None;
    }
    let rest: Vec<String> = (0..tokens.len())
        .filter(|&index| free(index) && compound_stem(&tokens[index]).is_none())
        .map(|index| tokens[index].clone())
        .collect();
    (!rest.is_empty() && rest.len() <= 3).then(|| join(&rest))
}

/// "sports", "thể thao": a section the pack knows, said without "about".
fn section_hit(tokens: &[String], marked: &[bool]) -> Option<&'static str> {
    let mut best: Option<(usize, usize, &'static str)> = None;
    for (key, query) in &catalog().news_topics {
        let parts: Vec<&str> = key.split_whitespace().collect();
        if parts.is_empty() || tokens.len() < parts.len() {
            continue;
        }
        let found = (0..=tokens.len() - parts.len()).find(|&start| {
            !marked[start..start + parts.len()].contains(&true)
                && tokens[start..start + parts.len()].iter().map(String::as_str).eq(parts.iter().copied())
        });
        // The longest phrase wins, then the one said first, so the pick never depends on map order.
        if let Some(start) = found.filter(|&start| best.is_none_or(|(length, at, _)| (parts.len(), at) > (length, start))) {
            best = Some((parts.len(), start, query));
        }
    }
    best.map(|(_, _, query)| query)
}

/// The words after "about", or the word in front of a postposition such as Japanese "について".
fn marker_topic(tokens: &[String], marked: &[bool]) -> Option<String> {
    let cat = catalog();
    let edge = |index: usize| marked[index] || cat.chat_news_about().contains(tokens[index].as_str()) || after_marker(&tokens[index]);
    let dull = |index: &usize| noise(&tokens[*index]);
    for (index, token) in tokens.iter().enumerate() {
        if marked[index] {
            continue;
        }
        let span: Vec<usize> = if cat.chat_news_about().contains(token.as_str()) {
            (index + 1..tokens.len()).take_while(|&next| !edge(next)).filter(|next| !dull(next)).collect()
        } else if after_marker(token) {
            let mut before: Vec<usize> =
                (0..index).rev().take_while(|&prev| !edge(prev)).skip_while(dull).take_while(|prev| !dull(prev)).collect();
            before.reverse();
            before
        } else {
            continue;
        };
        let words: Vec<String> = span.into_iter().map(|at| section(&tokens[at])).collect();
        let topic = join(&words);
        if (2..=40).contains(&topic.chars().count()) && topic.split(' ').count() <= 4 {
            return Some(topic);
        }
    }
    None
}

fn after_marker(token: &str) -> bool {
    catalog().chat_news_about_after().contains(token)
}

fn section(word: &str) -> String {
    catalog().news_topics.get(word).map_or_else(|| word.to_string(), |query| (*query).to_string())
}

/// Chinese, Japanese and Thai are written without spaces. The search words go back together the same way.
fn join(words: &[String]) -> String {
    let mut out = String::new();
    for word in words {
        let glued = out.chars().last().is_some_and(is_script_unit) && word.chars().next().is_some_and(is_script_unit);
        if !out.is_empty() && !glued {
            out.push(' ');
        }
        out.push_str(word);
    }
    out
}

/// Chinese, Japanese, Korean and Thai arrive cut into single signs. The news words of the pack are put
/// back together here, so "关于" is "about" and not the "关" that switches a lamp off.
fn regroup(tokens: &[String]) -> Vec<String> {
    let script = |token: &String| token.chars().next().is_some_and(is_script_unit);
    if !tokens.iter().any(script) {
        return tokens.to_vec();
    }
    let cat = catalog();
    let known = |word: &str| cue(word) || cat.chat_news_story().contains(word) || cat.news_topics.contains_key(word);
    let mut out = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let mut best = 1;
        if script(&tokens[index]) {
            let mut word = tokens[index].clone();
            for (offset, next) in tokens[index + 1..].iter().take(15).enumerate() {
                if !script(next) {
                    break;
                }
                word.push_str(next);
                if known(&word) {
                    best = offset + 2;
                }
            }
        }
        out.push(tokens[index..index + best].concat());
        index += best;
    }
    out
}

fn has_news_word(tokens: &[String]) -> bool {
    catalog().any(tokens, catalog().chat_news()) || tokens.iter().any(|token| compound_stem(token).is_some())
}

/// Tokens covered by a word or a phrase of the set.
fn marks(tokens: &[String], set: &HashSet<&'static str>) -> Vec<bool> {
    let mut hit = vec![false; tokens.len()];
    for phrase in set {
        let parts: Vec<&str> = phrase.split_whitespace().collect();
        if parts.is_empty() || tokens.len() < parts.len() {
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
    // "الأخبار" is the news with an article, not news about "ال".
    best.map(|word| token[..token.len() - word.len()].to_string()).filter(|stem| stem.chars().count() >= 3)
}

/// Words of the request itself. None of them names a topic.
fn noise(token: &str) -> bool {
    let cat = catalog();
    cue(token)
        || cat.chat_news_story().contains(token)
        || cat.fillers().contains(token)
        || cat.particles().contains(token)
        || cat.question_words().contains(token)
        || cat.is_question_start(token)
        || cat.affirm().contains(token)
        || cat.chat_thanks().contains(token)
        || cat.conjunctions().contains(token)
        || cat.chat_news_dismiss().contains(token)
        || cat.all_words().contains(token)
        || day_word(token)
}

fn cue(token: &str) -> bool {
    let cat = catalog();
    cat.chat_news_glue().contains(token)
        || cat.chat_news_follow().contains(token)
        || cat.chat_news_about().contains(token)
        || cat.chat_news_about_after().contains(token)
}

fn day_word(token: &str) -> bool {
    catalog().calendar_today().contains(token) || catalog().calendar_tomorrow().contains(token)
}

fn day_marks(tokens: &[String]) -> Vec<bool> {
    let mut days = marks(tokens, catalog().calendar_today());
    for (index, hit) in marks(tokens, catalog().calendar_tomorrow()).into_iter().enumerate() {
        days[index] |= hit;
    }
    days
}

/// Everyday words that also steer the home: "heute", "alle", "what", the "on" of "more on that".
/// Beside a news word, or while a briefing is open, they are no proof of a home command.
fn calm(tokens: &[String], news_word: bool) -> Vec<String> {
    let cat = catalog();
    let days = day_marks(tokens);
    let keep_day = !news_word && agenda_verb(tokens);
    let solid = |token: &String| !cat.fillers().contains(token.as_str());
    let weak = |index: usize| {
        let word = tokens[index].as_str();
        if cat.all_words().contains(word) || (days[index] && !keep_day) {
            return true;
        }
        // "Nachrichten an", "put the news on": the news is what should start.
        if news_word && cat.on_words().contains(word) {
            return true;
        }
        if !cue(word) {
            return false;
        }
        let switch = cat.on_words().contains(word) || cat.off_words().contains(word);
        let inside = tokens[..index].iter().any(solid) && tokens[index + 1..].iter().any(solid);
        cat.status_words().contains(word) || (switch && inside)
    };
    (0..tokens.len()).filter(|&index| !weak(index)).map(|index| tokens[index].clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::default_home;
    use crate::parse::resolve::mentions_home as looks_like_home;

    fn toks(text: &str) -> Vec<String> {
        crate::parse::normalize::tokenize(text)
    }

    fn news_topic(tokens: &[String]) -> Option<String> {
        topic_of(&regroup(tokens))
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
        for text in ["lese nachrichten vor", "lies mir die news vor", "nachrichten vorlesen"] {
            assert!(news_topic(&toks(text)).is_none(), "{text} {:?}", news_topic(&toks(text)));
        }
        assert_eq!(news_topic(&toks("lese news sport vor")).as_deref(), Some("sport"));
        assert_eq!(news_topic(&toks("lies die sportnachrichten vor")).as_deref(), Some("sport"));
        assert!(news_topic(&toks("gib mir dazu alle aktuellen nachrichten")).is_none());
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
        let mut open = Session::new();
        let first = crate::nlu::parse("Was sind die aktuellen Nachrichten", &home, &mut open, &[], &settings);
        assert!(first.briefing);
        let back = crate::nlu::parse("gib mir dazu alle aktuellen nachrichten", &home, &mut open, &[], &settings);
        assert!(back.briefing, "{}", back.speech);
        assert!(back.speech.is_empty(), "dazu stays on the briefing, got {}", back.speech);
        assert!(back.news_topic.is_none(), "{:?}", back.news_topic);
        let schools = crate::nlu::parse("zu den schulen bitte mehr", &home, &mut open, &[], &settings);
        assert!(schools.briefing);
        assert!(schools.speech.is_empty(), "{}", schools.speech);
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
}
