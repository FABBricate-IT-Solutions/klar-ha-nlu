//! The one owner of `klar_nlu.json`.
//!
//! Every change goes through [`OverlayStore::update`]: read, edit and write happen under one
//! lock per directory, so two saves cannot lose each other's change. A file that cannot be
//! read is never overwritten by an edit; its bytes are kept as `klar_nlu.json.bad`. Each write
//! goes to a temp file of its own, is synced, read back and parsed, and only then renamed.
//! The last file that was written is kept as `klar_nlu.json.last-good`.

use super::overlay::Overlay;
use super::overlay_read::{parse_strict, parse_tolerant, OVERLAY_VERSION};
use super::paths::{confined_file, read_confined, remove_confined, sweep_temp_files, write_atomic_confined, write_checked_confined};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::convert::Infallible;
use std::hash::{Hash, Hasher};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};
use tokio::runtime::{Handle, RuntimeFlavor};

pub const OVERLAY_FILE: &str = "klar_nlu.json";
/// Bytes of a file the engine could not read, or could only read in part.
pub const BAD_FILE: &str = "klar_nlu.json.bad";
/// The file as the engine last wrote it.
pub const LAST_GOOD_FILE: &str = "klar_nlu.json.last-good";
/// The file as it was before the last restore replaced it.
pub const BEFORE_RESTORE_FILE: &str = "klar_nlu.json.before-restore";

/// What is on disk right now.
#[derive(Debug)]
pub enum OverlayRead {
    /// No file, or an empty one. There is nothing to lose.
    Missing,
    /// The file was read. `dropped` lists values that did not fit and fell back to defaults.
    Loaded { overlay: Box<Overlay>, dropped: Vec<String> },
    /// The file exists but cannot be read. It must not be overwritten by an edit.
    Unreadable(String),
}

struct Gate {
    /// Held from the read to the rename of one update.
    write: Mutex<()>,
    /// Digest of the unreadable content that was last logged and copied, to do both once.
    reported: Mutex<Option<u64>>,
}

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// One gate per directory, so every store opened on the same directory shares the lock.
fn gate_for(dir: &Path) -> Arc<Gate> {
    static GATES: OnceLock<Mutex<HashMap<PathBuf, Arc<Gate>>>> = OnceLock::new();
    let key = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    let mut gates = locked(GATES.get_or_init(Mutex::default));
    gates.entry(key).or_insert_with(|| Arc::new(Gate { write: Mutex::new(()), reported: Mutex::new(None) })).clone()
}

fn unreadable(reason: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, format!("{OVERLAY_FILE} is unreadable and was left untouched: {reason}"))
}

fn reads_back(written: &[u8]) -> io::Result<()> {
    parse_strict(written)
        .map(|_| ())
        .map_err(|reason| io::Error::new(ErrorKind::InvalidData, format!("{OVERLAY_FILE} would not read back: {reason}")))
}

/// The bytes the store would write for `overlay`. `Err` when they would not read back, for
/// example a number that does not survive the round trip.
fn storable_bytes(overlay: &Overlay) -> io::Result<Vec<u8>> {
    let mut stamped = overlay.clone();
    stamped.version = OVERLAY_VERSION;
    let bytes = serde_json::to_vec_pretty(&stamped).map_err(|err| io::Error::new(ErrorKind::InvalidData, err))?;
    reads_back(&bytes)?;
    Ok(bytes)
}

/// `Err` when `overlay` could not be stored and read again. Lets a caller refuse input
/// before anything is written.
pub fn check_storable(overlay: &Overlay) -> io::Result<()> {
    storable_bytes(overlay).map(|_| ())
}

/// Run blocking file work without stalling the async worker it was called from. On the
/// multi-thread runtime the worker hands its other tasks over first; anywhere else the work
/// simply runs.
fn off_the_reactor<T>(work: impl FnOnce() -> T) -> T {
    match Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == RuntimeFlavor::MultiThread => tokio::task::block_in_place(work),
        Ok(_) | Err(_) => work(),
    }
}

#[derive(Clone)]
pub struct OverlayStore {
    dir: PathBuf,
    gate: Arc<Gate>,
}

impl OverlayStore {
    pub fn open(dir: &Path) -> Self {
        Self { dir: dir.to_path_buf(), gate: gate_for(dir) }
    }

    pub fn path(&self) -> PathBuf {
        confined_file(&self.dir, OVERLAY_FILE).unwrap_or_else(|_| PathBuf::from(OVERLAY_FILE))
    }

    /// What is on disk, without side effects.
    pub fn read(&self) -> OverlayRead {
        self.read_raw().1
    }

    fn read_raw(&self) -> (Vec<u8>, OverlayRead) {
        let raw = match read_confined(&self.dir, OVERLAY_FILE) {
            Ok(raw) => raw,
            Err(err) if err.kind() == ErrorKind::NotFound => return (Vec::new(), OverlayRead::Missing),
            Err(err) => return (Vec::new(), OverlayRead::Unreadable(err.to_string())),
        };
        if raw.iter().all(u8::is_ascii_whitespace) {
            return (raw, OverlayRead::Missing);
        }
        let read = match parse_tolerant(&raw) {
            Ok(parsed) => OverlayRead::Loaded { overlay: Box::new(parsed.overlay), dropped: parsed.dropped },
            Err(reason) => OverlayRead::Unreadable(reason),
        };
        (raw, read)
    }

    /// Read, and when something could not be used: log it and keep the bytes, once per content.
    fn inspect(&self) -> (Vec<u8>, OverlayRead) {
        let (raw, read) = self.read_raw();
        match &read {
            OverlayRead::Unreadable(reason) => self.keep_bad(&raw, &[reason.clone()], true),
            OverlayRead::Loaded { dropped, .. } if !dropped.is_empty() => self.keep_bad(&raw, dropped, false),
            OverlayRead::Missing | OverlayRead::Loaded { .. } => {}
        }
        (raw, read)
    }

    fn keep_bad(&self, raw: &[u8], reasons: &[String], whole_file: bool) {
        let mut hasher = DefaultHasher::new();
        raw.hash(&mut hasher);
        reasons.hash(&mut hasher);
        let digest = hasher.finish();
        let mut reported = locked(&self.gate.reported);
        if *reported == Some(digest) {
            return;
        }
        *reported = Some(digest);
        let kept = raw.is_empty()
            || read_confined(&self.dir, BAD_FILE).is_ok_and(|bad| bad == raw)
            || write_atomic_confined(&self.dir, BAD_FILE, raw).is_ok();
        let copy = if raw.is_empty() {
            "keine Kopie möglich"
        } else if kept {
            "Kopie in klar_nlu.json.bad"
        } else {
            "Kopie nach klar_nlu.json.bad fehlgeschlagen"
        };
        let path = self.path();
        if whole_file {
            tracing::error!(
                "{} ist unlesbar und wird nicht überschrieben ({copy}). Speichern schlägt fehl, bis die Datei repariert oder ein Backup eingespielt ist: {}",
                path.display(),
                reasons.join("; ")
            );
        } else {
            tracing::warn!(
                "{}: {} Wert(e) nicht lesbar, dafür gelten Standardwerte ({copy}): {}",
                path.display(),
                reasons.len(),
                reasons.join("; ")
            );
        }
    }

    /// The overlay to run with. A missing file is empty. An unreadable file is left alone and
    /// the last good copy is used instead, or an empty overlay when there is none.
    pub fn load(&self) -> Overlay {
        match self.inspect().1 {
            OverlayRead::Missing => Overlay::default(),
            OverlayRead::Loaded { overlay, .. } => *overlay,
            OverlayRead::Unreadable(_) => self.last_good().unwrap_or_default(),
        }
    }

    /// Like [`Self::load`], but an unreadable file is an error instead of a fallback.
    pub fn load_strict(&self) -> io::Result<Overlay> {
        match self.inspect().1 {
            OverlayRead::Missing => Ok(Overlay::default()),
            OverlayRead::Loaded { overlay, .. } => Ok(*overlay),
            OverlayRead::Unreadable(reason) => Err(unreadable(&reason)),
        }
    }

    fn last_good(&self) -> Option<Overlay> {
        let raw = read_confined(&self.dir, LAST_GOOD_FILE).ok()?;
        parse_tolerant(&raw).ok().map(|parsed| parsed.overlay)
    }

    /// Change the overlay and write it. Nothing is written when the file on disk is
    /// unreadable, when the result would not read back, or when the disk refuses.
    pub fn update<T>(&self, edit: impl FnOnce(&mut Overlay) -> T) -> io::Result<T> {
        self.try_update(|overlay| Ok::<T, Infallible>(edit(overlay))).map(|out| match out {
            Ok(out) => out,
            Err(never) => match never {},
        })
    }

    /// Like [`Self::update`], but `edit` may refuse. A refused edit writes nothing and comes
    /// back as the inner `Err`; the outer `Err` is a save that did not reach the disk.
    pub fn try_update<T, E>(&self, edit: impl FnOnce(&mut Overlay) -> Result<T, E>) -> io::Result<Result<T, E>> {
        off_the_reactor(|| {
            let _held = locked(&self.gate.write);
            let (raw, read) = self.inspect();
            let mut overlay = match read {
                OverlayRead::Missing => Overlay::default(),
                OverlayRead::Loaded { overlay, .. } => *overlay,
                OverlayRead::Unreadable(reason) => return Err(unreadable(&reason)),
            };
            match edit(&mut overlay) {
                Ok(out) => self.write(&overlay, &raw).map(|()| Ok(out)),
                Err(refused) => Ok(Err(refused)),
            }
        })
    }

    /// Write `overlay` as the whole file, whatever is on disk. For restore, import and
    /// fixtures. An unreadable file is copied to `klar_nlu.json.bad` first.
    pub fn replace(&self, overlay: &Overlay) -> io::Result<()> {
        off_the_reactor(|| {
            let _held = locked(&self.gate.write);
            let (raw, _) = self.inspect();
            self.write(overlay, &raw)
        })
    }

    /// Like [`Self::replace`], for a restore: the file as it was is kept as
    /// `klar_nlu.json.before-restore` first. When that copy cannot be written, nothing changes.
    pub fn restore(&self, overlay: &Overlay) -> io::Result<()> {
        off_the_reactor(|| {
            let _held = locked(&self.gate.write);
            let (raw, _) = self.inspect();
            // Refuse what cannot be stored before the copy of the old file is replaced.
            storable_bytes(overlay)?;
            if raw.is_empty() {
                // No file to keep. A copy from an earlier restore would be undone to instead.
                match remove_confined(&self.dir, BEFORE_RESTORE_FILE) {
                    Err(err) if err.kind() != ErrorKind::NotFound => return Err(err),
                    Ok(()) | Err(_) => {}
                }
            } else {
                write_atomic_confined(&self.dir, BEFORE_RESTORE_FILE, &raw)?;
            }
            self.write(overlay, &raw)
        })
    }

    /// Put the file back as it was before the last [`Self::restore`], byte for byte.
    pub fn undo_restore(&self) -> io::Result<()> {
        off_the_reactor(|| {
            let _held = locked(&self.gate.write);
            match read_confined(&self.dir, BEFORE_RESTORE_FILE) {
                Ok(before) => write_atomic_confined(&self.dir, OVERLAY_FILE, &before),
                Err(err) if err.kind() == ErrorKind::NotFound => match remove_confined(&self.dir, OVERLAY_FILE) {
                    Err(err) if err.kind() != ErrorKind::NotFound => Err(err),
                    Ok(()) | Err(_) => Ok(()),
                },
                Err(err) => Err(err),
            }
        })
    }

    fn write(&self, overlay: &Overlay, on_disk: &[u8]) -> io::Result<()> {
        // Checked before touching the disk too, so a value that cannot be stored costs no write.
        let bytes = storable_bytes(overlay)?;
        if bytes != on_disk {
            write_checked_confined(&self.dir, OVERLAY_FILE, &bytes, reads_back)?;
        }
        self.keep_last_good(&bytes);
        Ok(())
    }

    fn keep_last_good(&self, bytes: &[u8]) {
        if read_confined(&self.dir, LAST_GOOD_FILE).is_ok_and(|kept| kept == bytes) {
            return;
        }
        if let Err(err) = write_atomic_confined(&self.dir, LAST_GOOD_FILE, bytes) {
            tracing::warn!("{LAST_GOOD_FILE} konnte nicht geschrieben werden: {err}");
        }
    }

    /// Startup housekeeping: remove temp files a crash left behind, and keep a last good copy
    /// of a file that reads cleanly.
    pub fn checkpoint(&self) {
        let _held = locked(&self.gate.write);
        // Temp files of the copies start with the same name, so one sweep covers all three.
        sweep_temp_files(&self.dir, OVERLAY_FILE);
        let (raw, read) = self.inspect();
        if matches!(&read, OverlayRead::Loaded { dropped, .. } if dropped.is_empty()) {
            self.keep_last_good(&raw);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::overlay::UiPoint;
    use crate::home::test_dir::TestDir;

    const HALF_A_FILE: &[u8] = b"{\"aliases\": {\"light.kellertreppe\": [\"stiege\"]}, \"custom\": [";

    #[test]
    fn a_refused_edit_writes_nothing() {
        let dir = TestDir::new("refused");
        let store = OverlayStore::open(dir.path());
        store.update(|overlay| overlay.preferred.push("light.kellertreppe".into())).unwrap();
        let before = std::fs::read(dir.file(OVERLAY_FILE)).unwrap();
        let refused = store.try_update(|overlay| {
            overlay.preferred.clear();
            Err::<(), _>("nein")
        });
        assert_eq!(refused.unwrap(), Err("nein"));
        assert_eq!(std::fs::read(dir.file(OVERLAY_FILE)).unwrap(), before);
        assert_eq!(store.load().preferred, ["light.kellertreppe"]);
    }

    #[test]
    fn a_missing_and_an_empty_file_are_not_unreadable() {
        let dir = TestDir::new("missing");
        let store = OverlayStore::open(dir.path());
        assert!(matches!(store.read(), OverlayRead::Missing));
        std::fs::write(dir.file(OVERLAY_FILE), b" \n").unwrap();
        assert!(matches!(store.read(), OverlayRead::Missing));
        store.update(|overlay| overlay.preferred.push("light.kellertreppe".into())).unwrap();
        assert!(matches!(store.read(), OverlayRead::Loaded { .. }));
        assert_eq!(dir.names(), [OVERLAY_FILE, LAST_GOOD_FILE]);
    }

    #[test]
    fn an_unreadable_file_is_kept_and_the_last_good_copy_is_served() {
        let dir = TestDir::new("fallback");
        let store = OverlayStore::open(dir.path());
        store.update(|overlay| overlay.preferred.push("light.kellertreppe".into())).unwrap();
        std::fs::write(dir.file(OVERLAY_FILE), HALF_A_FILE).unwrap();

        assert!(matches!(store.read(), OverlayRead::Unreadable(_)));
        assert_eq!(store.load().preferred, ["light.kellertreppe"], "the last good copy stands in");
        assert!(store.load_strict().is_err());
        let refused = store.update(|overlay| overlay.preferred.clear());
        assert_eq!(refused.unwrap_err().kind(), ErrorKind::InvalidData);
        assert_eq!(std::fs::read(dir.file(OVERLAY_FILE)).unwrap(), HALF_A_FILE);
        assert_eq!(std::fs::read(dir.file(BAD_FILE)).unwrap(), HALF_A_FILE);
    }

    #[test]
    fn replace_repairs_an_unreadable_file_and_keeps_its_bytes() {
        let dir = TestDir::new("replace");
        std::fs::write(dir.file(OVERLAY_FILE), HALF_A_FILE).unwrap();
        let store = OverlayStore::open(dir.path());
        let mut next = Overlay::default();
        next.preferred.push("light.dachboden".into());
        store.replace(&next).unwrap();
        assert_eq!(store.load_strict().unwrap().preferred, ["light.dachboden"]);
        assert_eq!(std::fs::read(dir.file(BAD_FILE)).unwrap(), HALF_A_FILE);
    }

    #[test]
    fn every_write_carries_the_version() {
        let dir = TestDir::new("version");
        std::fs::write(dir.file(OVERLAY_FILE), br#"{"preferred": ["light.dachboden"]}"#).unwrap();
        let store = OverlayStore::open(dir.path());
        assert_eq!(store.load().version, 0, "a file from before the field is version 0");
        store.update(|_| ()).unwrap();
        let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(dir.file(OVERLAY_FILE)).unwrap()).unwrap();
        assert_eq!(saved["version"], OVERLAY_VERSION);
        assert_eq!(saved["preferred"][0], "light.dachboden");
    }

    #[test]
    fn checkpoint_sweeps_temp_files_and_keeps_a_copy_of_a_clean_file() {
        let dir = TestDir::new("checkpoint");
        let clean = br#"{"preferred": ["light.dachboden"]}"#;
        std::fs::write(dir.file(OVERLAY_FILE), clean).unwrap();
        std::fs::write(dir.file("klar_nlu.json.0f0f.tmp"), b"{").unwrap();
        std::fs::write(dir.file("klar_nlu.json.tmp"), b"{").unwrap();
        OverlayStore::open(dir.path()).checkpoint();
        assert_eq!(dir.names(), [OVERLAY_FILE, LAST_GOOD_FILE]);
        assert_eq!(std::fs::read(dir.file(LAST_GOOD_FILE)).unwrap(), clean);
    }

    #[test]
    fn a_number_that_would_not_read_back_is_not_storable() {
        let mut overlay = Overlay::default();
        overlay.ui.graph.insert("light.dachboden".into(), UiPoint { x: f32::INFINITY, y: 1.0 });
        assert!(check_storable(&overlay).is_err());
        let dir = TestDir::new("storable");
        let store = OverlayStore::open(dir.path());
        assert!(store.replace(&overlay).is_err());
        assert!(dir.names().is_empty(), "{:?}", dir.names());
    }
}
