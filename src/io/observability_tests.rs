//! Package 3.10: load/save failures leave a log line; ops counters move.

use crate::home::overlay_store::{OverlayStore, OVERLAY_FILE};
use crate::home::snapshot::{load_home_snapshot, HOME_SNAPSHOT_FILE};
use crate::home::test_dir::TestDir;
use crate::io::conversations::{turn_from_outcome, ConversationJournal};
use crate::ops;
use crate::types::{ParseDecision, ParseOutcome};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::sync::{Arc, Mutex};
use tracing_subscriber::fmt::MakeWriter;

#[derive(Clone, Default)]
struct LogBuf(Arc<Mutex<Vec<u8>>>);

impl Write for LogBuf {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("log buf").extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for LogBuf {
    type Writer = LogBuf;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

fn logged<T>(f: impl FnOnce() -> T) -> (T, String) {
    let buf = LogBuf::default();
    let subscriber = tracing_subscriber::fmt().with_writer(buf.clone()).with_max_level(tracing::Level::WARN).finish();
    let out = tracing::subscriber::with_default(subscriber, f);
    let text = String::from_utf8_lossy(&buf.0.lock().expect("log buf")).to_string();
    (out, text)
}

fn sample_outcome() -> ParseOutcome {
    ParseOutcome {
        schema_version: "2.0".into(),
        text: "x".into(),
        conversation_id: "c1".into(),
        decision: ParseDecision::Chat,
        speech: "ok".into(),
        confidence: 0.5,
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
        quiet_ack_eligible: false,
        clock_answer: false,
        refine_band: None,
        teach: None,
    }
}

#[test]
fn unreadable_overlay_load_emits_warning_or_error() {
    let dir = TestDir::new("obs-overlay-load");
    std::fs::write(dir.file(OVERLAY_FILE), b"{\"custom\": [").unwrap();
    let (_overlay, text) = logged(|| OverlayStore::open(dir.path()).load());
    assert!(
        text.contains("unlesbar") || text.contains("nicht lesbar") || text.contains("klar_nlu.json"),
        "expected load failure log, got: {text}"
    );
}

#[test]
fn unreadable_overlay_blocks_save_with_log() {
    let dir = TestDir::new("obs-overlay-save");
    let store = OverlayStore::open(dir.path());
    store.update(|overlay| overlay.preferred.push("light.a".into())).unwrap();
    std::fs::write(dir.file(OVERLAY_FILE), b"{").unwrap();
    let (result, text) = logged(|| store.update(|overlay| overlay.preferred.clear()));
    assert!(result.is_err());
    assert!(
        text.contains("unlesbar") || text.contains("nicht überschrieben") || text.contains("klar_nlu.json"),
        "expected save/load failure log, got: {text}"
    );
}

#[test]
fn corrupt_home_snapshot_load_emits_warning() {
    let dir = TestDir::new("obs-snap-load");
    std::fs::write(dir.file(HOME_SNAPSHOT_FILE), b"{not-json").unwrap();
    let (loaded, text) = logged(|| load_home_snapshot(dir.path()));
    assert!(loaded.is_none());
    assert!(text.contains(HOME_SNAPSHOT_FILE) && text.contains("ungültig"), "expected snapshot warn, got: {text}");
}

#[test]
fn journal_save_failure_emits_warning() {
    let dir = TestDir::new("obs-journal-save");
    let journal = ConversationJournal::open(dir.path());
    let mut perms = std::fs::metadata(dir.path()).unwrap().permissions();
    perms.set_mode(0o555);
    std::fs::set_permissions(dir.path(), perms).unwrap();
    let turn = turn_from_outcome(&sample_outcome(), Vec::new(), None);
    let (_, text) = logged(|| journal.append(turn));
    let mut perms = std::fs::metadata(dir.path()).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(dir.path(), perms).unwrap();
    assert!(text.contains("conversations.jsonl") && text.contains("nicht"), "expected journal save warn, got: {text}");
}

#[test]
fn catalog_cache_counters_move_on_reuse() {
    let before = ops::ops().snapshot();
    let first = crate::lang::catalog_for(&["en".into()]);
    let second = crate::lang::catalog_for(&["en".into()]);
    assert_eq!(first.langs, second.langs);
    let after = ops::ops().snapshot();
    assert!(
        after.cache_hits + after.cache_misses > before.cache_hits + before.cache_misses,
        "cache counters should move: before={before:?} after={after:?}"
    );
}

#[test]
fn overlay_write_counter_moves_on_successful_save() {
    let dir = TestDir::new("obs-overlay-count");
    let before = ops::ops().snapshot().overlay_writes;
    OverlayStore::open(dir.path()).update(|overlay| overlay.preferred.push("light.b".into())).unwrap();
    let after = ops::ops().snapshot().overlay_writes;
    assert!(after > before, "overlay_writes {before} -> {after}");
}
