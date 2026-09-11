use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{EntityRec, HomeGraph, ParseDecision, Settings};

fn timer_home() -> HomeGraph {
    HomeGraph {
        entities: vec![EntityRec {
            entity_id: "timer.oven".into(),
            name: "Oven".into(),
            domain: "timer".into(),
            platform: None,
            area: None,
            aliases: vec!["oven".into()],
            tags: vec![],
        }],
        ..HomeGraph::default()
    }
}

fn names(outcome: &klar_nlu::types::ParseOutcome) -> Vec<String> {
    outcome.plan.as_ref().map(|plan| plan.intents().into_iter().map(|intent| intent.name).collect()).unwrap_or_default()
}

fn slot(outcome: &klar_nlu::types::ParseOutcome, name: &str) -> Vec<String> {
    outcome
        .plan
        .as_ref()
        .map(|plan| {
            plan.intents().into_iter().flat_map(|intent| intent.slots).filter(|slot| slot.name == name).map(|slot| slot.value).collect()
        })
        .unwrap_or_default()
}

#[test]
fn every_locale_can_decrease_a_timer() {
    let home = timer_home();
    for (lang, text) in [
        ("en", "timer decrease 10 seconds"),
        ("fr", "minuteur reduire 10 seconde"),
        ("de", "timer verringern 10 sekunden"),
        ("ja", "タイマー 減らす 10 秒"),
        ("zh-CN", "定时 减少 10 秒"),
        ("ar", "مؤقت أنقص 10 ثانية"),
        ("th", "ตั้งเวลา ลด 10 วินาที"),
    ] {
        let outcome = parse(text, &home, &mut Session::new(), &[], &Settings::pinned(lang));
        let found = names(&outcome);
        assert!(
            matches!(outcome.decision, ParseDecision::Execute)
                && found.iter().any(|name| name == "HassDecreaseTimer")
                && slot(&outcome, "seconds").iter().any(|value| value == "10"),
            "{lang} {text} decision={:?} intents={found:?} seconds={:?}",
            outcome.decision,
            slot(&outcome, "seconds")
        );
    }
}

#[test]
fn every_locale_asks_how_long_without_a_duration() {
    let home = timer_home();
    for (lang, text, prompt) in [
        ("en", "Start the timer", "How long?"),
        ("fr", "allume minuteur", "Combien de temps ?"),
        ("de", "timer starten", "Wie lange?"),
        ("ja", "つけて タイマー", "どのくらい？"),
        ("zh-CN", "打开 定时", "多长时间？"),
    ] {
        let outcome = parse(text, &home, &mut Session::new(), &[], &Settings::pinned(lang));
        assert!(
            matches!(&outcome.decision, ParseDecision::Clarify { prompt: got, .. } if got == prompt) && names(&outcome).is_empty(),
            "{lang} {text} decision={:?} speech={:?} intents={:?}",
            outcome.decision,
            outcome.speech,
            names(&outcome)
        );
    }
}
