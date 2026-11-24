//! Word classes that tell a command from a sentence that must not change anything:
//! a negated one, a question, a report of what happened, a condition or a schedule.
//! `parse::mood` reads a clause with them.
//!
//! Every entry is folded the way speech is tokenized. An entry may be one word or several
//! words separated by one space; it then matches those tokens in that order. The tokenizer
//! drops apostrophes and a trailing "'s", so "don't" is `dont` and "it's" is `it`.

pub struct MoodWords {
    /// "not", "never", "nicht", "kein": the sentence forbids the action.
    pub negation: &'static [&'static str],
    /// A word right before a negation that makes it an exception: "but not", "aber nicht".
    pub exception_leads: &'static [&'static str],
    /// A place word right after a negation in a command about everything: "all lights,
    /// not in the bedroom". That negation takes a room out of the command.
    pub exception_places: &'static [&'static str],
    /// The first word of a question: auxiliaries, modals and question words.
    pub question_starts: &'static [&'static str],
    /// "tell me if", "sag mir ob": the sentence asks, whatever follows.
    pub ask_frames: &'static [&'static str],
    /// A sentence opening that keeps a command a command although it reads like a
    /// question or a statement: "can you", "i want", "kannst du", "ich moechte".
    pub request_frames: &'static [&'static str],
    /// Politeness that carries no meaning: "please", "if you dont mind", "wenn moeglich".
    pub courtesy: &'static [&'static str],
    /// A subject that opens a report: "i", "someone", "ich", "jemand".
    pub subjects: &'static [&'static str],
    /// Verbs that describe a state anywhere in the sentence: "is", "was", "ist", "wurde".
    pub state_verbs: &'static [&'static str],
    /// Participles and past forms of command verbs: "turned", "eingeschaltet".
    pub participles: &'static [&'static str],
    /// Words that only pad a question or a report about a state: "still", "anywhere",
    /// "noch", "irgendwo". They name nothing, so a state question may carry them.
    pub state_padding: &'static [&'static str],
    /// "if", "unless", "wenn", "sobald".
    pub conditions: &'static [&'static str],
    /// Words that put the action somewhere else in time: "tomorrow", "minutes", "spaeter".
    pub time_words: &'static [&'static str],
    /// A word that turns the number after it into a clock time: "at", "um".
    pub clock_leads: &'static [&'static str],
    /// A word that turns the number before it into a clock time: "pm".
    pub clock_marks: &'static [&'static str],
    /// A unit after a number that makes it a value, not a time: "degrees", "prozent".
    pub measure_units: &'static [&'static str],
    /// A question particle that opens the sentence: "czy", "est ce".
    pub question_openers: &'static [&'static str],
    /// A question particle that may stand anywhere: "mi", "ли".
    pub question_particles: &'static [&'static str],
    /// The sentence ends in this, with or without a space before it: "か", "吗", "khong".
    pub question_endings: &'static [&'static str],
}

impl MoodWords {
    pub const EMPTY: Self = Self {
        negation: &[],
        exception_leads: &[],
        exception_places: &[],
        question_starts: &[],
        ask_frames: &[],
        request_frames: &[],
        courtesy: &[],
        subjects: &[],
        state_verbs: &[],
        participles: &[],
        state_padding: &[],
        conditions: &[],
        time_words: &[],
        clock_leads: &[],
        clock_marks: &[],
        measure_units: &[],
        question_openers: &[],
        question_particles: &[],
        question_endings: &[],
    };
}
