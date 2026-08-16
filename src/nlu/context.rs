use crate::lang::Catalog;
use crate::session::Session;
use crate::types::{CustomSentence, HomeGraph, Settings};

pub struct ParseContext<'a> {
    pub text: &'a str,
    pub home: &'a HomeGraph,
    pub session: &'a Session,
    pub custom: &'a [CustomSentence],
    pub settings: &'a Settings,
    pub catalog: &'static Catalog,
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
        Self { text, home, session, custom, settings, catalog }
    }
}
