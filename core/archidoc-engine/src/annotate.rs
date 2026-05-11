use std::fs;
use std::path::{Path, PathBuf};

/// Directories skipped during recursive annotation walks.
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

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Rust,
    TypeScript,
    JavaScript,
    Markdown,
}

impl Lang {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rs" | "rust" => Some(Self::Rust),
            "ts" | "typescript" => Some(Self::TypeScript),
            "js" | "javascript" => Some(Self::JavaScript),
            "md" | "markdown" => Some(Self::Markdown),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum SkipReason {
    /// Entry file already contains an `@c4` annotation.
    AlreadyAnnotated,
    /// Entry file exists but has no `@c4` annotation — pass `--force` to prepend.
    FileExistsNoForce,
}

#[derive(Debug)]
pub enum Outcome {
    Created(PathBuf),
    Prepended(PathBuf),
    Skipped { path: PathBuf, reason: SkipReason },
    Error { path: PathBuf, error: String },
}

#[derive(Debug)]
pub struct DryRunItem {
    pub path: PathBuf,
    pub action: &'static str,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Annotate a single directory for the given language.
pub fn annotate_dir(dir: &Path, lang: Lang, force: bool) -> Outcome {
    if !dir.exists() || !dir.is_dir() {
        return Outcome::Error {
            path: dir.to_path_buf(),
            error: "path does not exist or is not a directory".to_string(),
        };
    }
    match lang {
        Lang::Markdown => annotate_md(dir, force),
        _ => annotate_code(dir, lang, force),
    }
}

/// Annotate a directory and all non-skipped subdirectories.
///
/// If `max_depth` is `Some(n)`, only descend `n` levels below `root`.
pub fn annotate_recursive(root: &Path, lang: Lang, force: bool, max_depth: Option<usize>) -> Vec<Outcome> {
    let mut results = Vec::new();
    results.push(annotate_dir(root, lang, force));
    walk_and_annotate(root, lang, force, &mut results, 1, max_depth);
    results
}

/// Preview what `annotate_dir` / `annotate_recursive` would do without writing.
///
/// If `max_depth` is `Some(n)`, only descend `n` levels below `root`.
pub fn dry_run(root: &Path, lang: Lang, recursive: bool, max_depth: Option<usize>) -> Vec<DryRunItem> {
    let dirs = if recursive {
        let mut all = vec![root.to_path_buf()];
        collect_subdirs(root, &mut all, 1, max_depth);
        all
    } else {
        vec![root.to_path_buf()]
    };

    dirs.iter()
        .filter_map(|dir| dry_run_dir(dir, lang))
        .collect()
}

// ---------------------------------------------------------------------------
// Internals — walking
// ---------------------------------------------------------------------------

fn walk_and_annotate(
    dir: &Path,
    lang: Lang,
    force: bool,
    results: &mut Vec<Outcome>,
    depth: usize,
    max_depth: Option<usize>,
) {
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }

    let Ok(entries) = fs::read_dir(dir) else { return };

    let mut subdirs: Vec<PathBuf> = entries
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
            Some(path)
        })
        .collect();

    subdirs.sort();

    for subdir in subdirs {
        results.push(annotate_dir(&subdir, lang, force));
        walk_and_annotate(&subdir, lang, force, results, depth + 1, max_depth);
    }
}

fn collect_subdirs(dir: &Path, all: &mut Vec<PathBuf>, depth: usize, max_depth: Option<usize>) {
    if let Some(max) = max_depth {
        if depth > max {
            return;
        }
    }

    let Ok(entries) = fs::read_dir(dir) else { return };

    let mut subdirs: Vec<PathBuf> = entries
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
            Some(path)
        })
        .collect();

    subdirs.sort();

    for subdir in subdirs {
        all.push(subdir.clone());
        collect_subdirs(&subdir, all, depth + 1, max_depth);
    }
}

fn dry_run_dir(dir: &Path, lang: Lang) -> Option<DryRunItem> {
    if !dir.exists() || !dir.is_dir() {
        return None;
    }
    let path = entry_file_path(dir, lang);
    let action = match lang {
        Lang::Markdown => {
            if path.exists() { "skip (exists)" } else { "create" }
        }
        _ => {
            if !path.exists() {
                "create"
            } else if has_c4_annotation(&path) {
                "skip (already annotated)"
            } else {
                "prepend (requires --force)"
            }
        }
    };
    Some(DryRunItem { path, action })
}

// ---------------------------------------------------------------------------
// Language-specific annotation logic
// ---------------------------------------------------------------------------

fn annotate_md(dir: &Path, force: bool) -> Outcome {
    let index_path = dir.join("_index.md");

    if index_path.exists() && !force {
        return Outcome::Skipped {
            path: index_path,
            reason: SkipReason::FileExistsNoForce,
        };
    }

    let content = generate_md_annotation(dir);
    match fs::write(&index_path, content) {
        Ok(_) => Outcome::Created(index_path),
        Err(e) => Outcome::Error { path: index_path, error: e.to_string() },
    }
}

fn annotate_code(dir: &Path, lang: Lang, force: bool) -> Outcome {
    let entry_path = entry_file_path(dir, lang);

    if !entry_path.exists() {
        let content = generate_code_annotation(dir, lang);
        return match fs::write(&entry_path, content) {
            Ok(_) => Outcome::Created(entry_path),
            Err(e) => Outcome::Error { path: entry_path, error: e.to_string() },
        };
    }

    if has_c4_annotation(&entry_path) {
        return Outcome::Skipped {
            path: entry_path,
            reason: SkipReason::AlreadyAnnotated,
        };
    }

    if !force {
        return Outcome::Skipped {
            path: entry_path,
            reason: SkipReason::FileExistsNoForce,
        };
    }

    // Prepend annotation block to existing file.
    let existing = match fs::read_to_string(&entry_path) {
        Ok(s) => s,
        Err(e) => return Outcome::Error { path: entry_path, error: e.to_string() },
    };
    let annotation = generate_code_annotation(dir, lang);
    let new_content = format!("{}\n{}", annotation, existing);
    match fs::write(&entry_path, new_content) {
        Ok(_) => Outcome::Prepended(entry_path),
        Err(e) => Outcome::Error { path: entry_path, error: e.to_string() },
    }
}

// ---------------------------------------------------------------------------
// Entry file resolution
// ---------------------------------------------------------------------------

fn entry_file_path(dir: &Path, lang: Lang) -> PathBuf {
    match lang {
        Lang::Rust => {
            let lib = dir.join("lib.rs");
            let modrs = dir.join("mod.rs");
            if lib.exists() {
                lib
            } else if modrs.exists() {
                modrs
            } else {
                // Neither exists — pick based on whether this is a crate root.
                if dir.join("Cargo.toml").exists() { lib } else { dir.join("mod.rs") }
            }
        }
        Lang::TypeScript => dir.join("index.ts"),
        Lang::JavaScript => dir.join("index.js"),
        Lang::Markdown => dir.join("_index.md"),
    }
}

fn has_c4_annotation(path: &Path) -> bool {
    fs::read_to_string(path)
        .map(|s| s.contains("@c4"))
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// Template generation
// ---------------------------------------------------------------------------

fn generate_code_annotation(dir: &Path, lang: Lang) -> String {
    let title = dir_title(dir);
    let c4_level = infer_c4_level(dir);
    let files = scan_source_files(dir, lang);

    match lang {
        Lang::Rust => {
            let mut out = String::new();
            out.push_str(&format!("//! @c4 {}\n", c4_level));
            out.push_str("//!\n");
            out.push_str(&format!("//! # {}\n", title));
            out.push_str("//!\n");
            out.push_str("//! TODO — describe this module's responsibility.\n");
            if !files.is_empty() {
                out.push_str("//!\n");
                out.push_str("//! | File | Pattern | Purpose | Health |\n");
                out.push_str("//! |------|---------|---------|--------|\n");
                for f in &files {
                    out.push_str(&format!("//! | `{}` | -- | TODO | planned |\n", f));
                }
            }
            out
        }
        Lang::TypeScript | Lang::JavaScript => {
            let mut body = String::new();
            body.push_str(&format!(" * @c4 {}\n", c4_level));
            body.push_str(" *\n");
            body.push_str(&format!(" * # {}\n", title));
            body.push_str(" *\n");
            body.push_str(" * TODO — describe this module's responsibility.\n");
            if !files.is_empty() {
                body.push_str(" *\n");
                body.push_str(" * | File | Pattern | Purpose | Health |\n");
                body.push_str(" * |------|---------|---------|--------|\n");
                for f in &files {
                    body.push_str(&format!(" * | `{}` | -- | TODO | planned |\n", f));
                }
            }
            format!("/**\n{}*/\n", body)
        }
        Lang::Markdown => unreachable!(),
    }
}

fn generate_md_annotation(dir: &Path) -> String {
    let title = dir_title(dir);
    let c4_level = infer_c4_level(dir);
    let subdirs = scan_subdirs(dir);
    let md_files = scan_md_files(dir);

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("c4: {}\n", c4_level));
    out.push_str(&format!("title: {}\n", title));
    out.push_str("description: TODO — describe this directory's purpose\n");
    out.push_str("---\n");
    out.push('\n');
    out.push_str("| File | Pattern | Purpose | Health |\n");
    out.push_str("|------|---------|---------|--------|\n");
    for subdir in &subdirs {
        out.push_str(&format!("| `{}/` | -- | TODO | planned |\n", subdir));
    }
    for file in &md_files {
        out.push_str(&format!("| `{}` | -- | TODO | planned |\n", file));
    }
    out
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn dir_title(dir: &Path) -> String {
    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("module");
    let with_spaces = name.replace('_', " ").replace('-', " ");
    let mut chars = with_spaces.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn infer_c4_level(dir: &Path) -> &'static str {
    let components: Vec<_> = dir.components().collect();
    let src_pos = components.iter().position(|c| {
        let s = c.as_os_str().to_string_lossy();
        s == "src" || s == "lib" || s == "source"
    });
    match src_pos {
        Some(pos) if components.len() - pos - 1 == 1 => "container",
        Some(_) => "component",
        None => "container",
    }
}

fn scan_source_files(dir: &Path, lang: Lang) -> Vec<String> {
    let (extensions, entry_files): (&[&str], &[&str]) = match lang {
        Lang::Rust => (&[".rs"], &["mod.rs", "lib.rs", "main.rs"]),
        Lang::TypeScript => (&[".ts"], &["index.ts"]),
        Lang::JavaScript => (&[".js"], &["index.js"]),
        Lang::Markdown => (&[".md"], &["_index.md"]),
    };

    let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };

    let mut files: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() {
                return None;
            }
            let name = path.file_name()?.to_string_lossy().to_string();
            if entry_files.contains(&name.as_str()) {
                return None;
            }
            if extensions.iter().any(|ext| name.ends_with(ext)) {
                Some(name)
            } else {
                None
            }
        })
        .collect();

    files.sort();
    files
}

fn scan_md_files(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };

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

fn scan_subdirs(dir: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn lang_from_str_round_trips() {
        assert_eq!(Lang::from_str("rs"), Some(Lang::Rust));
        assert_eq!(Lang::from_str("rust"), Some(Lang::Rust));
        assert_eq!(Lang::from_str("ts"), Some(Lang::TypeScript));
        assert_eq!(Lang::from_str("typescript"), Some(Lang::TypeScript));
        assert_eq!(Lang::from_str("js"), Some(Lang::JavaScript));
        assert_eq!(Lang::from_str("md"), Some(Lang::Markdown));
        assert_eq!(Lang::from_str("python"), None);
    }

    #[test]
    fn annotate_rs_creates_lib_rs() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("src").join("auth");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("handler.rs"), "").unwrap();

        let outcome = annotate_dir(&dir, Lang::Rust, false);
        assert!(matches!(outcome, Outcome::Created(_)));

        let content = fs::read_to_string(dir.join("mod.rs")).unwrap();
        assert!(content.contains("@c4"));
        assert!(content.contains("handler.rs"));
    }

    #[test]
    fn annotate_rs_skips_if_already_annotated() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("lib.rs"), "//! @c4 container\n").unwrap();

        let outcome = annotate_dir(tmp.path(), Lang::Rust, false);
        assert!(matches!(outcome, Outcome::Skipped { reason: SkipReason::AlreadyAnnotated, .. }));
    }

    #[test]
    fn annotate_rs_skips_existing_without_force() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("lib.rs"), "// existing code\n").unwrap();

        let outcome = annotate_dir(tmp.path(), Lang::Rust, false);
        assert!(matches!(outcome, Outcome::Skipped { reason: SkipReason::FileExistsNoForce, .. }));
    }

    #[test]
    fn annotate_rs_prepends_with_force() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("lib.rs"), "// existing code\n").unwrap();

        let outcome = annotate_dir(tmp.path(), Lang::Rust, true);
        assert!(matches!(outcome, Outcome::Prepended(_)));

        let content = fs::read_to_string(tmp.path().join("lib.rs")).unwrap();
        assert!(content.contains("@c4"));
        assert!(content.contains("// existing code"));
    }

    #[test]
    fn annotate_md_creates_index() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("design.md"), "").unwrap();

        let outcome = annotate_dir(tmp.path(), Lang::Markdown, false);
        assert!(matches!(outcome, Outcome::Created(_)));

        let content = fs::read_to_string(tmp.path().join("_index.md")).unwrap();
        assert!(content.contains("c4:"));
        assert!(content.contains("title:"));
        assert!(content.contains("design.md"));
    }

    #[test]
    fn annotate_md_skips_existing_without_force() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("_index.md"), "existing\n").unwrap();

        let outcome = annotate_dir(tmp.path(), Lang::Markdown, false);
        assert!(matches!(outcome, Outcome::Skipped { reason: SkipReason::FileExistsNoForce, .. }));
    }

    #[test]
    fn annotate_md_overwrites_with_force() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("_index.md"), "old content\n").unwrap();

        let outcome = annotate_dir(tmp.path(), Lang::Markdown, true);
        assert!(matches!(outcome, Outcome::Created(_)));

        let content = fs::read_to_string(tmp.path().join("_index.md")).unwrap();
        assert!(content.contains("c4:"));
        assert!(!content.contains("old content"));
    }

    #[test]
    fn dry_run_shows_correct_actions() {
        let tmp = TempDir::new().unwrap();
        let sub = tmp.path().join("sub");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("lib.rs"), "//! @c4 container\n").unwrap();

        let items = dry_run(tmp.path(), Lang::Rust, true, None);
        let sub_item = items.iter().find(|i| i.path.parent() == Some(&sub)).unwrap();
        assert_eq!(sub_item.action, "skip (already annotated)");
    }

    #[test]
    fn ts_annotation_uses_jsdoc_syntax() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("service.ts"), "").unwrap();

        annotate_dir(tmp.path(), Lang::TypeScript, false);
        let content = fs::read_to_string(tmp.path().join("index.ts")).unwrap();
        assert!(content.starts_with("/**"));
        assert!(content.contains("@c4"));
    }

    #[test]
    fn md_annotation_uses_frontmatter() {
        let tmp = TempDir::new().unwrap();
        let outcome = annotate_dir(tmp.path(), Lang::Markdown, false);
        assert!(matches!(outcome, Outcome::Created(_)));
        let content = fs::read_to_string(tmp.path().join("_index.md")).unwrap();
        assert!(content.starts_with("---\n"));
        assert!(content.contains("c4:"));
        assert!(content.contains("title:"));
        assert!(content.contains("description:"));
    }
}
