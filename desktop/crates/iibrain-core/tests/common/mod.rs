//! A Brain in a temporary directory and a Device beside it, as the apps
//! would have them.

// Helpers of tests may unwrap, expect, panic and index, as tests may.
#![allow(
    dead_code,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::fs;
use std::path::{Path, PathBuf};

use iibrain_core::Core;
use tempfile::TempDir;

pub type TestResult = Result<(), Box<dyn std::error::Error>>;

pub struct World {
    pub dir: TempDir,
}

impl World {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("brain")).unwrap();
        Self { dir }
    }

    /// The Brain's folder.
    pub fn brain(&self) -> PathBuf {
        self.dir.path().join("brain")
    }

    /// The Device's own directory.
    pub fn device(&self) -> PathBuf {
        self.dir.path().join("device")
    }

    /// A second Device on the same folder.
    pub fn other_device(&self) -> PathBuf {
        self.dir.path().join("device-2")
    }

    pub fn core(&self) -> Core {
        Core::open_folder(&self.device(), &self.brain()).unwrap()
    }

    /// Writes a file into the Brain as another tool would.
    pub fn write(&self, relative: &str, content: &str) {
        let path = self.brain().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    pub fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.brain().join(relative)).unwrap()
    }

    pub fn exists(&self, relative: &str) -> bool {
        self.brain().join(relative).exists()
    }

    /// Every file of the Brain, relative, sorted.
    pub fn files(&self) -> Vec<String> {
        let mut out = Vec::new();
        walk(&self.brain(), &self.brain(), &mut out);
        out.sort();
        out
    }
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(root, &path, out);
        } else {
            out.push(
                path.strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
}
