#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod action;
pub(crate) mod also;
pub(crate) mod calendar;
pub(crate) mod chat;
pub(crate) mod clause;
pub(crate) mod clause_area;
pub(crate) mod clause_early;
pub(crate) mod clause_session;
pub(crate) mod clause_support;
pub mod compound;
pub(crate) mod fuzzy;
pub(crate) mod infer;
pub(crate) mod media;
pub mod normalize;
pub mod numbers;
pub(crate) mod policy;
pub mod resolve;
pub(crate) mod resolve_named;
pub mod respond;
pub(crate) mod slots;
pub mod split;

use crate::session::Session;
use crate::types::{CustomSentence, HomeGraph, ParseResult, Settings};

pub fn parse(text: &str, home: &HomeGraph, session: &mut Session, custom: &[CustomSentence], settings: &Settings) -> ParseResult {
    let mut compatibility = settings.clone();
    compatibility.confirm_risky_actions = false;
    crate::nlu::parse_compatible(text, home, session, custom, &compatibility)
}
