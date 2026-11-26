use std::fs::{File, OpenOptions, Permissions};
use std::io::{self, ErrorKind, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use uuid::Uuid;

fn invalid(message: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidInput, message)
}

/// Resolve `dir/name` only when `name` is a single leaf and the result stays under `dir`.
///
/// CodeQL `rust/path-injection` treats Axum `State` as remote input, so every data-dir
/// file access must go through this check (`contains("..")`, `canonicalize`, `starts_with`).
pub fn confined_file(dir: &Path, name: &str) -> io::Result<PathBuf> {
    if name.contains("..") || name.contains("/") || name.contains("\\") {
        return Err(invalid("unsafe file name"));
    }
    let raw = dir.to_string_lossy();
    if raw.contains("..") {
        return Err(invalid("unsafe data directory"));
    }
    let dir = PathBuf::from(raw.as_ref());
    let _ = std::fs::create_dir_all(&dir);
    let base = dir.canonicalize()?;
    let path = base.join(name);
    if !path.starts_with(&base) {
        return Err(invalid("path escapes directory"));
    }
    match path.canonicalize() {
        Ok(resolved) => {
            if !resolved.starts_with(&base) {
                return Err(invalid("path escapes directory"));
            }
            Ok(resolved)
        }
        Err(_) => Ok(path),
    }
}

fn safe_path(path: PathBuf) -> io::Result<PathBuf> {
    let raw = path.to_string_lossy();
    if raw.contains("..") {
        return Err(invalid("unsafe path"));
    }
    Ok(PathBuf::from(raw.as_ref()))
}

pub fn read_confined(dir: &Path, name: &str) -> io::Result<Vec<u8>> {
    let path = safe_path(confined_file(dir, name)?)?;
    std::fs::read(path)
}

pub fn read_to_string_confined(dir: &Path, name: &str) -> io::Result<String> {
    let path = safe_path(confined_file(dir, name)?)?;
    std::fs::read_to_string(path)
}

/// Non-atomic replace. Prefer [`write_atomic_confined`] when a kill mid-write must leave the
/// previous file readable.
#[allow(dead_code)]
pub fn write_confined(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    let path = safe_path(confined_file(dir, name)?)?;
    std::fs::write(path, bytes)
}

/// Append bytes to `dir/name` and sync. A kill mid-write can leave a short last line; every
/// earlier line stays readable (jsonl). Prefer this for append-only journals.
pub fn append_confined(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    let path = safe_path(confined_file(dir, name)?)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// Replace `dir/name` in one step: the bytes go to a temp file of their own, are synced, and
/// only then renamed over the target. A crash or a second writer never leaves a half file.
pub fn write_atomic_confined(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    write_checked_confined(dir, name, bytes, |_| Ok(()))
}

/// Like [`write_atomic_confined`], but the temp file is read back and handed to `check`
/// before the rename. When the bytes differ or `check` fails, the target is not touched.
pub fn write_checked_confined(dir: &Path, name: &str, bytes: &[u8], check: impl Fn(&[u8]) -> io::Result<()>) -> io::Result<()> {
    let path = safe_path(confined_file(dir, name)?)?;
    let tmp = safe_path(confined_file(dir, &format!("{name}.{}{TEMP_SUFFIX}", Uuid::new_v4().simple()))?)?;
    let written = write_synced(&tmp, bytes).and_then(|()| {
        let back = std::fs::read(&tmp)?;
        if back != bytes {
            return Err(io::Error::new(ErrorKind::InvalidData, "temp file does not hold what was written"));
        }
        check(&back)?;
        std::fs::rename(&tmp, &path)
    });
    if written.is_err() {
        let _ = std::fs::remove_file(&tmp);
        return written;
    }
    if let Some(parent) = path.parent() {
        // Make the rename itself durable. Not every filesystem lets a directory be synced.
        let _ = File::open(parent).and_then(|handle| handle.sync_all());
    }
    Ok(())
}

const TEMP_SUFFIX: &str = ".tmp";
/// Owner may read and write, nobody else. For files that hold a secret.
const PRIVATE_MODE: u32 = 0o600;

fn write_synced(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

/// A file that is written and synced under a temp name and not yet in place. [`Staged::commit`]
/// renames it over the target; dropping it without a commit removes the temp file.
pub struct Staged {
    tmp: PathBuf,
    path: PathBuf,
    committed: bool,
}

impl Staged {
    pub fn commit(mut self) -> io::Result<()> {
        std::fs::rename(&self.tmp, &self.path)?;
        self.committed = true;
        if let Some(parent) = self.path.parent() {
            let _ = File::open(parent).and_then(|handle| handle.sync_all());
        }
        Ok(())
    }
}

impl Drop for Staged {
    fn drop(&mut self) {
        if !self.committed {
            let _ = std::fs::remove_file(&self.tmp);
        }
    }
}

/// Prepare `dir/name` with owner-only permissions, so that the only step left is a rename.
pub fn stage_private_confined(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<Staged> {
    let path = safe_path(confined_file(dir, name)?)?;
    let tmp = safe_path(confined_file(dir, &format!("{name}.{}{TEMP_SUFFIX}", Uuid::new_v4().simple()))?)?;
    let staged = Staged { tmp, path, committed: false };
    let mut file = OpenOptions::new().write(true).create_new(true).mode(PRIVATE_MODE).open(&staged.tmp)?;
    // The mode given at creation is cut by the umask only; set it again in case the name existed.
    file.set_permissions(Permissions::from_mode(PRIVATE_MODE))?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(staged)
}

/// Replace `dir/name` in one step with a file only the owner can read.
pub fn write_private_confined(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    stage_private_confined(dir, name, bytes)?.commit()
}

/// Remove temp files a crashed write of `dir/name` left behind. Call before the first write.
pub fn sweep_temp_files(dir: &Path, name: &str) {
    let Ok(base) = confined_file(dir, name) else {
        return;
    };
    let Some(parent) = base.parent() else {
        return;
    };
    let prefix = format!("{name}.");
    for entry in std::fs::read_dir(parent).into_iter().flatten().flatten() {
        let leaf = entry.file_name().to_string_lossy().into_owned();
        if leaf.starts_with(&prefix) && leaf.ends_with(TEMP_SUFFIX) {
            let _ = remove_confined(dir, &leaf);
        }
    }
}

pub fn remove_confined(dir: &Path, name: &str) -> io::Result<()> {
    let path = safe_path(confined_file(dir, name)?)?;
    std::fs::remove_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::home::test_dir::TestDir;

    #[test]
    fn rejects_parent_dir_in_name_and_dir() {
        let scratch = TestDir::new("paths-reject");
        let dir = scratch.path().to_path_buf();
        assert!(confined_file(&dir, "../passwd").is_err());
        assert!(confined_file(&dir, "a/b").is_err());
        assert!(confined_file(&dir.join(".."), "klar_nlu.json").is_err());
    }

    #[test]
    fn roundtrip_stays_under_dir() {
        let scratch = TestDir::new("paths-ok");
        let dir = scratch.path().to_path_buf();
        write_atomic_confined(&dir, "klar_nlu.json", b"{\"aliases\":{}}").unwrap();
        let raw = read_to_string_confined(&dir, "klar_nlu.json").unwrap();
        assert!(raw.contains("aliases"));
        let path = confined_file(&dir, "klar_nlu.json").unwrap();
        assert!(path.starts_with(dir.canonicalize().unwrap()));
        remove_confined(&dir, "klar_nlu.json").unwrap();
        assert!(read_confined(&dir, "klar_nlu.json").is_err());
    }

    #[test]
    fn a_failed_check_keeps_the_old_file_and_leaves_no_temp_file() {
        let dir = TestDir::new("check");
        write_atomic_confined(dir.path(), "state.json", b"old").unwrap();
        let refused = write_checked_confined(dir.path(), "state.json", b"new", |_| Err(invalid("no")));
        assert!(refused.is_err());
        assert_eq!(read_confined(dir.path(), "state.json").unwrap(), b"old");
        assert_eq!(dir.names(), ["state.json"]);
    }

    #[test]
    fn a_private_file_is_owner_only_and_an_uncommitted_one_leaves_nothing() {
        let dir = TestDir::new("private");
        write_private_confined(dir.path(), "secret.json", b"one").unwrap();
        let mode = std::fs::metadata(dir.file("secret.json")).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
        drop(stage_private_confined(dir.path(), "secret.json", b"two").unwrap());
        assert_eq!(read_confined(dir.path(), "secret.json").unwrap(), b"one");
        assert_eq!(dir.names(), ["secret.json"]);
    }

    #[test]
    fn sweep_removes_only_temp_files_of_that_name() {
        let dir = TestDir::new("sweep");
        for name in ["state.json", "state.json.tmp", "state.json.0123.tmp", "state.json.bad", "other.json.tmp"] {
            std::fs::write(dir.file(name), b"x").unwrap();
        }
        sweep_temp_files(dir.path(), "state.json");
        assert_eq!(dir.names(), ["other.json.tmp", "state.json", "state.json.bad"]);
    }

    #[test]
    fn append_keeps_prior_bytes_readable() {
        let dir = TestDir::new("append");
        append_confined(dir.path(), "log.jsonl", b"{\"a\":1}\n").unwrap();
        append_confined(dir.path(), "log.jsonl", b"{\"a\":2}\n").unwrap();
        assert_eq!(read_to_string_confined(dir.path(), "log.jsonl").unwrap(), "{\"a\":1}\n{\"a\":2}\n");
    }
}
