use super::*;
use crate::home::test_dir::TestDir;
use crate::io::llm::{save_stored_endpoint, StoredEndpoint};
use crate::types::{PolicyEffect, PolicyMatch, PolicyRule, Settings};

fn sample_overlay() -> Overlay {
    let mut overlay =
        Overlay { settings: Some(Settings { refine_speech: true, nlu_rag: true, ..Settings::default() }), ..Overlay::default() };
    overlay.aliases.insert("light.wohnzimmer".into(), vec!["kugel".into()]);
    overlay.areas.insert("light.wohnzimmer".into(), "wohnzimmer".into());
    overlay.policies.push(PolicyRule {
        id: "block-lock".into(),
        enabled: true,
        label: "lock".into(),
        when: PolicyMatch { domain: Some("lock".into()), ..Default::default() },
        effect: PolicyEffect::Block,
        prefer: None,
        payload: None,
    });
    overlay
}

fn write_overlay(dir: &Path, overlay: &Overlay) {
    OverlayStore::open(dir).replace(overlay).unwrap();
}

#[test]
fn roundtrip_keeps_settings_alias_and_policy() {
    let dir = TestDir::new("backup-round");
    let dir = dir.path();
    write_overlay(dir, &sample_overlay());
    let bytes = pack_dir(dir, false).unwrap();
    let packed = unpack(&bytes).unwrap();
    assert!(packed.overlay.settings.as_ref().is_some_and(|s| s.refine_speech && s.nlu_rag));
    assert_eq!(packed.overlay.aliases.get("light.wohnzimmer").map(Vec::as_slice), Some(["kugel".to_string()].as_slice()));
    assert_eq!(packed.overlay.policies[0].id, "block-lock");
}

#[test]
fn default_export_omits_api_key() {
    let dir = TestDir::new("backup-secret");
    let dir = dir.path();
    write_overlay(dir, &Overlay::default());
    save_stored_endpoint(
        dir,
        &StoredEndpoint {
            base_url: "http://127.0.0.1:8000/v1".into(),
            api_key: "sk-secret".into(),
            model: "gemma".into(),
            enable_thinking: false,
            provider: crate::llm::LlmProviderKind::Custom,
        },
    )
    .unwrap();
    let packed = unpack(&pack_dir(dir, false).unwrap()).unwrap();
    assert_eq!(packed.endpoint.as_ref().map(|e| e.api_key.as_str()), Some(""));
    assert!(!String::from_utf8_lossy(&pack_dir(dir, false).unwrap()).contains("sk-secret"));
    let with = unpack(&pack_dir(dir, true).unwrap()).unwrap();
    assert_eq!(with.endpoint.as_ref().map(|e| e.api_key.as_str()), Some("sk-secret"));
}

fn octal_field(slot: &mut [u8], value: u64) {
    let text = format!("{:0width$o}", value, width = slot.len().saturating_sub(1));
    slot[..text.len()].copy_from_slice(text.as_bytes());
    if text.len() < slot.len() {
        slot[text.len()] = 0;
    }
}

fn gnu_header(name: &str, size: u64) -> [u8; 512] {
    let mut header = [0u8; 512];
    let bytes = name.as_bytes();
    header[..bytes.len()].copy_from_slice(bytes);
    octal_field(&mut header[100..108], 0o644);
    octal_field(&mut header[124..136], size);
    header[156] = b'0';
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    header[148..156].copy_from_slice(b"        ");
    let sum: u32 = header.iter().map(|b| u32::from(*b)).sum();
    let checksum = format!("{sum:06o}\0 ");
    header[148..156].copy_from_slice(checksum.as_bytes());
    header
}

fn raw_tar_gz(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut tar_bytes = Vec::new();
    for (name, data) in files {
        tar_bytes.extend_from_slice(&gnu_header(name, data.len() as u64));
        tar_bytes.extend_from_slice(data);
        let pad = (512 - (data.len() % 512)) % 512;
        tar_bytes.extend(std::iter::repeat(0u8).take(pad));
    }
    tar_bytes.extend_from_slice(&[0u8; 1024]);
    let mut gz = Vec::new();
    let mut enc = GzEncoder::new(&mut gz, Compression::default());
    enc.write_all(&tar_bytes).unwrap();
    enc.finish().unwrap();
    gz
}

#[test]
fn rejects_traversal_and_extra_files() {
    let evil = raw_tar_gz(&[(MANIFEST, b"{}"), ("../evil", b"no")]);
    assert!(unpack(&evil).is_err());
    let extra = raw_tar_gz(&[
        (MANIFEST, b"{\"format\":\"klar-settings-v1\",\"engine\":\"1\",\"created_at\":\"t\",\"include_secrets\":false,\"files\":[]}"),
        (OVERLAY_FILE, b"{}"),
        ("notes.txt", b"no"),
    ]);
    assert!(unpack(&extra).is_err());
}

#[test]
fn unknown_format_leaves_disk() {
    let dir = TestDir::new("backup-fmt");
    let dir = dir.path();
    write_overlay(dir, &sample_overlay());
    let before = std::fs::read(dir.join(OVERLAY_FILE)).unwrap();
    let bad = pack_files(&[
        (
            MANIFEST,
            serde_json::to_vec(&Manifest {
                format: "klar-settings-v0".into(),
                engine: "1".into(),
                created_at: "t".into(),
                include_secrets: false,
                files: vec![OVERLAY_FILE.into()],
            })
            .unwrap(),
        ),
        (OVERLAY_FILE, b"{}".to_vec()),
    ])
    .unwrap();
    assert!(unpack(&bad).is_err());
    assert_eq!(std::fs::read(dir.join(OVERLAY_FILE)).unwrap(), before);
}

#[test]
fn allowed_names_are_leaves() {
    assert!(allowed_name("klar_nlu.json"));
    assert!(!allowed_name("../klar_nlu.json"));
    assert!(!allowed_name("dir/klar_nlu.json"));
    assert!(!allowed_name("notes.txt"));
}
