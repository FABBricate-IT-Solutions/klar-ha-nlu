use crate::lang::Catalog;
use crate::session::Session;
use crate::types::{CustomSentence, HomeGraph, PolicyRule, Settings, SpeechBank};
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
        Self { text, home, session, custom, settings, catalog, policies: &[], speech_bank: empty_bank() }
    }

    pub fn with_policies(mut self, policies: &'a [PolicyRule], speech_bank: &'a SpeechBank) -> Self {
        self.policies = policies;
        self.speech_bank = speech_bank;
        self
    }
}

fn empty_bank() -> &'static SpeechBank {
    static BANK: LazyLock<SpeechBank> = LazyLock::new(SpeechBank::default);
    &BANK
}
