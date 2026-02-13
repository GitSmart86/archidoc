use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Creates a temporary directory tree with annotated Rust source files.
///
/// Converts module paths (dot notation) to directory structures:
/// - `"bus"` -> `bus/mod.rs`
/// - `"bus.calc"` -> `bus/calc/mod.rs`
/// - `"_lib"` -> `lib.rs` (crate root)
pub struct FakeSourceTree {
    temp_dir: TempDir,
}

impl FakeSourceTree {
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().expect("failed to create temp dir"),
        }
    }

    /// Get the root path of the fake source tree.
    pub fn root(&self) -> &Path {
        self.temp_dir.path()
    }

    /// Create an annotated source file at the path derived from module_path.
    ///
    /// The content should be raw annotation text (without `//!` prefixes).
    /// This method wraps each line with `//!` to create valid Rust doc comments.
    pub fn create_module(&self, module_path: &str, content: &str) {
        let file_path = self.module_path_to_file(module_path);

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("failed to create module directory");
        }

        // Wrap content in //! doc comments
        let doc_content = content
            .lines()
            .map(|line| {
                if line.is_empty() {
                    "//!".to_string()
                } else {
                    format!("//! {}", line)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&file_path, doc_content).expect("failed to write module file");
    }

    /// Place an extra .rs file on disk in an element's directory (for orphan tests).
    pub fn place_extra_file(&self, module_path: &str, filename: &str) {
        let dir = self.module_dir(module_path);
        fs::create_dir_all(&dir).expect("failed to create module directory");
        let file_path = dir.join(filename);
        fs::write(&file_path, "// placeholder file for testing\n")
            .expect("failed to write extra file");
    }

    /// Create a Rust source file with raw code (no `//!` wrapping) in a module's directory.
    ///
    /// Used for pattern heuristic tests where actual Rust code is needed.
    pub fn create_code_file(&self, module_path: &str, filename: &str, code: &str) {
        let dir = self.module_dir(module_path);
        fs::create_dir_all(&dir).expect("failed to create module directory");
        let file_path = dir.join(filename);
        fs::write(&file_path, code).expect("failed to write code file");
    }

    /// Remove a file from disk in an element's directory (for ghost tests).
    pub fn remove_file(&self, module_path: &str, filename: &str) {
        let file_path = self.module_dir(module_path).join(filename);
        if file_path.exists() {
            fs::remove_file(&file_path).expect("failed to remove file");
        }
    }

    /// Get the directory for a module path.
    pub fn module_dir(&self, module_path: &str) -> PathBuf {
        let root = self.temp_dir.path();
        let parts: Vec<&str> = module_path.split('.').collect();
        let mut path = root.to_path_buf();
        path.push("src"); // Add src/ to match real Rust project structure
        for part in &parts {
            path.push(part);
        }
        path
    }

    /// Convert a dot-notation module path to a file path.
    fn module_path_to_file(&self, module_path: &str) -> PathBuf {
        let root = self.temp_dir.path();

        if module_path == "_lib" {
            return root.join("src").join("lib.rs");
        }

        let parts: Vec<&str> = module_path.split('.').collect();
        let mut path = root.to_path_buf();
        path.push("src"); // Add src/ to match real Rust project structure
        for part in &parts {
            path.push(part);
        }
        path.push("mod.rs");
        path
    }
}
