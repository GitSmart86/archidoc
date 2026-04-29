use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Built-in directories to skip. Extended (not replaced) by config.tree.json exclude_dirs.
const DEFAULT_SKIP_DIRS: &[&str] = &[
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

/// File extensions to include. Replaced by config.tree.json include_extensions if non-empty.
const DEFAULT_INCLUDE_EXTENSIONS: &[&str] = &[
    ".md", ".rs", ".ts", ".js", ".toml", ".yaml", ".yml", ".json", ".py",
    ".sh", ".ps1", ".drawio", ".csv",
];

/// Files to always skip. Extended by config.tree.json exclude_files.
const DEFAULT_SKIP_FILES: &[&str] = &[
    "package-lock.json",
    "yarn.lock",
    "Cargo.lock",
    ".DS_Store",
];

/// Dirs with more than this many files get a count summary instead of inline listing.
const DEFAULT_INLINE_THRESHOLD: usize = 6;

// ── Config ────────────────────────────────────────────────────────────────────

/// Runtime configuration for tree generation, resolved from built-in defaults
/// merged with `.archidoc/config.tree.json` at the project root.
pub struct TreeConfig {
    /// Merged exclusion set: DEFAULT_SKIP_DIRS + user additions.
    pub exclude_dirs: Vec<String>,
    /// Merged exclusion set: DEFAULT_SKIP_FILES + user additions.
    pub exclude_files: Vec<String>,
    /// If config specifies extensions, replaces defaults. Otherwise uses DEFAULT_INCLUDE_EXTENSIONS.
    pub include_extensions: Vec<String>,
    /// Dirs with more files than this threshold get a count summary. Default: 6.
    pub inline_threshold: usize,
}

impl TreeConfig {
    /// Returns a config populated entirely from built-in defaults.
    pub fn defaults() -> Self {
        Self {
            exclude_dirs: DEFAULT_SKIP_DIRS.iter().map(|s| s.to_string()).collect(),
            exclude_files: DEFAULT_SKIP_FILES.iter().map(|s| s.to_string()).collect(),
            include_extensions: DEFAULT_INCLUDE_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
            inline_threshold: DEFAULT_INLINE_THRESHOLD,
        }
    }

    /// Load from `root/.archidoc/config.tree.json`, merging with built-in defaults.
    ///
    /// - Missing file → returns defaults silently.
    /// - Invalid JSON → prints a warning and returns defaults.
    /// - `exclude_dirs` / `exclude_files` → **additive** (built-ins are always included).
    /// - `include_extensions` → **replaces** defaults if the array is non-empty.
    /// - `inline_threshold` → **overrides** the default if present.
    pub fn load(root: &Path) -> Self {
        let config_path = root.join(".archidoc").join("config.tree.json");
        let mut config = Self::defaults();

        let Ok(content) = fs::read_to_string(&config_path) else {
            return config;
        };

        let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) else {
            eprintln!("warning: .archidoc/config.tree.json is not valid JSON — using defaults");
            return config;
        };

        // Additive: merge user exclude_dirs into the built-in set
        if let Some(arr) = value.get("exclude_dirs").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    let s = s.to_string();
                    if !config.exclude_dirs.contains(&s) {
                        config.exclude_dirs.push(s);
                    }
                }
            }
        }

        // Additive: merge user exclude_files into the built-in set
        if let Some(arr) = value.get("exclude_files").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    let s = s.to_string();
                    if !config.exclude_files.contains(&s) {
                        config.exclude_files.push(s);
                    }
                }
            }
        }

        // Replacing: non-empty include_extensions replaces defaults entirely
        if let Some(arr) = value.get("include_extensions").and_then(|v| v.as_array()) {
            if !arr.is_empty() {
                config.include_extensions = arr
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect();
            }
        }

        // Override: inline_threshold replaces the default
        if let Some(n) = value.get("inline_threshold").and_then(|v| v.as_u64()) {
            config.inline_threshold = n as usize;
        }

        config
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Generate a brace-expanded compact dirs-only tree.
///
/// Each line shows one branch directory and its immediate subdirectory children
/// in a `{child1, child2}` brace list. Leaf directories (no subdirs) never get
/// their own line — they appear only as entries in their parent's brace list.
/// This eliminates repeated path prefixes and collapses wide flat dirs to a
/// single line each.
///
/// ```text
/// ./ {0_White, 1_Yellow, 2_Blue, 3_Red, todo, tools}
/// 0_White/ {Framework, ICP, ICP-Workflows}
/// 0_White/Framework/ {-1_worldview, 0_terrain-thesis, 1_gestalt_methods}
/// ```
pub fn compact_dirs_tree(root: &Path, max_depth: Option<usize>, config: &TreeConfig) -> String {
    let mut out = String::new();
    brace_walk(&mut out, root, root, 0, max_depth, config);
    out
}

/// Generate a compact dirs+files tree.
///
/// Format:
/// - Root files on the first line: `[root] file1, file2`
/// - Each dir on its own line with an adaptive file listing:
///   - ≤ inline_threshold files → inline: `path/ {file1, file2, file3}`
///   - > inline_threshold files → count:  `path/ [12 files: 10.md 2.rs]`
///   - No files                 → just the path
pub fn compact_files_tree(root: &Path, max_depth: Option<usize>, config: &TreeConfig) -> String {
    let mut out = String::new();

    // Root-level files on the first line
    let root_files = collect_files(root, config);
    if !root_files.is_empty() {
        out.push_str(&format!("[root] {}\n", root_files.join(", ")));
    }

    walk_files(&mut out, root, root, 0, max_depth, config);
    out
}

// ── Internal walkers ──────────────────────────────────────────────────────────

fn brace_walk(
    out: &mut String,
    root: &Path,
    dir: &Path,
    depth: usize,
    max_depth: Option<usize>,
    config: &TreeConfig,
) {
    let mut subdirs = read_subdirs(dir, config);
    if subdirs.is_empty() {
        return; // leaf — appears only in parent's brace list
    }
    subdirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    // Relative path display — root itself shown as "."
    let rel = if dir == root {
        ".".to_string()
    } else {
        dir.strip_prefix(root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| dir.to_string_lossy().replace('\\', "/"))
    };

    // Child names without trailing slash — redundant inside braces
    let children: Vec<String> = subdirs
        .iter()
        .filter_map(|s| s.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .collect();

    out.push_str(&format!("{}/ {{{}}}\n", rel, children.join(", ")));

    // Respect depth limit — still emit the current line, just don't recurse
    if let Some(max) = max_depth {
        if depth >= max {
            return;
        }
    }

    for subdir in &subdirs {
        brace_walk(out, root, subdir, depth + 1, max_depth, config);
    }
}

fn walk_files(
    out: &mut String,
    root: &Path,
    dir: &Path,
    depth: usize,
    max_depth: Option<usize>,
    config: &TreeConfig,
) {
    if let Some(max) = max_depth {
        if depth >= max {
            return;
        }
    }

    let mut subdirs = read_subdirs(dir, config);
    subdirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for subdir in subdirs {
        let rel = subdir.strip_prefix(root).unwrap_or(&subdir);
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let files = collect_files(&subdir, config);

        let suffix = format_file_suffix(&files, config);
        if suffix.is_empty() {
            out.push_str(&format!("{}/\n", rel_str));
        } else {
            out.push_str(&format!("{}/{}\n", rel_str, suffix));
        }

        walk_files(out, root, &subdir, depth + 1, max_depth, config);
    }
}

// ── File collection and formatting ────────────────────────────────────────────

fn collect_files(dir: &Path, config: &TreeConfig) -> Vec<String> {
    let Ok(entries) = fs::read_dir(dir) else { return Vec::new() };

    let mut files: Vec<String> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() {
                return None;
            }
            let name = path.file_name()?.to_string_lossy().to_string();
            if config.exclude_files.iter().any(|f| f == &name) {
                return None;
            }
            let included = config.include_extensions.iter().any(|ext| name.ends_with(ext.as_str()));
            if included { Some(name) } else { None }
        })
        .collect();

    files.sort();
    files
}

/// Format the file suffix for a directory line.
///
/// - Empty file list              → empty string (no suffix)
/// - ≤ inline_threshold files     → ` {file1, file2, ...}`
/// - > inline_threshold files     → ` [N files: Xmd Yrs ...]`
fn format_file_suffix(files: &[String], config: &TreeConfig) -> String {
    if files.is_empty() {
        return String::new();
    }
    if files.len() <= config.inline_threshold {
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

fn read_subdirs(dir: &Path, config: &TreeConfig) -> Vec<std::path::PathBuf> {
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
            if config.exclude_dirs.iter().any(|d| d == name_str.as_ref()) {
                return None;
            }
            Some(path)
        })
        .collect()
}
