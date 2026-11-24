//! What a short reply to an open question means.
//!
//! Each language brings its own yes, no and cancel phrases. A reply counts only when the
//! whole utterance is made of such phrases: "yes please" is a yes, "yes but only the
//! kitchen" is no answer at all, and neither is a yes and a no in one breath. The words
//! are read as spoken, before filler stripping, because "okay" is a filler elsewhere.

use crate::lang::Catalog;

use super::phrase::words;

/// Longest reply that is still read as an answer.
pub(super) const MAX_ANSWER_WORDS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Answer {
    Yes,
    /// "no" and "cancel" both close the question without acting.
    No,
}

pub(super) fn classify(said: &[String], catalog: &Catalog) -> Option<Answer> {
    if said.is_empty() || said.len() > MAX_ANSWER_WORDS {
        return None;
    }
    let phrases = |pick: fn(&crate::lang::Answers) -> &[&str]| -> Vec<Vec<String>> {
        catalog.answers().flat_map(|answers| pick(answers).iter().map(|phrase| words(phrase))).filter(|phrase| !phrase.is_empty()).collect()
    };
    let mut refusals = phrases(|answers| answers.no);
    refusals.extend(phrases(|answers| answers.cancel));
    // A refusal wins over a reading as consent: "please don't" is a no, although "please"
    // alone is a yes.
    if made_of(said, &refusals) {
        return Some(Answer::No);
    }
    made_of(said, &phrases(|answers| answers.yes)).then_some(Answer::Yes)
}

/// The utterance is one or more of these phrases, back to back, with nothing left over.
fn made_of(said: &[String], phrases: &[Vec<String>]) -> bool {
    let mut reachable = vec![false; said.len() + 1];
    reachable[0] = true;
    for start in 0..said.len() {
        if !reachable[start] {
            continue;
        }
        for phrase in phrases {
            let end = start + phrase.len();
            if end <= said.len() && said[start..end] == phrase[..] {
                reachable[end] = true;
            }
        }
    }
    reachable[said.len()]
}

#[cfg(test)]
#[path = "answer_tests.rs"]
mod tests;
