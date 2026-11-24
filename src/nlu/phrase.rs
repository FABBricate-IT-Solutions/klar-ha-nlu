//! Household and routine phrases are whole utterances, compared word by word.
//!
//! "movie" is a routine; "what movie is on tonight" is not. A phrase hits when the
//! utterance is that phrase with nothing added but filler words ("please", "the").
//! Questions to the house ("what time is it") may also carry question words. A phrase
//! never matches inside a longer sentence and never inside a word.

use crate::lang::Catalog;
use crate::parse::normalize::{fold_umlaut, tokenize};

/// Words of an utterance or a pack phrase, folded the same way on both sides.
pub(super) fn words(text: &str) -> Vec<String> {
    tokenize(&fold_umlaut(text.trim()).replace(['\'', '’', '`'], ""))
}

/// Which extra words an utterance may carry around a phrase.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Slack {
    /// Commands (routine, undo, teach): filler words only, so a question never triggers one.
    Filler,
    /// Questions (clock, explain): filler and question words ("is", "what").
    Question,
}

fn ignorable(word: &str, catalog: &Catalog, slack: Slack) -> bool {
    catalog.is_filler(word) || (slack == Slack::Question && catalog.is_query_hint(word))
}

/// Walks the phrase through the utterance. Returns where the phrase ended, if every
/// utterance word up to there was a phrase word or ignorable.
fn consume(utterance: &[String], phrase: &[String], catalog: &Catalog, slack: Slack) -> Option<usize> {
    let mut wanted = phrase.iter().peekable();
    wanted.peek()?;
    for (at, word) in utterance.iter().enumerate() {
        if wanted.peek().is_some_and(|next| *next == word) {
            wanted.next();
            if wanted.peek().is_none() {
                return Some(at + 1);
            }
        } else if !ignorable(word, catalog, slack) {
            return None;
        }
    }
    None
}

/// The whole utterance is this phrase.
pub(super) fn is_phrase(utterance: &[String], phrase: &str, catalog: &Catalog, slack: Slack) -> bool {
    let phrase = words(phrase);
    utterance == phrase.as_slice()
        || consume(utterance, &phrase, catalog, slack)
            .is_some_and(|end| utterance[end..].iter().all(|word| ignorable(word, catalog, slack)))
}

pub(super) fn is_any_phrase<'a>(utterance: &[String], mut phrases: impl Iterator<Item = &'a str>, catalog: &Catalog, slack: Slack) -> bool {
    phrases.any(|phrase| is_phrase(utterance, phrase, catalog, slack))
}

/// The utterance opens with this phrase; returns the words after it.
pub(super) fn after_prefix<'a>(utterance: &'a [String], phrase: &str, catalog: &Catalog) -> Option<&'a [String]> {
    consume(utterance, &words(phrase), catalog, Slack::Filler).map(|end| &utterance[end..])
}

/// The phrase stands somewhere in the utterance as whole words.
pub(super) fn has_words(utterance: &[String], phrase: &str) -> bool {
    let phrase = words(phrase);
    !phrase.is_empty() && utterance.windows(phrase.len()).any(|window| window == phrase.as_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::catalog_for;

    fn hit(lang: &str, text: &str, phrase: &str, slack: Slack) -> bool {
        is_phrase(&words(text), phrase, catalog_for(&[lang.into()]), slack)
    }

    #[test]
    fn a_command_phrase_is_the_whole_utterance() {
        let command = |text, phrase| hit("en", text, phrase, Slack::Filler);
        assert!(command("movie", "movie"));
        assert!(command("Movie, please!", "movie"));
        assert!(!command("what movie is on tonight", "movie"));
        assert!(!command("what is the movie", "movie"), "a question does not start a routine");
        assert!(!command("turn on the hall light before the movie", "movie"));
        assert!(!command("movies", "movie"));
        assert!(command("can you undo that please", "undo that"));
        assert!(!command("undo the hall light", "undo"));
        assert!(!command("that undo", "undo that"), "phrase words keep their order");
    }

    #[test]
    fn german_fillers_may_split_a_command_phrase() {
        let command = |text, phrase| hit("de", text, phrase, Slack::Filler);
        assert!(command("Mach das bitte wieder rückgängig", "mach das rueckgaengig"));
        assert!(!command("Mach das Licht nicht rückgängig", "mach das rueckgaengig"));
        assert!(!command("Rückgängigmachen", "rueckgaengig"));
    }

    #[test]
    fn a_question_phrase_may_carry_question_words() {
        let question = |lang, text, phrase| hit(lang, text, phrase, Slack::Question);
        assert!(question("en", "what time is it", "what time"));
        assert!(question("en", "what is the time", "what time"));
        assert!(!question("en", "what time is my appointment", "what time"));
        assert!(question("de", "Wie spät ist es?", "wie spaet"));
        assert!(!question("de", "Wie spät ist mein Termin", "wie spaet"));
    }

    #[test]
    fn prefix_returns_the_rest() {
        let catalog = catalog_for(&["en".into()]);
        let said = words("please call it reading lamp");
        assert_eq!(after_prefix(&said, "call it ", catalog), Some(&said[3..]));
        assert_eq!(after_prefix(&words("recall it reading lamp"), "call it ", catalog), None);
    }

    #[test]
    fn has_words_respects_word_boundaries() {
        assert!(has_words(&words("wie wird das wetter morgen"), "wetter"));
        assert!(!has_words(&words("schalte die wetterstation ein"), "wetter"));
    }
}
