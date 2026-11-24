"""Writes tests/news_langs.rs: one news briefing per compiled pack, in its own words."""

from __future__ import annotations

from pathlib import Path

from lang_packs.emit_rust import RUST_BANNER, rust_str
from lang_packs.news_flow import FLOW, HAND

_HEAD = """use klar_nlu::home::default_home;
use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{ParseDecision, ParseOutcome, Settings};

/// Per pack: ask for the news, ask for more on one story, ask what happened, ask for "all the news
/// about that", then a section and a named topic with the search words they must yield. Last come
/// a light on and a light off command, which must still reach the home while the briefing is open.
#[rustfmt::skip]
const CASES: &[(&str, [&str; 6], [&str; 2], [&str; 2])] = &[
"""

_TAIL = """];

fn roundup(turn: &ParseOutcome) -> bool {
    matches!(turn.decision, ParseDecision::Chat) && turn.briefing && !turn.speech.is_empty()
}

fn follow_up(turn: &ParseOutcome) -> bool {
    matches!(turn.decision, ParseDecision::Chat) && turn.briefing && turn.speech.is_empty()
}

fn told(turn: &ParseOutcome) -> String {
    format!("decision={:?} briefing={} topic={:?} speech={:?}", turn.decision, turn.briefing, turn.news_topic, turn.speech)
}

#[test]
fn every_pack_keeps_a_news_briefing_on_track() {
    let home = default_home();
    let mut failures = Vec::new();
    for (pack, say, want, lights) in CASES {
        let settings = Settings::pinned(*pack);
        let mut session = Session::new();
        let first = parse(say[0], &home, &mut session, &[], &settings);
        if !roundup(&first) || first.news_topic.is_some() {
            failures.push(format!("{pack} ask {:?}: {}", say[0], told(&first)));
        }
        for text in &say[1..4] {
            let turn = parse(text, &home, &mut session, &[], &settings);
            if !follow_up(&turn) || turn.news_topic.is_some() {
                failures.push(format!("{pack} follow-up {text:?}: {}", told(&turn)));
            }
        }
        for (text, topic) in [(say[4], want[0]), (say[5], want[1])] {
            let fresh = parse(text, &home, &mut Session::new(), &[], &settings);
            if !roundup(&fresh) || fresh.news_topic.as_deref() != Some(topic) {
                failures.push(format!("{pack} topic {text:?} wants {topic:?}: {}", told(&fresh)));
            }
        }
        let switched = parse(say[5], &home, &mut session, &[], &settings);
        if !roundup(&switched) || switched.news_topic.as_deref() != Some(want[1]) {
            failures.push(format!("{pack} new topic inside the briefing {:?}: {}", say[5], told(&switched)));
        }
        for text in lights {
            let turn = parse(text, &home, &mut session, &[], &settings);
            if !matches!(turn.decision, ParseDecision::Execute) {
                failures.push(format!("{pack} home command inside the briefing {text:?}: {}", told(&turn)));
            }
            session.briefing = true;
        }
    }
    assert!(failures.is_empty(), "{} news turns went wrong:\\n{}", failures.len(), failures.join("\\n"));
}
"""


def write_news_test(lights: dict[str, tuple[str, str]]) -> None:
    """`lights` maps every pack code to a light-on and a light-off sentence."""
    rows = []
    for code, pair in lights.items():
        flow = HAND.get(code) or FLOW[code]
        say = ", ".join(rust_str(text) for text in flow["say"])
        want = ", ".join(rust_str(text) for text in flow["want"])
        home = ", ".join(rust_str(text) for text in pair)
        rows.append(f"    ({rust_str(code)}, [{say}], [{want}], [{home}]),\n")
    dest = Path(__file__).resolve().parents[2] / "tests" / "news_langs.rs"
    dest.write_text(RUST_BANNER + _HEAD + "".join(rows) + _TAIL, encoding="utf-8")
