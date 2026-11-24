use super::harness::serve_models;
use crate::llm::client::list_models_with;
use crate::llm::target::GuardedResolver;
use crate::llm::{LlmEndpoint, LlmError};
use serde_json::json;
use std::net::SocketAddr;

/// A resolver that answers every name with one fixed address, the way a public DNS name can
/// be pointed at any address.
fn pointing_at(addr: &str) -> GuardedResolver {
    let addr: SocketAddr = addr.parse().unwrap();
    GuardedResolver::with_lookup(move |_| async move { Ok(vec![addr]) })
}

#[tokio::test]
async fn a_name_that_resolves_to_an_open_address_is_reached_through_the_resolver() {
    let (base, handle) = serve_models(json!({"data": [{"id": "kleines-modell"}]})).await;
    let port = base.trim_end_matches("/v1").rsplit(':').next().unwrap().to_string();
    let endpoint = LlmEndpoint::for_discovery(&format!("http://llm.example.test:{port}/v1"), "").unwrap();
    let models = list_models_with(&endpoint, pointing_at("127.0.0.1:0")).await.unwrap();
    assert_eq!(models, vec!["kleines-modell"]);
    handle.abort();
}

#[tokio::test]
async fn a_name_that_resolves_to_a_blocked_address_gets_no_connection() {
    let endpoint = LlmEndpoint::for_discovery("http://llm.example.test:9/v1", "").unwrap();
    for inward in ["169.254.169.254:0", "100.100.100.200:0", "172.30.32.2:0", "[fe80::1]:0"] {
        let refused = list_models_with(&endpoint, pointing_at(inward)).await.unwrap_err();
        assert!(matches!(refused, LlmError::Transport), "{inward}: {refused:?}");
    }
}

#[test]
fn blocked_addresses_and_names_are_refused_as_written() {
    for base in [
        "http://169.254.169.254/v1",
        "http://2852039166/v1",
        "http://100.100.100.200/v1",
        "http://172.30.32.2/v1",
        "http://supervisor/v1",
        "http://SUPERVISOR./core/api",
        "http://hassio/v1",
        "http://[fe80::1]/v1",
        "http://[::ffff:169.254.169.254]/v1",
    ] {
        assert!(LlmEndpoint::for_discovery(base, "").is_err(), "{base}");
    }
    for base in
        ["http://127.0.0.1:11434/v1", "http://172.30.33.4:8000/v1", "http://100.101.102.103:8000/v1", "http://ollama.local:11434/v1"]
    {
        assert!(LlmEndpoint::for_discovery(base, "").is_ok(), "{base}");
    }
}

#[test]
fn the_same_base_is_the_same_scheme_host_port_and_path() {
    let stored = LlmEndpoint::from_parts("http://192.168.178.15:8000/v1", "", "m").unwrap();
    for same in ["http://192.168.178.15:8000/v1", "http://192.168.178.15:8000/v1/", " http://192.168.178.15:8000/v1 "] {
        assert!(stored.same_base(same), "{same}");
    }
    for other in [
        "http://192.168.178.16:8000/v1",
        "http://192.168.178.15:8001/v1",
        "https://192.168.178.15:8000/v1",
        "http://192.168.178.15:8000/v2",
        "http://192.168.178.15.nip.io:8000/v1",
        "",
        "kein url",
    ] {
        assert!(!stored.same_base(other), "{other}");
    }
    let cloud = LlmEndpoint::from_parts("https://API.OpenAI.com", "", "m").unwrap();
    assert!(cloud.same_base("https://api.openai.com:443/v1"));
    assert!(cloud.same_base(""), "an empty base means the default, which is this one");
}
