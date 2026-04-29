use std::fs;
use std::path::Path;

/// Directories to skip during tree walk — matches scaffold.rs SKIP_DIRS.
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

/// File extensions to include in the files variant.
/// Keeps the tree focused on documentation and source — skips build artifacts,
/// lock files, images, etc.
const INCLUDE_EXTENSIONS: &[&str] = &[
    ".md", ".rs", ".ts", ".js", ".toml", ".yaml", ".yml", ".json", ".py",
];

/// Files to always skip regardless of extension.
const SKIP_FILES: &[&str] = &[
    "package-lock.json",
    "yarn.lock",
    "Cargo.lock",
    ".DS_Store",
];

/// Generate a directory-only tree for `root`.
///
/// Produces indented markdown lines, one per directory, with a trailing `/`.
/// Respects SKIP_DIRS. The root directory itself is shown as the first line.
/// `max_depth` of `None` means unlimited.
pub fn dirs_tree(root: &Path, max_depth: Option<usize>) -> String {
    let root_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".");

    let mut out = String::new();
    out.push_str(&format!("{}/\n", root_name));
    walk_dirs(&mut out, root, 1, max_depth);
    out
}

/// Generate a directory + files tree for `root`.
///
/// Produces indented markdown lines. Directories get a trailing `/`.
/// Files are shown under their parent directory, filtered by INCLUDE_EXTENSIONS.
/// `max_depth` of `None` means unlimited.
pub fn files_tree(root: &Path, max_depth: Option<usize>) -> String {
    let root_name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(".");

    let mut out = String::new();
    out.push_str(&format!("{}/\n", root_name));
    walk_files(&mut out, root, 1, max_depth, true);
    out
}

// ── Internal walkers ──────────────────────────────────────────────────────────

fn walk_dirs(out: &mut String, dir: &Path, depth: usize, max_depth: Option<usize>) {
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }

    let mut subdirs = read_subdirs(dir);
    subdirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for subdir in subdirs {
        let name = subdir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        out.push_str(&format!("{}{}/\n", indent(depth), name));
        walk_dirs(out, &subdir, depth + 1, max_depth);
    }
}

fn walk_files(
    out: &mut String,
    dir: &Path,
    depth: usize,
    max_depth: Option<usize>,
    is_root_call: bool,
) {
    // Emit files in this directory first (unless this is the root call —
    // root-level files are listed before subdirs).
    if !is_root_call {
        emit_files(out, dir, depth);
    } else {
        emit_files(out, dir, depth);
    }

    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }

    let mut subdirs = read_subdirs(dir);
    subdirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for subdir in subdirs {
        let name = subdir.file_name().and_then(|n| n.to_str()).unwrap_or("");
        out.push_str(&format!("{}{}/\n", indent(depth), name));
        walk_files(out, &subdir, depth + 1, max_depth, false);
    }
}

fn emit_files(out: &mut String, dir: &Path, depth: usize) {
    let Ok(entries) = fs::read_dir(dir) else { return };

    let mut files: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() {
                return None;
            }
            let name = path.file_name()?.to_string_lossy().to_string();
            if SKIP_FILES.contains(&name.as_str()) {
                return None;
            }
            let has_ext = INCLUDE_EXTENSIONS.iter().any(|ext| name.ends_with(ext));
            if has_ext { Some(name) } else { None }
        })
        .collect();

    files.sort();

    for file in files {
        out.push_str(&format!("{}{}\n", indent(depth), file));
    }
}

fn read_subdirs(dir: &Path) -> Vec<std::path::PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };

    entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_dir() {
                return None;
            }
            let name = e.file_name();
            let name_str = name.to_string_lossy();
            if SKIP_DIRS.contains(&name_str.as_ref()) {
                return None;
            }
            Some(path)
        })
        .collect()
}

fn indent(depth: usize) -> String {
    "  ".repeat(depth)
}
