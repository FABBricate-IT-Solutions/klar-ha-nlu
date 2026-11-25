//! Offload `/api/v2/parse` (and Wyoming) onto blocking workers with a slot and deadline (3.4).

use crate::nlu::parse_with_controls;
use crate::session::Session;
use crate::types::{CustomSentence, HomeGraph, MatchControl, ParseOutcome, PolicyRule, Settings, SpeechBank};
use axum::http::StatusCode;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;

/// Hard wall clock for one parse on a blocking worker.
pub const PARSE_TIMEOUT: Duration = Duration::from_millis(1_500);
/// Concurrent CPU-heavy parses. Extra callers get 503 quickly.
pub const PARSE_SLOTS: usize = 4;
const SLOT_WAIT: Duration = Duration::from_millis(50);

#[derive(Clone)]
pub struct ParseLimiter {
    slots: Arc<Semaphore>,
}

impl Default for ParseLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl ParseLimiter {
    pub fn new() -> Self {
        Self { slots: Arc::new(Semaphore::new(PARSE_SLOTS)) }
    }
}

pub struct ParseRequest {
    pub text: String,
    pub home: Arc<HomeGraph>,
    pub session: Session,
    pub custom: Vec<CustomSentence>,
    pub settings: Settings,
    pub policies: Vec<PolicyRule>,
    pub speech_bank: SpeechBank,
    pub match_controls: Vec<MatchControl>,
}

pub struct ParseResponse {
    pub outcome: ParseOutcome,
    pub session: Session,
}

/// Run parse off the async runtime. Returns 503 when slots are full, 408 on deadline.
pub async fn run_parse(limiter: &ParseLimiter, mut request: ParseRequest) -> Result<ParseResponse, StatusCode> {
    let permit = acquire_slot(limiter).await?;
    let handle = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        let outcome = parse_with_controls(
            &request.text,
            &request.home,
            &mut request.session,
            &request.custom,
            &request.settings,
            &request.policies,
            &request.speech_bank,
            &request.match_controls,
        );
        ParseResponse { outcome, session: request.session }
    });
    match timeout(PARSE_TIMEOUT, handle).await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(_)) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        Err(_) => Err(StatusCode::REQUEST_TIMEOUT),
    }
}

async fn acquire_slot(limiter: &ParseLimiter) -> Result<OwnedSemaphorePermit, StatusCode> {
    match timeout(SLOT_WAIT, limiter.slots.clone().acquire_owned()).await {
        Ok(Ok(permit)) => Ok(permit),
        Ok(Err(_)) => Err(StatusCode::SERVICE_UNAVAILABLE),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::default_home;
    use crate::types::Settings;

    #[tokio::test]
    async fn parse_limiter_returns_an_outcome() {
        let limiter = ParseLimiter::new();
        let response = run_parse(
            &limiter,
            ParseRequest {
                text: "turn on the light".into(),
                home: Arc::new(default_home()),
                session: Session::default(),
                custom: Vec::new(),
                settings: Settings::pinned("en"),
                policies: Vec::new(),
                speech_bank: SpeechBank::default(),
                match_controls: Vec::new(),
            },
        )
        .await
        .expect("parse");
        assert!(!matches!(response.outcome.decision, crate::types::ParseDecision::Error { .. }));
    }
}
