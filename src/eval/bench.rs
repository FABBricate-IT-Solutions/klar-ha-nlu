use crate::nlu;
use crate::session::Session;
use crate::types::{HomeGraph, ParseOutcome, Settings};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchReport {
    pub samples: u32,
    pub p50_us: u64,
    pub p95_us: u64,
    pub p99_us: u64,
    pub rss_kb: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StageSample {
    pub stage: String,
    pub duration_us: u64,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassBench {
    pub class: String,
    pub text: String,
    pub setup: Option<String>,
    pub decision: String,
    pub cold_us: u64,
    pub warm: BenchReport,
    pub stages: Vec<StageSample>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldoutBench {
    pub utterances: usize,
    pub turns: usize,
    pub wall_us: u64,
}

impl HeldoutBench {
    pub fn per_utterance_us(&self) -> u64 {
        if self.utterances == 0 {
            0
        } else {
            self.wall_us / self.utterances as u64
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LanguageBench {
    pub language: String,
    pub classes: Vec<ClassBench>,
    pub heldout: HeldoutBench,
    pub rss_kb: u64,
    pub verdict: &'static str,
}

struct Case {
    class: &'static str,
    text: &'static str,
    setup: Option<&'static str>,
}

const DE: &[Case] = &[
    Case { class: "short_light", text: "Mach das Eingangslicht an", setup: None },
    Case { class: "two_clause", text: "Mach Garagenlicht an und mach Eingangslicht an", setup: None },
    Case { class: "status", text: "Status Lichter Eingang", setup: None },
    Case { class: "timer", text: "Starte den Timer Ofen-Timer für 2 Minuten", setup: None },
    Case { class: "clarify", text: "mach die lampe im schlafzimmer an", setup: None },
    Case { class: "ood", text: "Was ist die Hauptstadt von Frankreich", setup: None },
    Case { class: "follow_up", text: "ja", setup: Some("mach die lampe im schlafzimmer an") },
];

const EN: &[Case] = &[
    Case { class: "short_light", text: "Turn on the Entryway Light", setup: None },
    Case { class: "two_clause", text: "Turn on the Garage Light and turn on the Entryway Light", setup: None },
    Case { class: "status", text: "What's the status of the entryway light?", setup: None },
    Case { class: "timer", text: "Start the oven timer for 2 minutes 30 seconds", setup: None },
    Case { class: "clarify", text: "lock the door", setup: None },
    Case { class: "ood", text: "What is the capital of France", setup: None },
    Case { class: "follow_up", text: "yes", setup: Some("lock the door") },
];

pub fn bench_warm(home: &HomeGraph, text: &str, language: &str, repeat: u32) -> BenchReport {
    let settings = settings_for(language);
    let _ = parse_once(home, &settings, text, None);
    warm_report(home, &settings, text, None, repeat.max(1)).0
}

pub fn bench_language(home: &HomeGraph, language: &str, repeat: u32) -> Result<LanguageBench, String> {
    let repeat = repeat.max(8);
    let settings = settings_for(language);
    let cases = if language.starts_with("de") { DE } else { EN };
    let classes = cases.iter().map(|case| time_class(home, &settings, case, repeat)).collect::<Vec<_>>();
    let heldout = bench_heldout(home, language)?;
    let rss_kb = current_rss_kb();
    let verdict = verdict_for(&classes);
    Ok(LanguageBench { language: language.to_string(), classes, heldout, rss_kb, verdict })
}

pub fn format_language_bench(repeat: u32, bench: &LanguageBench) -> String {
    let mut out = format!("language={} repeat={} rss_kb={}\n", bench.language, repeat.max(8), bench.rss_kb);
    for class in &bench.classes {
        let setup = class.setup.as_deref().map(|text| format!(" setup={text}")).unwrap_or_default();
        out.push_str(&format!(
            "class={} cold_us={} p50_us={} p95_us={} p99_us={} n={} decision={} text={}{setup}\n",
            class.class,
            class.cold_us,
            class.warm.p50_us,
            class.warm.p95_us,
            class.warm.p99_us,
            class.warm.samples,
            class.decision,
            class.text
        ));
        for stage in &class.stages {
            out.push_str(&format!("  stage={} us={} detail={}\n", stage.stage, stage.duration_us, stage.detail));
        }
    }
    let held = &bench.heldout;
    out.push_str(&format!(
        "heldout utterances={} turns={} wall_us={} per_utterance_us={}\n",
        held.utterances,
        held.turns,
        held.wall_us,
        held.per_utterance_us()
    ));
    out.push_str(&format!("verdict={}\n", bench.verdict));
    out
}

fn bench_heldout(home: &HomeGraph, language: &str) -> Result<HeldoutBench, String> {
    let items = super::load_corpus(&super::heldout_dir(language), language)?;
    let settings = settings_for(language);
    let turns = items.iter().map(|item| item.turns.len()).sum();
    let started = Instant::now();
    for item in &items {
        let mut session = Session::new();
        for turn in &item.turns {
            let _ = nlu::parse(turn, home, &mut session, &[], &settings);
        }
    }
    Ok(HeldoutBench { utterances: items.len(), turns, wall_us: started.elapsed().as_micros() as u64 })
}

fn time_class(home: &HomeGraph, settings: &Settings, case: &Case, repeat: u32) -> ClassBench {
    let (cold_us, _) = parse_once(home, settings, case.text, case.setup);
    let (warm, outcome) = warm_report(home, settings, case.text, case.setup, repeat);
    let stages = outcome
        .trace
        .stages
        .iter()
        .map(|stage| StageSample { stage: stage.stage.clone(), duration_us: stage.duration_us, detail: stage.detail.clone() })
        .collect();
    ClassBench {
        class: case.class.to_string(),
        text: case.text.to_string(),
        setup: case.setup.map(str::to_string),
        decision: outcome.decision.type_name().to_string(),
        cold_us,
        warm,
        stages,
    }
}

fn warm_report(home: &HomeGraph, settings: &Settings, text: &str, setup: Option<&str>, repeat: u32) -> (BenchReport, ParseOutcome) {
    let mut samples = Vec::with_capacity(repeat as usize);
    let mut last = None;
    for _ in 0..repeat {
        let (elapsed, outcome) = parse_once(home, settings, text, setup);
        samples.push(elapsed);
        last = Some(outcome);
    }
    samples.sort_unstable();
    let report = BenchReport {
        samples: repeat,
        p50_us: percentile(&samples, 50),
        p95_us: percentile(&samples, 95),
        p99_us: percentile(&samples, 99),
        rss_kb: current_rss_kb(),
    };
    (report, last.unwrap_or_else(|| parse_once(home, settings, text, setup).1))
}

fn parse_once(home: &HomeGraph, settings: &Settings, text: &str, setup: Option<&str>) -> (u64, ParseOutcome) {
    let mut session = Session::new();
    if let Some(setup) = setup {
        let _ = nlu::parse(setup, home, &mut session, &[], settings);
    }
    let started = Instant::now();
    let outcome = nlu::parse(text, home, &mut session, &[], settings);
    (started.elapsed().as_micros() as u64, outcome)
}

fn settings_for(language: &str) -> Settings {
    Settings { languages: vec![language.to_string()], ..Settings::default() }
}

fn verdict_for(classes: &[ClassBench]) -> &'static str {
    let improve = classes.iter().any(|class| class.warm.p95_us >= 50_000 || class.warm.p99_us >= 100_000);
    if improve {
        return "improve";
    }
    let fast = classes.iter().all(|class| class.warm.p95_us < 20_000 && class.cold_us < 100_000);
    if fast {
        "fast"
    } else {
        "acceptable"
    }
}

fn percentile(sorted: &[u64], pct: u32) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let index = ((pct as usize).saturating_mul(sorted.len().saturating_sub(1))) / 100;
    sorted[index]
}

fn current_rss_kb() -> u64 {
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return 0;
    };
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmRSS:"))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ParseDecision;

    #[test]
    fn german_short_light_executes_on_german_pack() {
        let home = super::super::family_home("de").expect("home");
        let settings = settings_for("de");
        let (_, outcome) = parse_once(&home, &settings, "Mach das Eingangslicht an", None);
        assert!(matches!(outcome.decision, ParseDecision::Execute), "{:?}", outcome.decision);
    }

    #[test]
    fn follow_up_confirms_pending_clarify() {
        let home = super::super::family_home("de").expect("home");
        let settings = settings_for("de");
        let mut session = Session::new();
        let first = nlu::parse("mach die lampe im schlafzimmer an", &home, &mut session, &[], &settings);
        assert!(matches!(first.decision, ParseDecision::Clarify { .. } | ParseDecision::Confirm { .. }), "{:?}", first.decision);
        let second = nlu::parse("ja", &home, &mut session, &[], &settings);
        assert!(matches!(second.decision, ParseDecision::Execute), "{:?}", second.decision);

        let home = super::super::family_home("en").expect("home");
        let settings = settings_for("en");
        let mut session = Session::new();
        let first = nlu::parse("lock the door", &home, &mut session, &[], &settings);
        assert!(matches!(first.decision, ParseDecision::Clarify { .. } | ParseDecision::Confirm { .. }), "{:?}", first.decision);
        let second = nlu::parse("yes", &home, &mut session, &[], &settings);
        assert!(matches!(second.decision, ParseDecision::Execute), "{:?}", second.decision);
    }
}
