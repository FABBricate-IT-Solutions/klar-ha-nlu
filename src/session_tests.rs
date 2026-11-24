use super::*;

#[test]
fn rejects_oversized_conversation_id() {
    let mut sessions = Sessions::default();
    let huge = "x".repeat(200);
    let session = sessions.get_or_create(Some(&huge));
    assert_ne!(session.id, huge);
    assert!(session.id.len() <= 128);
}

#[test]
fn reuses_same_id() {
    let mut sessions = Sessions::default();
    sessions.get_or_create(Some("assist-1")).remember_entity("light.a");
    assert_eq!(sessions.get_or_create(Some("assist-1")).last_entities().collect::<Vec<_>>(), ["light.a"]);
}

#[test]
fn same_conversation_followup_keeps_last() {
    let mut sessions = Sessions::default();
    let mut first = sessions.take(Some("assist-1"));
    first.remember_entity("light.wohnzimmer");
    sessions.put(first);
    let follow = sessions.take(Some("assist-1"));
    assert_eq!(follow.last_entities().collect::<Vec<_>>(), ["light.wohnzimmer"]);
}

#[test]
fn new_conversation_does_not_inherit_without_satellite_hint() {
    let mut sessions = Sessions::default();
    let mut first = sessions.take(Some("assist-music"));
    first.remember_entity("media_player.kuchenbereich_2");
    first.remember_entity("light.wohnzimmer");
    sessions.put(first);
    let follow = sessions.take(Some("assist-calendar"));
    assert_eq!(follow.id, "assist-calendar");
    assert!(follow.last_entities().next().is_none());
}

#[test]
fn same_satellite_inherits_light_not_media() {
    let mut sessions = Sessions::default();
    let mut first = sessions.take_with_area(Some("assist-music"), "kueche");
    first.remember_entity("media_player.kuchenbereich_2");
    first.remember_entity("light.wohnzimmer");
    sessions.put(first);
    let follow = sessions.take_with_area(Some("assist-calendar"), "kueche");
    assert_eq!(follow.last_entities().collect::<Vec<_>>(), ["light.wohnzimmer"]);
}

#[test]
fn different_satellite_does_not_inherit() {
    let mut sessions = Sessions::default();
    let mut first = sessions.take_with_area(Some("wake-1"), "kueche");
    first.remember_entity("light.wohnzimmer");
    sessions.put(first);
    let follow = sessions.take_with_area(Some(FOLLOWUP_ID), "wohnzimmer");
    assert!(follow.last_entities().next().is_none());
}

#[test]
fn followup_fallback_needs_shared_satellite() {
    let mut sessions = Sessions::default();
    let mut first = sessions.take(Some("wake-1"));
    first.remember_entity("light.wohnzimmer");
    sessions.put(first);
    let follow = sessions.take(Some(FOLLOWUP_ID));
    assert_eq!(follow.id, FOLLOWUP_ID);
    assert!(follow.last_entities().next().is_none());
}

#[test]
fn existing_last_is_not_replaced_by_recent() {
    let mut sessions = Sessions::default();
    let mut first = sessions.take(Some("wake-1"));
    first.remember_entity("light.wohnzimmer");
    sessions.put(first);
    let mut second = sessions.take(Some("wake-2"));
    second.last.clear();
    second.remember_entity("light.kueche");
    sessions.put(second);
    let follow = sessions.take(Some("wake-2"));
    assert_eq!(follow.last_entities().collect::<Vec<_>>(), ["light.kueche"]);
}

#[test]
fn evicts_when_over_cap() {
    let mut sessions = Sessions::default();
    for i in 0..300 {
        sessions.get_or_create(Some(&format!("id-{i}")));
    }
    assert!(sessions.len() <= MAX_SESSIONS, "{}", sessions.len());
}

#[test]
fn caps_wrong_log() {
    let mut session = Session::new();
    for _ in 0..100 {
        session.mark_wrong();
    }
    assert_eq!(session.wrong_log.len(), WRONG_LOG_KEEP);
}

#[test]
fn caps_pending_interaction_storage() {
    let mut session = Session::new();
    session.set_clarify(vec!["x".repeat(200); MAX_CLARIFY_OPTIONS + 10], Intent::new("HassTurnOn"));
    let clarify = session.pending_clarify().expect("clarification");
    assert_eq!(clarify.options.len(), MAX_CLARIFY_OPTIONS);
    assert!(clarify.options.iter().all(|option| option.chars().count() <= 128));

    let evidence = crate::types::Evidence { kind: "test".into(), source: "test".into(), value: "test".into(), score: 1.0, exact: true };
    let mut plan = IntentPlan::from_intents(vec![Intent::new("HassTurnOn").with("entity_id", "lock.front")], 1.0, &[]);
    plan.evidence = vec![evidence.clone(); MAX_EVIDENCE_PER_ITEM + 10];
    plan.steps[0].evidence = vec![evidence; MAX_EVIDENCE_PER_ITEM + 10];
    assert!(session.set_confirm("candidate".into(), plan, "p".repeat(MAX_DETAIL_CHARS + 10)));
    let confirm = session.pending_confirm().expect("confirmation");
    assert_eq!(confirm.plan.evidence.len(), MAX_EVIDENCE_PER_ITEM);
    assert_eq!(confirm.plan.steps[0].evidence.len(), MAX_EVIDENCE_PER_ITEM);
    assert_eq!(confirm.prompt.chars().count(), MAX_DETAIL_CHARS);
}

#[test]
fn rejects_invalid_pending_confirm_ids_and_steps() {
    let mut session = Session::new();
    let plan = IntentPlan::from_intents(vec![Intent::new("HassTurnOn").with("entity_id", "lock.front")], 1.0, &[]);
    assert!(!session.set_confirm("x".repeat(129), plan.clone(), "confirm".into()));
    assert!(session.pending_confirm().is_none());
    let oversized = IntentPlan::from_intents(vec![Intent::new("HassTurnOn"); MAX_PLAN_STEPS + 1], 1.0, &[]);
    assert!(!session.set_confirm("candidate".into(), oversized, "confirm".into()));
    assert!(session.pending_confirm().is_none());
}

fn lock_plan() -> IntentPlan {
    IntentPlan::from_intents(vec![Intent::new("HassTurnOff").with("entity_id", "lock.front")], 1.0, &[])
}

#[test]
fn an_open_question_expires() {
    let mut session = Session::new();
    assert!(session.set_confirm("candidate".into(), lock_plan(), "Really?".into()));
    assert!(session.pending_confirm().is_some());
    session.backdate_pending(PENDING_TTL);
    assert!(session.pending().is_none(), "a question older than the limit is closed");
    assert!(session.pending_confirm().is_none());

    session.set_clarify(vec!["office".into(), "hall".into()], Intent::new("HassTurnOn"));
    session.backdate_pending(PENDING_TTL - Duration::from_secs(1));
    assert!(session.pending_clarify().is_some(), "a question inside the limit stays open");
}

#[test]
fn an_open_question_is_handed_out_once() {
    let mut sessions = Sessions::default();
    let mut asked = sessions.take(Some("door"));
    assert!(asked.set_confirm("candidate".into(), lock_plan(), "Really?".into()));
    sessions.put(asked);

    let first = sessions.take(Some("door"));
    let second = sessions.take(Some("door"));
    assert!(first.pending_confirm().is_some());
    assert!(second.pending_confirm().is_none(), "the second copy must not see the question");
}

#[test]
fn a_copy_without_the_question_does_not_erase_it() {
    let mut sessions = Sessions::default();
    let mut asked = sessions.take(Some("door"));
    assert!(asked.set_confirm("candidate".into(), lock_plan(), "Really?".into()));
    sessions.put(asked);

    let owner = sessions.take(Some("door"));
    let bystander = sessions.take(Some("door"));
    sessions.put(owner);
    sessions.put(bystander);
    assert!(sessions.take(Some("door")).pending_confirm().is_some(), "the unanswered question is still open");

    let mut answered = sessions.take(Some("door"));
    answered.clear_pending();
    sessions.put(answered);
    assert!(sessions.take(Some("door")).pending_confirm().is_none(), "the owner closed it");
}

#[test]
fn an_empty_or_malformed_id_is_never_a_shared_session() {
    let mut sessions = Sessions::default();
    for id in ["", "   ", "bell\u{7}", "line\nbreak"] {
        let first = sessions.take(Some(id));
        let mut marked = first.clone();
        marked.remember_entity("light.a");
        sessions.put(marked);
        let second = sessions.take(Some(id));
        assert_ne!(first.id, id, "{id:?} must not become a key");
        assert_ne!(first.id, second.id, "{id:?} must not be shared");
        assert!(second.last_entities().next().is_none(), "{id:?} leaked state");
    }
    assert!(valid_id("assist-1") && !valid_id("") && !valid_id(&"x".repeat(129)) && !valid_id("a\u{0}b"));
}

#[test]
fn new_ids_do_not_push_out_an_open_confirmation() {
    let mut sessions = Sessions::default();
    let mut asked = sessions.take(Some("door"));
    assert!(asked.set_confirm("candidate".into(), lock_plan(), "Really?".into()));
    sessions.put(asked);
    for i in 0..(MAX_SESSIONS + 50) {
        sessions.get_or_create(Some(&format!("stranger-{i}")));
    }
    assert!(sessions.len() <= MAX_SESSIONS);
    assert!(sessions.take(Some("door")).pending_confirm().is_some(), "the waiting session was evicted");
}
