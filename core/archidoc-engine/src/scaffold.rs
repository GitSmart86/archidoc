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

/// Try to load a custom `_index.md` template from `.archidoc/custom/_index.md`
/// relative to the project root. Returns `None` if the file does not exist.
fn load_custom_template(root: &Path) -> Option<String> {
    let custom_path = root.join(".archidoc").join("custom").join("_index.md");
    fs::read_to_string(custom_path).ok()
}

/// Count columns in a markdown table header line.
fn count_table_columns(header: &str) -> usize {
    header.split('|').filter(|s| !s.trim().is_empty()).count()
}

/// Find the table header line in a template and return its column count.
fn template_column_count(template: &str) -> usize {
    for line in template.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && (trimmed.contains("File") || trimmed.contains("Name")) {
            return count_table_columns(trimmed);
        }
    }
    0
}

/// Format a file row matching the column count from the template header.
///
/// Column layout: File | (middle columns as [TODO]) | Health (second-to-last) | Notes (last, empty)
/// Falls back to the default 4-column format if column count is unknown.
fn format_file_row(filename: &str, col_count: usize) -> String {
    if col_count <= 4 {
        return format!("| `{}` | -- | TODO: archidoc | planned |", filename);
    }

    let mut cells = vec![format!("`{}`", filename)];
    for i in 1..col_count {
        if i == col_count - 2 {
            cells.push("planned".to_string());        // Health column
        } else if i == col_count - 1 {
            cells.push(String::new());                 // Notes column (empty)
        } else {
            cells.push("[TODO]".to_string());          // Purpose/Content/Takeaway
        }
    }
    format!("| {} |", cells.join(" | "))
}

/// Infer C4 level from directory depth relative to the root.
/// Depth 1 from root = container, deeper = component.
fn infer_c4_level(dir: &Path, root: &Path) -> &'static str {
    let depth = dir
        .strip_prefix(root)
        .map(|rel| rel.components().count())
        .unwrap_or(1);
    if depth <= 1 { "container" } else { "component" }
}

/// Generate `_index.md` content for the given directory.
///
/// If a custom template exists at `.archidoc/custom/_index.md`, uses it with
/// `{{c4_level}}` and `{{file_rows}}` substitution. Otherwise falls back to
/// the built-in stub format.
pub fn scaffold_stub(dir: &Path, root: &Path) -> String {
    let custom = load_custom_template(root);

    let md_files = scan_md_files(dir);
    let subdirs = scan_subdirs(dir);

    if let Some(template) = custom {
        let c4_level = infer_c4_level(dir, root);
        let col_count = template_column_count(&template);

        let mut rows: Vec<String> = Vec::new();
        for subdir in &subdirs {
            rows.push(format_file_row(&format!("{}/", subdir), col_count));
        }
        for file in &md_files {
            rows.push(format_file_row(file, col_count));
        }

        let file_rows = rows.join("\n");

        template
            .replace("{{c4_level}}", c4_level)
            .replace("{{file_rows}}", &file_rows)
    } else {
        // Built-in fallback (no custom template)
        let dir_name = dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let mut out = String::new();
        out.push_str(
            "<!-- TODO: archidoc — add @c4 annotation and description when filled -->\n",
        );
        out.push('\n');
        out.push_str(&format!("# {}\n", dir_name));
        out.push('\n');
        out.push_str("TODO: archidoc — describe this directory's responsibility.\n");
        out.push('\n');
        out.push_str("| File | Pattern | Purpose | Health |\n");
        out.push_str("|------|---------|---------|--------|\n");

        for file in &md_files {
            out.push_str(&format!(
                "| `{}` | -- | TODO: archidoc | planned |\n",
                file
            ));
        }

        out
    }
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

/// Scan a directory for subdirectories (excluding SKIP_DIRS and hidden dirs).
fn scan_subdirs(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut dirs: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_dir() {
                return None;
            }
            let name = e.file_name().to_string_lossy().to_string();
            if SKIP_DIRS.contains(&name.as_str()) || name.starts_with('.') {
                return None;
            }
            Some(name)
        })
        .collect();

    dirs.sort();
    dirs
}

/// Result of a scaffold run.
pub struct ScaffoldReport {
    pub created: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
    pub errors: Vec<(PathBuf, String)>,
}

/// Write `_index.md` files for every directory under `root` that is missing one.
///
/// If a custom template exists at `.archidoc/custom/_index.md`, uses it.
/// Otherwise falls back to the built-in stub format.
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
        let content = scaffold_stub(&dir, root);
        match fs::write(&index_path, &content) {
            Ok(_) => report.created.push(dir),
            Err(e) => report.errors.push((dir, e.to_string())),
        }
    }

    report
}
