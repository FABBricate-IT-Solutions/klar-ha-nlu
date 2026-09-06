//! Operator settings backup: allowlisted tar.gz of overlay + LLM endpoint.

use crate::home::overlay::{apply_overlay, load_overlay, save_overlay, Overlay};
use crate::io::auth::writes_allowed;
use crate::io::llm::{load_endpoint, load_stored_endpoint, save_stored_endpoint, StoredEndpoint, LLM_FILE};
use crate::io::state::AppState;
use axum::body::Bytes;
use axum::extract::{ConnectInfo, DefaultBodyLimit, Query, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tar::{Archive, Builder, Header};

const FORMAT: &str = "klar-settings-v1";
const MANIFEST: &str = "manifest.json";
const OVERLAY_FILE: &str = "klar_nlu.json";
const MAX_ARCHIVE: usize = 8 * 1024 * 1024;
const MAX_ENTRIES: usize = 8;
const ALLOWED: &[&str] = &[MANIFEST, OVERLAY_FILE, LLM_FILE];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Manifest {
    format: String,
    engine: String,
    created_at: String,
    include_secrets: bool,
    files: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct BackupQuery {
    #[serde(default)]
    secrets: Option<String>,
}

#[derive(Debug)]
struct Packed {
    overlay: Overlay,
    endpoint: Option<StoredEndpoint>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/v2/settings/backup", get(download_backup))
        .route("/api/v2/settings/restore", post(restore_backup))
        .layer(DefaultBodyLimit::max(MAX_ARCHIVE))
}

async fn download_backup(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<BackupQuery>,
) -> Result<impl IntoResponse, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let include_secrets = wants_secrets(query.secrets.as_deref());
    let bytes = pack_dir(&state.data_dir, include_secrets).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let name = format!("klar-settings-{}.tar.gz", utc_yyyymmdd());
    let mut disposition =
        HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")).unwrap_or(HeaderValue::from_static("attachment"));
    disposition.set_sensitive(false);
    Ok(([(header::CONTENT_TYPE, HeaderValue::from_static("application/gzip")), (header::CONTENT_DISPOSITION, disposition)], bytes))
}

async fn restore_backup(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<StatusCode, StatusCode> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if body.len() > MAX_ARCHIVE {
        return Err(StatusCode::PAYLOAD_TOO_LARGE);
    }
    let packed = unpack(&body).map_err(|err| match err.kind() {
        io::ErrorKind::InvalidData | io::ErrorKind::InvalidInput => StatusCode::BAD_REQUEST,
        _ => StatusCode::BAD_REQUEST,
    })?;
    apply_packed(&state, packed).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

fn wants_secrets(raw: Option<&str>) -> bool {
    matches!(raw.map(str::trim), Some("1" | "true" | "yes" | "on"))
}

fn pack_dir(dir: &Path, include_secrets: bool) -> io::Result<Vec<u8>> {
    let overlay = load_overlay(dir);
    let overlay_bytes = serde_json::to_vec_pretty(&overlay).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    let endpoint = load_stored_endpoint(dir).map(|mut stored| {
        if !include_secrets {
            stored.api_key.clear();
        }
        stored
    });
    let mut files = vec![(MANIFEST, Vec::new()), (OVERLAY_FILE, overlay_bytes)];
    if let Some(stored) = &endpoint {
        let bytes = serde_json::to_vec_pretty(stored).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        files.push((LLM_FILE, bytes));
    }
    let names: Vec<String> = files.iter().filter(|(name, _)| *name != MANIFEST).map(|(name, _)| (*name).to_string()).collect();
    let manifest = Manifest {
        format: FORMAT.into(),
        engine: env!("CARGO_PKG_VERSION").into(),
        created_at: utc_stamp(),
        include_secrets,
        files: names,
    };
    files[0].1 = serde_json::to_vec_pretty(&manifest).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    pack_files(&files)
}

fn pack_files(files: &[(&str, Vec<u8>)]) -> io::Result<Vec<u8>> {
    let mut tar_bytes = Vec::new();
    {
        let mut builder = Builder::new(&mut tar_bytes);
        for (name, data) in files {
            if !allowed_name(name) {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "unsafe file name"));
            }
            let mut header = Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, name, data.as_slice())?;
        }
        builder.finish()?;
    }
    let mut gz = Vec::new();
    let mut enc = GzEncoder::new(&mut gz, Compression::default());
    enc.write_all(&tar_bytes)?;
    enc.finish()?;
    if gz.len() > MAX_ARCHIVE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "backup too large"));
    }
    Ok(gz)
}

fn unpack(bytes: &[u8]) -> io::Result<Packed> {
    if bytes.len() > MAX_ARCHIVE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "backup too large"));
    }
    let tar_bytes = if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut dec = GzDecoder::new(bytes);
        let mut out = Vec::new();
        let mut buf = [0u8; 8192];
        loop {
            let n = dec.read(&mut buf)?;
            if n == 0 {
                break;
            }
            if out.len() + n > MAX_ARCHIVE {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "backup too large"));
            }
            out.extend_from_slice(&buf[..n]);
        }
        out
    } else {
        bytes.to_vec()
    };
    let mut archive = Archive::new(tar_bytes.as_slice());
    let mut found = BTreeMap::new();
    for entry in archive.entries()? {
        let mut entry = entry?;
        if found.len() >= MAX_ENTRIES {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "too many files"));
        }
        if !entry.header().entry_type().is_file() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "only regular files"));
        }
        let name = entry.path()?.to_string_lossy().into_owned();
        if !allowed_name(&name) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "unsafe file name"));
        }
        let mut data = Vec::new();
        entry.read_to_end(&mut data)?;
        if data.len() > MAX_ARCHIVE {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "backup too large"));
        }
        found.insert(name, data);
    }
    let manifest: Manifest =
        serde_json::from_slice(found.get(MANIFEST).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "manifest"))?)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "manifest"))?;
    if manifest.format != FORMAT {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "unknown backup format"));
    }
    for name in found.keys() {
        if !ALLOWED.contains(&name.as_str()) {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "unexpected file"));
        }
    }
    let overlay: Overlay =
        serde_json::from_slice(found.get(OVERLAY_FILE).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "overlay"))?)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "overlay"))?;
    let endpoint = match found.get(LLM_FILE) {
        Some(raw) => Some(serde_json::from_slice(raw).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "endpoint"))?),
        None => None,
    };
    Ok(Packed { overlay, endpoint })
}

fn allowed_name(name: &str) -> bool {
    ALLOWED.contains(&name) && !name.contains("..") && !name.contains('/') && !name.contains('\\')
}

async fn apply_packed(state: &AppState, packed: Packed) -> io::Result<()> {
    save_overlay(&state.data_dir, &packed.overlay)?;
    if let Some(mut stored) = packed.endpoint {
        if stored.api_key.is_empty() {
            if let Some(local) = load_stored_endpoint(&state.data_dir) {
                stored.api_key = local.api_key;
            }
        }
        if !stored.base_url.trim().is_empty() && !stored.model.trim().is_empty() {
            save_stored_endpoint(&state.data_dir, &stored)?;
        }
    }
    *state.settings.lock().await = packed.overlay.settings.clone().unwrap_or_default();
    *state.custom.lock().await = packed.overlay.custom.clone();
    *state.policies.lock().await = packed.overlay.policies.clone();
    *state.speech_bank.lock().await = packed.overlay.speech_bank.clone();
    *state.match_controls.lock().await = packed.overlay.match_controls.clone();
    crate::lang::install_user_overlay(if packed.overlay.language.sets.is_empty() { None } else { Some(packed.overlay.language.clone()) });
    state
        .home
        .edit(|graph| {
            apply_overlay(graph, &packed.overlay);
        })
        .await;
    *state.llm.lock().await = load_endpoint(&state.data_dir);
    Ok(())
}

fn utc_stamp() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let (year, month, day) = utc_ymd(secs);
    format!("{year:04}-{month:02}-{day:02}T00:00:00Z")
}

fn utc_yyyymmdd() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let (year, month, day) = utc_ymd(secs);
    format!("{year:04}{month:02}{day:02}")
}

fn utc_ymd(secs: u64) -> (i32, u32, u32) {
    let z = (secs / 86_400) as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 24 + yoe / 1460);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::load::LoadedHome;
    use crate::home::sample::default_home;
    use crate::types::{PolicyEffect, PolicyMatch, PolicyRule, Settings};

    fn temp(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("klar-backup-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_overlay() -> Overlay {
        let mut overlay = Overlay::default();
        overlay.settings = Some(Settings { refine_speech: true, nlu_rag: true, ..Settings::default() });
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
        save_overlay(dir, overlay).unwrap();
    }

    #[test]
    fn roundtrip_keeps_settings_alias_and_policy() {
        let dir = temp("round");
        write_overlay(&dir, &sample_overlay());
        let bytes = pack_dir(&dir, false).unwrap();
        let packed = unpack(&bytes).unwrap();
        assert!(packed.overlay.settings.as_ref().is_some_and(|s| s.refine_speech && s.nlu_rag));
        assert_eq!(packed.overlay.aliases.get("light.wohnzimmer").map(Vec::as_slice), Some(["kugel".to_string()].as_slice()));
        assert_eq!(packed.overlay.policies[0].id, "block-lock");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn default_export_omits_api_key() {
        let dir = temp("secret");
        write_overlay(&dir, &Overlay::default());
        save_stored_endpoint(
            &dir,
            &StoredEndpoint {
                base_url: "http://127.0.0.1:8000/v1".into(),
                api_key: "sk-secret".into(),
                model: "gemma".into(),
                enable_thinking: false,
            },
        )
        .unwrap();
        let packed = unpack(&pack_dir(&dir, false).unwrap()).unwrap();
        assert_eq!(packed.endpoint.as_ref().map(|e| e.api_key.as_str()), Some(""));
        assert!(!String::from_utf8_lossy(&pack_dir(&dir, false).unwrap()).contains("sk-secret"));
        let with = unpack(&pack_dir(&dir, true).unwrap()).unwrap();
        assert_eq!(with.endpoint.as_ref().map(|e| e.api_key.as_str()), Some("sk-secret"));
        let _ = std::fs::remove_dir_all(&dir);
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
        let dir = temp("fmt");
        write_overlay(&dir, &sample_overlay());
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
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn restore_empty_key_keeps_local() {
        let dir = temp("keep-key");
        write_overlay(&dir, &Overlay::default());
        save_stored_endpoint(
            &dir,
            &StoredEndpoint {
                base_url: "http://127.0.0.1:8000/v1".into(),
                api_key: "sk-local".into(),
                model: "old".into(),
                enable_thinking: false,
            },
        )
        .unwrap();
        let incoming = StoredEndpoint {
            base_url: "http://192.168.1.2:8000/v1".into(),
            api_key: String::new(),
            model: "gemma".into(),
            enable_thinking: true,
        };
        let state = AppState::new(
            LoadedHome {
                graph: default_home(),
                settings: Settings::default(),
                custom: Vec::new(),
                language: Default::default(),
                policies: Vec::new(),
                speech_bank: Default::default(),
                match_controls: Vec::new(),
            },
            dir.clone(),
            None,
        );
        apply_packed(&state, Packed { overlay: sample_overlay(), endpoint: Some(incoming) }).await.unwrap();
        let stored = load_stored_endpoint(&dir).unwrap();
        assert_eq!(stored.api_key, "sk-local");
        assert_eq!(stored.model, "gemma");
        assert_eq!(stored.base_url, "http://192.168.1.2:8000/v1");
        assert!(state.settings.lock().await.refine_speech);
        crate::lang::reset_runtime_packs();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn allowed_names_are_leaves() {
        assert!(allowed_name("klar_nlu.json"));
        assert!(!allowed_name("../klar_nlu.json"));
        assert!(!allowed_name("dir/klar_nlu.json"));
        assert!(!allowed_name("notes.txt"));
    }
}
