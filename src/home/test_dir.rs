//! A scratch directory for unit tests. It is removed when dropped, also when the test panics.
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub(crate) struct TestDir(TempDir);

impl TestDir {
    pub(crate) fn new(tag: &str) -> Self {
        let dir = tempfile::Builder::new().prefix(&format!("klar-unit-{tag}-")).tempdir().expect("create test dir");
        Self(dir)
    }

    pub(crate) fn path(&self) -> &Path {
        self.0.path()
    }

    pub(crate) fn file(&self, name: &str) -> PathBuf {
        self.0.path().join(name)
    }

    /// Names in the directory, sorted.
    pub(crate) fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(self.0.path())
            .map(|entries| entries.flatten().map(|entry| entry.file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        names.sort();
        names
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_is_gone_after_drop() {
        let dir = TestDir::new("drop");
        let path = dir.path().to_path_buf();
        std::fs::write(dir.file("a.txt"), "x").unwrap();
        assert_eq!(dir.names(), ["a.txt"]);
        drop(dir);
        assert!(!path.exists());
    }

    #[test]
    fn directory_is_gone_after_a_panic() {
        let (sender, receiver) = std::sync::mpsc::channel();
        let outcome = std::thread::spawn(move || {
            let dir = TestDir::new("panic");
            sender.send(dir.path().to_path_buf()).unwrap();
            panic!("a failing test");
        })
        .join();
        assert!(outcome.is_err());
        assert!(!receiver.recv().unwrap().exists());
    }

    #[test]
    fn two_directories_with_one_tag_do_not_collide() {
        let (first, second) = (TestDir::new("same"), TestDir::new("same"));
        assert_ne!(first.path(), second.path());
    }
}
