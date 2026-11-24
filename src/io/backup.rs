//! Operator settings backup: allowlisted tar.gz of overlay + LLM endpoint.

use crate::home::overlay::Overlay;
use crate::home::overlay_store::{check_storable, OverlayStore, OVERLAY_FILE};
use crate::io::auth::writes_allowed;
use crate::io::llm::{load_stored_endpoint, LLM_FILE};
use crate::io::restore::{check, install, Packed};
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
) -> Result<StatusCode, (StatusCode, String)> {
    if !writes_allowed(Some(peer), &headers, &state.token) {
        return Err((StatusCode::UNAUTHORIZED, String::new()));
    }
    if body.len() > MAX_ARCHIVE {
        return Err((StatusCode::PAYLOAD_TOO_LARGE, String::new()));
    }
    let packed = unpack(&body).map_err(|err| refused(&err.to_string()))?;
    let checked = check(packed).map_err(|section| refused(&format!("invalid section: {section}")))?;
    install(&state, checked).await.map_err(|err| {
        tracing::error!("Restore fehlgeschlagen, Dateien unverändert: {err}");
        (StatusCode::INTERNAL_SERVER_ERROR, String::new())
    })?;
    tracing::info!("Einstellungen aus Backup eingespielt, Vorher-Kopie in klar_nlu.json.before-restore");
    Ok(StatusCode::NO_CONTENT)
}

/// The archive was not installed and nothing changed. The body names the reason.
fn refused(reason: &str) -> (StatusCode, String) {
    tracing::warn!("Restore abgelehnt: {reason}");
    (StatusCode::BAD_REQUEST, reason.to_string())
}

fn wants_secrets(raw: Option<&str>) -> bool {
    matches!(raw.map(str::trim), Some("1" | "true" | "yes" | "on"))
}

fn pack_dir(dir: &Path, include_secrets: bool) -> io::Result<Vec<u8>> {
    // An unreadable file is an error here: an archive of defaults would look like a backup.
    let overlay = OverlayStore::open(dir).load_strict()?;
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
    // A value that would not read back from disk (a number too large for its field) is refused here.
    check_storable(&overlay)?;
    let endpoint = match found.get(LLM_FILE) {
        Some(raw) => Some(serde_json::from_slice(raw).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "endpoint"))?),
        None => None,
    };
    Ok(Packed { overlay, endpoint })
}

fn allowed_name(name: &str) -> bool {
    ALLOWED.contains(&name) && !name.contains("..") && !name.contains('/') && !name.contains('\\')
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
#[path = "backup_tests.rs"]
mod tests;
