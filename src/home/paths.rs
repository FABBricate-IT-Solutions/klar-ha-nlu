use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

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

pub fn write_confined(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    let path = safe_path(confined_file(dir, name)?)?;
    std::fs::write(path, bytes)
}

pub fn write_atomic_confined(dir: &Path, name: &str, bytes: &[u8]) -> io::Result<()> {
    let path = safe_path(confined_file(dir, name)?)?;
    let tmp = safe_path(confined_file(dir, &format!("{name}.tmp"))?)?;
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(tmp, path)
}

pub fn remove_confined(dir: &Path, name: &str) -> io::Result<()> {
    let path = safe_path(confined_file(dir, name)?)?;
    std::fs::remove_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("klar-paths-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn rejects_parent_dir_in_name_and_dir() {
        let dir = temp("reject");
        assert!(confined_file(&dir, "../passwd").is_err());
        assert!(confined_file(&dir, "a/b").is_err());
        assert!(confined_file(&dir.join(".."), "klar_nlu.json").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn roundtrip_stays_under_dir() {
        let dir = temp("ok");
        write_atomic_confined(&dir, "klar_nlu.json", b"{\"aliases\":{}}").unwrap();
        let raw = read_to_string_confined(&dir, "klar_nlu.json").unwrap();
        assert!(raw.contains("aliases"));
        let path = confined_file(&dir, "klar_nlu.json").unwrap();
        assert!(path.starts_with(dir.canonicalize().unwrap()));
        remove_confined(&dir, "klar_nlu.json").unwrap();
        assert!(read_confined(&dir, "klar_nlu.json").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
