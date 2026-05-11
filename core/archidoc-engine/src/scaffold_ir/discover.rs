use std::collections::HashSet;
use std::path::{Path, PathBuf};

use archidoc_types::scaffold_ir::ScaffoldIR;

/// Find the first `.archidoc/scaffolds/<name>.json` file walking up from `start`.
///
/// Returns the path to the JSON file if found (nearest wins), or `None`.
pub fn find(name: &str, start: &Path) -> Option<PathBuf> {
    let filename = format!("{}.json", name);
    let mut dir = start.to_path_buf();

    loop {
        let candidate = dir.join(".archidoc").join("scaffolds").join(&filename);
        if candidate.exists() {
            return Some(candidate);
        }

        match dir.parent() {
            Some(parent) if parent != dir => dir = parent.to_path_buf(),
            _ => return None,
        }
    }
}

/// Collect all ScaffoldIR templates from `.archidoc/scaffolds/` walking up from `start`.
///
/// Returns `(name, path, ir)` tuples, nearest wins on name collision.
pub fn list(start: &Path) -> Vec<(String, PathBuf, ScaffoldIR)> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result: Vec<(String, PathBuf, ScaffoldIR)> = Vec::new();
    let mut dir = start.to_path_buf();

    loop {
        let scaffolds_dir = dir.join(".archidoc").join("scaffolds");
        if scaffolds_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&scaffolds_dir) {
                let mut files: Vec<PathBuf> = entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
                    .collect();
                files.sort();

                for path in files {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    if name.is_empty() || seen.contains(&name) {
                        continue;
                    }
                    if let Ok(ir) = ScaffoldIR::load(&path) {
                        seen.insert(name.clone());
                        result.push((name, path, ir));
                    }
                }
            }
        }

        match dir.parent() {
            Some(parent) if parent != dir => dir = parent.to_path_buf(),
            _ => break,
        }
    }

    result
}
