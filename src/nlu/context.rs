use crate::lang::Catalog;
use crate::session::Session;
use crate::types::{CustomSentence, HomeGraph, MatchControl, PolicyRule, Settings, SpeechBank};
use std::sync::LazyLock;

pub struct ParseContext<'a> {
    pub text: &'a str,
    pub home: &'a HomeGraph,
    pub session: &'a Session,
    pub custom: &'a [CustomSentence],
    pub settings: &'a Settings,
    pub catalog: &'static Catalog,
    pub policies: &'a [PolicyRule],
    pub speech_bank: &'a SpeechBank,
    pub match_controls: &'a [MatchControl],
}

impl<'a> ParseContext<'a> {
    pub fn new(
        text: &'a str,
        home: &'a HomeGraph,
        session: &'a Session,
        custom: &'a [CustomSentence],
        settings: &'a Settings,
        catalog: &'static Catalog,
    ) -> Self {
        Self { text, home, session, custom, settings, catalog, policies: &[], speech_bank: empty_bank(), match_controls: empty_controls() }
    }

    pub fn with_policies(mut self, policies: &'a [PolicyRule], speech_bank: &'a SpeechBank) -> Self {
        self.policies = policies;
        self.speech_bank = speech_bank;
        self
    }

    pub fn with_match_controls(mut self, match_controls: &'a [MatchControl]) -> Self {
        self.match_controls = match_controls;
        self
    }
}

fn empty_controls() -> &'static [MatchControl] {
    static CONTROLS: LazyLock<Vec<MatchControl>> = LazyLock::new(Vec::new);
    CONTROLS.as_slice()
}

fn empty_bank() -> &'static SpeechBank {
    static BANK: LazyLock<SpeechBank> = LazyLock::new(SpeechBank::default);
    &BANK
}
