use super::*;
use crate::home::load::LoadedHome;
use crate::home::overlay_store::{BEFORE_RESTORE_FILE, OVERLAY_FILE};
use crate::home::sample::default_home;
use crate::home::test_dir::TestDir;
use crate::io::llm::save_stored_endpoint;
use crate::llm::LlmProviderKind;
use crate::types::{PolicyEffect, PolicyRule, Settings};
use std::os::unix::fs::PermissionsExt;

const LOCAL_URL: &str = "http://127.0.0.1:8000/v1";
const MADE_UP_KEY: &str = "sk-made-up-for-tests";

fn state_in(dir: &Path) -> AppState {
    let loaded = LoadedHome {
        graph: default_home(),
        settings: Settings::default(),
        custom: Vec::new(),
        language: Default::default(),
        policies: Vec::new(),
        speech_bank: Default::default(),
        match_controls: Vec::new(),
    };
    AppState::new(loaded, dir.to_path_buf(), None)
}

fn endpoint(base_url: &str, api_key: &str, model: &str) -> StoredEndpoint {
    StoredEndpoint {
        base_url: base_url.into(),
        api_key: api_key.into(),
        model: model.into(),
        enable_thinking: false,
        provider: LlmProviderKind::Custom,
    }
}

fn overlay_with_alias(alias: &str) -> Overlay {
    let mut overlay = Overlay { settings: Some(Settings { refine_speech: true, ..Settings::default() }), ..Overlay::default() };
    overlay.aliases.insert("light.wohnzimmer".into(), vec![alias.into()]);
    overlay
}

async fn restore(state: &AppState, overlay: Overlay, incoming: Option<StoredEndpoint>) -> io::Result<()> {
    let outcome = install(state, check(Packed { overlay, endpoint: incoming }).expect("valid backup")).await;
    crate::lang::reset_runtime_packs();
    outcome
}

#[tokio::test]
async fn a_backup_without_key_keeps_the_local_key_for_the_same_url() {
    let dir = TestDir::new("restore-same-url");
    save_stored_endpoint(dir.path(), &endpoint(LOCAL_URL, MADE_UP_KEY, "alt")).unwrap();
    let state = state_in(dir.path());
    // Written differently, the same address.
    restore(&state, overlay_with_alias("kugel"), Some(endpoint("http://127.0.0.1:8000/v1/", "", "neu"))).await.unwrap();
    let stored = load_stored_endpoint(dir.path()).unwrap();
    assert_eq!(stored.api_key, MADE_UP_KEY);
    assert_eq!(stored.model, "neu");
    assert!(state.settings.lock().await.refine_speech);
    assert_eq!(state.llm.lock().await.as_ref().map(|live| live.model.clone()).as_deref(), Some("neu"));
}

#[tokio::test]
async fn a_backup_without_key_for_another_url_gets_no_key() {
    let dir = TestDir::new("restore-other-url");
    save_stored_endpoint(dir.path(), &endpoint(LOCAL_URL, MADE_UP_KEY, "alt")).unwrap();
    let state = state_in(dir.path());
    restore(&state, overlay_with_alias("kugel"), Some(endpoint("http://192.168.1.2:8000/v1", "", "neu"))).await.unwrap();
    let stored = load_stored_endpoint(dir.path()).unwrap();
    assert_eq!(stored.base_url, "http://192.168.1.2:8000/v1");
    assert_eq!(stored.api_key, "", "the key entered for one address must not follow to another");
    assert_eq!(state.llm.lock().await.as_ref().map(|live| live.api_key.clone()).as_deref(), Some(""));

    let kept = std::fs::read_to_string(dir.file(LLM_BEFORE_RESTORE_FILE)).unwrap();
    assert!(kept.contains(MADE_UP_KEY), "the endpoint as it was is kept");
    let mode = std::fs::metadata(dir.file(LLM_BEFORE_RESTORE_FILE)).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "a copy that holds a key is owner-only");
}

#[tokio::test]
async fn a_key_in_the_backup_goes_with_its_own_url() {
    let dir = TestDir::new("restore-own-key");
    save_stored_endpoint(dir.path(), &endpoint(LOCAL_URL, MADE_UP_KEY, "alt")).unwrap();
    let state = state_in(dir.path());
    restore(&state, overlay_with_alias("kugel"), Some(endpoint("http://192.168.1.2:8000/v1", "sk-from-the-archive", "neu"))).await.unwrap();
    assert_eq!(load_stored_endpoint(dir.path()).unwrap().api_key, "sk-from-the-archive");
}

#[tokio::test]
async fn when_the_endpoint_cannot_be_replaced_the_overlay_is_put_back() {
    let dir = TestDir::new("restore-rollback");
    let state = state_in(dir.path());
    restore(&state, overlay_with_alias("kugel"), None).await.unwrap();
    let before = std::fs::read(dir.file(OVERLAY_FILE)).unwrap();
    let checked = check(Packed { overlay: overlay_with_alias("ampel"), endpoint: Some(endpoint(LOCAL_URL, "", "neu")) }).unwrap();
    let outcome = install_with(&state, checked, |staged| {
        drop(staged);
        Err(io::Error::other("the rename did not happen"))
    })
    .await;
    assert!(outcome.is_err());
    assert_eq!(std::fs::read(dir.file(OVERLAY_FILE)).unwrap(), before, "both files are replaced or neither");
    assert!(!dir.file(LLM_FILE).exists(), "{:?}", dir.names());
    assert!(!dir.names().iter().any(|name| name.ends_with(".tmp")), "{:?}", dir.names());
    let home = state.home.snapshot().await;
    let lamp = home.entities.iter().find(|entity| entity.entity_id == "light.wohnzimmer").unwrap();
    assert!(lamp.aliases.iter().any(|alias| alias == "kugel") && !lamp.aliases.iter().any(|alias| alias == "ampel"));
}

#[tokio::test]
async fn the_file_as_it_was_is_kept_and_a_first_restore_leaves_no_stale_copy() {
    let dir = TestDir::new("restore-before");
    let state = state_in(dir.path());
    restore(&state, overlay_with_alias("kugel"), None).await.unwrap();
    assert!(!dir.file(BEFORE_RESTORE_FILE).exists(), "there was no file to keep");
    let first = std::fs::read(dir.file(OVERLAY_FILE)).unwrap();
    restore(&state, overlay_with_alias("ampel"), None).await.unwrap();
    assert_eq!(std::fs::read(dir.file(BEFORE_RESTORE_FILE)).unwrap(), first);
}

#[test]
fn each_section_is_held_to_its_route_validator() {
    let rule = |id: &str| PolicyRule {
        id: id.into(),
        enabled: true,
        label: String::new(),
        when: Default::default(),
        effect: PolicyEffect::Block,
        prefer: None,
        payload: None,
    };
    let refused = |overlay: Overlay, incoming: Option<StoredEndpoint>| check(Packed { overlay, endpoint: incoming }).unwrap_err();

    let twice = Overlay { policies: vec![rule("sperre"), rule("sperre")], ..Overlay::default() };
    assert_eq!(refused(twice, None), "policies");

    let no_id = Overlay { preferred: vec!["wohnzimmer".into()], ..Overlay::default() };
    assert_eq!(refused(no_id, None), "preferred");

    assert_eq!(refused(Overlay::default(), Some(endpoint("ftp://127.0.0.1/v1", "", "m"))), "llm_endpoint");
    assert_eq!(refused(Overlay::default(), Some(endpoint(LOCAL_URL, "", ""))), "llm_endpoint");
    assert!(check(Packed { overlay: overlay_with_alias("kugel"), endpoint: Some(endpoint(LOCAL_URL, "", "m")) }).is_ok());
}
