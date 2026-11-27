use super::*;
use crate::home::test_dir::TestDir;
use crate::types::{ParseDecision, ParseOutcome, RejectReason};

#[test]
fn confirm_turn_hides_plan() {
    let outcome = ParseOutcome {
        schema_version: "2.0".into(),
        text: "lock the door".into(),
        conversation_id: "c1".into(),
        decision: ParseDecision::Confirm { prompt: "Really?".into(), candidate_id: "sel".into() },
        speech: "Really?".into(),
        confidence: 0.7,
        margin: 1.0,
        selected_candidate_id: None,
        candidates: Vec::new(),
        plan: None,
        evidence: Vec::new(),
        trace: Default::default(),
        briefing: false,
        news_topic: None,
        retrieval: None,
        policy_trace: None,
        clock_answer: false,
        quiet_ack_eligible: false,
        refine_band: Some(RefineBand::Status),
        teach: None,
    };
    let turn = turn_from_outcome(&outcome, Vec::new(), None);
    assert_eq!(turn.decision, "confirm");
    assert_eq!(turn.text.as_deref(), Some("lock the door"));
    assert_eq!(turn.tokens, replay_tokens("lock the door"));
    assert_eq!(turn.speech, "Really?");
    assert_eq!(turn.confirm_prompt.as_deref(), Some("Really?"));
    assert_eq!(turn.candidate_id.as_deref(), Some("sel"));
    assert_eq!(turn.refine_band.as_deref(), Some("status"));
    let json = serde_json::to_string(&turn).expect("turn json");
    assert!(json.contains("\"last_names\":[]"), "{json}");
    assert!(json.contains("\"evidence_kinds\":[]"), "{json}");
    assert!(!json.contains("plan"), "{json}");
}

#[test]
fn reject_turn_has_no_candidate() {
    let outcome = ParseOutcome {
        schema_version: "2.0".into(),
        text: "x".into(),
        conversation_id: "c1".into(),
        decision: ParseDecision::Reject { reason: RejectReason::NoAction },
        speech: String::new(),
        confidence: 0.0,
        margin: 0.0,
        selected_candidate_id: None,
        candidates: Vec::new(),
        plan: None,
        evidence: Vec::new(),
        trace: Default::default(),
        briefing: false,
        news_topic: None,
        retrieval: None,
        policy_trace: None,
        clock_answer: false,
        quiet_ack_eligible: false,
        refine_band: None,
        teach: None,
    };
    let turn = turn_from_outcome(&outcome, vec!["Kugel".into()], Some("kueche".into()));
    assert_eq!(turn.preferred_area.as_deref(), Some("kueche"));
    assert_eq!(turn.text.as_deref(), Some("x"));
    assert_eq!(turn.tokens, replay_tokens("x"));
    assert!(turn.candidate_id.is_none());
    assert_eq!(turn.reject_reason.as_deref(), Some("no_action"));
}

#[test]
fn default_journal_keeps_tokens_and_speech_for_the_operator() {
    let spoken = "Mach das Küchenlicht an!";
    let outcome = ParseOutcome {
        schema_version: "2.0".into(),
        text: spoken.into(),
        conversation_id: "c1".into(),
        decision: ParseDecision::Chat,
        speech: "Verstanden.".into(),
        confidence: 0.2,
        margin: 0.0,
        selected_candidate_id: None,
        candidates: Vec::new(),
        plan: None,
        evidence: Vec::new(),
        trace: Default::default(),
        briefing: false,
        news_topic: None,
        retrieval: None,
        policy_trace: None,
        clock_answer: false,
        quiet_ack_eligible: false,
        refine_band: None,
        teach: None,
    };
    let turn = turn_from_outcome(&outcome, vec!["HassTurnOn".into()], None);
    assert_eq!(turn.text.as_deref(), Some(spoken));
    assert_eq!(turn.tokens, replay_tokens(spoken));
    assert_eq!(turn.speech, "Verstanden.");
    let json = serde_json::to_string(&turn).expect("turn json");
    assert!(json.contains(spoken), "{json}");
    assert!(json.contains("Küchenlicht"), "{json}");
}

#[test]
fn journal_roundtrip_keeps_utterance_for_the_operator() {
    let scratch = TestDir::new("journal-tokens");
    let dir = scratch.path().to_path_buf();
    let spoken = "Küche AN!";
    let outcome = ParseOutcome {
        schema_version: "2.0".into(),
        text: spoken.into(),
        conversation_id: "c9".into(),
        decision: ParseDecision::Execute,
        speech: "An.".into(),
        confidence: 0.9,
        margin: 1.0,
        selected_candidate_id: None,
        candidates: Vec::new(),
        plan: None,
        evidence: Vec::new(),
        trace: Default::default(),
        briefing: false,
        news_topic: None,
        retrieval: None,
        policy_trace: None,
        clock_answer: false,
        quiet_ack_eligible: false,
        refine_band: None,
        teach: None,
    };
    let journal = ConversationJournal::open(&dir);
    journal.append(turn_from_outcome(&outcome, Vec::new(), None));
    let reloaded = ConversationJournal::open(&dir);
    let turns = reloaded.list();
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].text.as_deref(), Some(spoken));
    assert_eq!(turns[0].tokens, replay_tokens(spoken));
    assert_eq!(turns[0].speech, "An.");
    let raw = crate::home::paths::read_to_string_confined(&dir, JOURNAL_FILE).expect("journal file");
    assert!(raw.contains(spoken), "{raw}");
}

#[test]
fn legacy_journal_line_defaults_empty_tokens() {
    let line = r#"{"conversation_id":"c1","ts_ms":1,"decision":"chat","speech":"hi","confidence":0.1,"briefing":false}"#;
    let turn: ConversationTurn = serde_json::from_str(line).expect("legacy turn");
    assert!(turn.tokens.is_empty());
    assert!(turn.text.is_none());
    assert!(turn.speech_source.is_none());
}

#[test]
fn note_spoken_patches_chat_reply_on_latest_turn() {
    let scratch = TestDir::new("journal-chat");
    let dir = scratch.path().to_path_buf();
    let outcome = ParseOutcome {
        schema_version: "2.0".into(),
        text: "erzähl einen witz".into(),
        conversation_id: "c-chat".into(),
        decision: ParseDecision::Chat,
        speech: "Verstanden.".into(),
        confidence: 0.2,
        margin: 0.0,
        selected_candidate_id: None,
        candidates: Vec::new(),
        plan: None,
        evidence: Vec::new(),
        trace: Default::default(),
        briefing: false,
        news_topic: None,
        retrieval: None,
        policy_trace: None,
        clock_answer: false,
        quiet_ack_eligible: false,
        refine_band: None,
        teach: None,
    };
    let journal = ConversationJournal::open(&dir);
    journal.append(turn_from_outcome(&outcome, Vec::new(), None));
    let spoken_at = journal.list()[0].ts_ms;
    journal.note_spoken(Some("c-chat"), "Zwei Roboter gehen in eine Bar.", "chat");
    journal.note_spoken(Some("c-chat"), "KLAR_PARSE: licht an", "chat");
    let turns = journal.list();
    assert_eq!(turns.len(), 1, "patching the reply must not add a second card");
    assert_eq!(turns[0].speech, "Zwei Roboter gehen in eine Bar.");
    assert_eq!(turns[0].speech_source.as_deref(), Some("chat"));
    assert_eq!(turns[0].decision, "chat");
    assert_eq!(turns[0].ts_ms, spoken_at, "the card keeps the request time, not the reply time");
}

#[test]
fn journal_append_keeps_prior_lines_readable() {
    let scratch = TestDir::new("journal-append");
    let dir = scratch.path().to_path_buf();
    let journal = ConversationJournal::open(&dir);
    let mut first = turn_from_outcome(
        &ParseOutcome {
            schema_version: "2.0".into(),
            text: "a".into(),
            conversation_id: "c1".into(),
            decision: ParseDecision::Chat,
            speech: String::new(),
            confidence: 0.0,
            margin: 0.0,
            selected_candidate_id: None,
            candidates: Vec::new(),
            plan: None,
            evidence: Vec::new(),
            trace: Default::default(),
            briefing: false,
            news_topic: None,
            retrieval: None,
            policy_trace: None,
            clock_answer: false,
            quiet_ack_eligible: false,
            refine_band: None,
            teach: None,
        },
        Vec::new(),
        None,
    );
    first.ts_ms = 1;
    let mut second = first.clone();
    journal.append(first);
    second.conversation_id = "c2".into();
    second.ts_ms = 2;
    journal.append(second);
    let raw = crate::home::paths::read_to_string_confined(&dir, JOURNAL_FILE).expect("journal");
    let lines: Vec<_> = raw.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(lines.len(), 2, "{raw}");
    assert!(lines[0].contains("\"c1\""), "{raw}");
    assert!(lines[1].contains("\"c2\""), "{raw}");
}
