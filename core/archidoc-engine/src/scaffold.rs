use std::fs;
use std::path::{Path, PathBuf};

/// Directories to skip during the scaffold walk.
///
/// Matches the archidoc-md walker's SKIP_DIRS, plus:
/// - `_archive`  — deprecated content, should not be stubbed
/// - `_context`  — auto-generated output directory
const SKIP_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    ".ragd",
    ".vite",
    ".claude",
    "_archive",
    "_context",
];

/// Walk `root` and return sorted paths of every directory lacking an `_index.md`.
///
/// The root directory itself is always excluded — it has no meaningful module_path
/// and is skipped by the archidoc-md walker by design.
pub fn find_missing(root: &Path) -> Vec<PathBuf> {
    let mut missing = Vec::new();
    walk(root, root, &mut missing);
    missing.sort();
    missing
}

fn walk(root: &Path, dir: &Path, missing: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    // Collect subdirectories first so we can recurse after reading entries.
    let mut subdirs: Vec<PathBuf> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if SKIP_DIRS.contains(&name_str.as_ref()) {
            continue;
        }
        subdirs.push(path);
    }

    for subdir in subdirs {
        // Check if this subdir is missing _index.md — but skip root itself.
        if subdir != root && !subdir.join("_index.md").exists() {
            missing.push(subdir.clone());
        }
        walk(root, &subdir, missing);
    }
}

/// Generate a stub `_index.md` for the given directory.
///
/// The stub intentionally omits the `<!-- @c4 ... -->` annotation. The
/// archidoc-md walker only picks up annotated files, so stubs remain invisible
/// to the architecture doc until a human or agent fills in the description and
/// adds the annotation. The `TODO: archidoc` marker is grep-findable:
///
///   grep -rl "TODO: archidoc" . --include="_index.md"
///
pub fn scaffold_stub(dir: &Path) -> String {
    let dir_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let md_files = scan_md_files(dir);

    let mut out = String::new();
    out.push_str("<!-- TODO: archidoc — add @c4 annotation and description when filled -->\n");
    out.push('\n');
    out.push_str(&format!("# {}\n", dir_name));
    out.push('\n');
    out.push_str("TODO: archidoc — describe this directory's responsibility.\n");
    out.push('\n');
    out.push_str("| File | Pattern | Purpose | Health |\n");
    out.push_str("|------|---------|---------|--------|\n");

    for file in &md_files {
        out.push_str(&format!("| `{}` | -- | TODO: archidoc | planned |\n", file));
    }

    out
}

/// Scan a directory for `.md` files (excluding `_index.md` itself).
fn scan_md_files(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut files: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() {
                return None;
            }
            let name = path.file_name()?.to_string_lossy().to_string();
            if name == "_index.md" || !name.ends_with(".md") {
                return None;
            }
            Some(name)
        })
        .collect();

    files.sort();
    files
}

/// Result of a scaffold run.
pub struct ScaffoldReport {
    pub created: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
    pub errors: Vec<(PathBuf, String)>,
}

/// Write stub `_index.md` files for every directory under `root` that is missing one.
///
/// Idempotent — existing files are never overwritten. Returns a report of what
/// was created, skipped, or failed.
pub fn write_stubs(root: &Path) -> ScaffoldReport {
    let missing = find_missing(root);
    let mut report = ScaffoldReport {
        created: Vec::new(),
        skipped: Vec::new(),
        errors: Vec::new(),
    };

    for dir in missing {
        let index_path = dir.join("_index.md");
        if index_path.exists() {
            // Race condition guard — find_missing already checked, but be safe.
            report.skipped.push(dir);
            continue;
        }
        let content = scaffold_stub(&dir);
        match fs::write(&index_path, &content) {
            Ok(_) => report.created.push(dir),
            Err(e) => report.errors.push((dir, e.to_string())),
        }
    }

    report
}
