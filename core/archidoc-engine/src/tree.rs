use std::collections::HashMap;
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

/// File extensions to include. Skips build artifacts, lock files, binaries.
const INCLUDE_EXTENSIONS: &[&str] = &[
    ".md", ".rs", ".ts", ".js", ".toml", ".yaml", ".yml", ".json", ".py",
    ".sh", ".ps1", ".drawio", ".csv",
];

/// Files to always skip regardless of extension.
const SKIP_FILES: &[&str] = &[
    "package-lock.json",
    "yarn.lock",
    "Cargo.lock",
    ".DS_Store",
];

/// Dirs with more than this many files get a count summary instead of inline listing.
const INLINE_THRESHOLD: usize = 6;

// ── Public API ────────────────────────────────────────────────────────────────

/// Generate a compact dirs-only tree.
///
/// Format: one full relative path per line, no indentation, trailing `/`.
/// Grep-friendly — every line is self-contained.
///
/// ```text
/// 0_White/
/// 0_White/Framework/
/// 0_White/Framework/-1_worldview/
/// ```
pub fn compact_dirs_tree(root: &Path, max_depth: Option<usize>) -> String {
    let mut out = String::new();
    walk_dirs(&mut out, root, root, 0, max_depth);
    out
}

/// Generate a compact dirs+files tree.
///
/// Format:
/// - Root files on the first line: `[root] file1, file2`
/// - Each dir on its own line with an adaptive file listing:
///   - ≤6 files  → inline: `path/ {file1, file2, file3}`
///   - >6 files  → count:  `path/ [12 files: 10.md 2.rs]`
///   - No files  → just the path
///
/// ```text
/// [root] CLAUDE.md, _index.md
/// 0_White/ {9+2.md, _index.md}
/// 0_White/Framework/ [10 files: 10.md]
/// 0_White/Framework/-1_worldview/ {README.md, _index.md, claims.md, ...}
/// ```
pub fn compact_files_tree(root: &Path, max_depth: Option<usize>) -> String {
    let mut out = String::new();

    // Root-level files on the first line
    let root_files = collect_files(root);
    if !root_files.is_empty() {
        out.push_str(&format!("[root] {}\n", root_files.join(", ")));
    }

    walk_files(&mut out, root, root, 0, max_depth);
    out
}

// ── Internal walkers ──────────────────────────────────────────────────────────

fn walk_dirs(out: &mut String, root: &Path, dir: &Path, depth: usize, max_depth: Option<usize>) {
    if let Some(max) = max_depth {
        if depth >= max {
            return;
        }
    }

    let mut subdirs = read_subdirs(dir);
    subdirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for subdir in subdirs {
        let rel = subdir.strip_prefix(root).unwrap_or(&subdir);
        // Normalise to forward slashes for cross-platform output
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        out.push_str(&format!("{}/\n", rel_str));
        walk_dirs(out, root, &subdir, depth + 1, max_depth);
    }
}

fn walk_files(out: &mut String, root: &Path, dir: &Path, depth: usize, max_depth: Option<usize>) {
    if let Some(max) = max_depth {
        if depth >= max {
            return;
        }
    }

    let mut subdirs = read_subdirs(dir);
    subdirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for subdir in subdirs {
        let rel = subdir.strip_prefix(root).unwrap_or(&subdir);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let files = collect_files(&subdir);

        let suffix = format_file_suffix(&files);
        if suffix.is_empty() {
            out.push_str(&format!("{}/\n", rel_str));
        } else {
            out.push_str(&format!("{}/{}\n", rel_str, suffix));
        }

        walk_files(out, root, &subdir, depth + 1, max_depth);
    }
}

// ── File collection and formatting ────────────────────────────────────────────

fn collect_files(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };

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
            let included = INCLUDE_EXTENSIONS.iter().any(|ext| name.ends_with(ext));
            if included { Some(name) } else { None }
        })
        .collect();

    files.sort();
    files
}

/// Format the file suffix for a directory line.
///
/// - Empty file list  → empty string (no suffix)
/// - ≤ INLINE_THRESHOLD → ` {file1, file2, ...}`
/// - > INLINE_THRESHOLD → ` [N files: Xmd Yrs ...]`
fn format_file_suffix(files: &[String]) -> String {
    if files.is_empty() {
        return String::new();
    }
    if files.len() <= INLINE_THRESHOLD {
        return format!(" {{{}}}", files.join(", "));
    }
    // Count by extension
    let mut ext_counts: HashMap<String, usize> = HashMap::new();
    for file in files {
        let ext = std::path::Path::new(file)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("other")
            .to_string();
        *ext_counts.entry(ext).or_default() += 1;
    }
    // Sort by count descending, then alphabetically
    let mut ext_list: Vec<(String, usize)> = ext_counts.into_iter().collect();
    ext_list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let breakdown: Vec<String> = ext_list
        .iter()
        .map(|(ext, count)| format!("{}.{}", count, ext))
        .collect();
    format!(" [{} files: {}]", files.len(), breakdown.join(" "))
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
