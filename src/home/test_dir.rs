//! A scratch directory for unit tests. It is removed when dropped, also when the test panics.
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub(crate) struct TestDir(PathBuf);

impl TestDir {
    pub(crate) fn new(tag: &str) -> Self {
        let path = std::env::temp_dir().join(format!("klar-unit-{tag}-{}", Uuid::new_v4().simple()));
        std::fs::create_dir_all(&path).expect("create test dir");
        Self(path)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }

    pub(crate) fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    /// Names in the directory, sorted.
    pub(crate) fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(&self.0)
            .map(|entries| entries.flatten().map(|entry| entry.file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        names.sort();
        names
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
