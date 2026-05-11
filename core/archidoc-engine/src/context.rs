//! LLM-optimized context snapshot generator.
//!
//! Combines directory tree structure with inline strategy annotations
//! to produce a single compact output that gives an LLM instant
//! comprehension of a codebase's layout and purpose.
//!
//! Output format:
//! ```text
//! # Context: project-name
//!
//! ## Tree
//!
//! src/ — REST API gateway [container]
//!   api/ — HTTP route handlers [component, Facade]
//!     mod.rs  handlers.rs  types.rs
//!   engine/
//!     mod.rs  pipeline.rs
//! tests/ [8 files]
//! Cargo.toml  README.md
//!
//! ## Modules
//!
//! | Path | Level | Pattern | Description | Health |
//! |------|-------|---------|-------------|--------|
//! | src | container | Mediator | REST API gateway | 2 stable, 1 active |
//! | src/api | component | Facade | HTTP route handlers | 3 planned |
//! ```

use archidoc_types::annotation::HealthStatus;
use archidoc_types::ir::{ArchitectureIR, DirNode};

/// Generate an LLM-optimized context snapshot (directories only).
///
/// Produces a compact directory tree with inline annotations and file counts.
/// Individual files are NOT listed — use `ai-files` for that.
pub fn generate(ir: &ArchitectureIR, max_depth: Option<usize>) -> String {
    let mut out = String::new();

    // Header
    let project_name = ir.root.description.as_deref().unwrap_or(&ir.root.name);
    out.push_str(&format!("# {} — Directory Structure\n\n", project_name));

    // Tree section (directories only)
    walk_dirs_only(&mut out, &ir.root, 0, max_depth);
    out.push('\n');

    // Modules section (annotated dirs only)
    let annotated = ir.annotated_dirs();
    if !annotated.is_empty() {
        out.push_str("## Modules\n\n");
        out.push_str("| Path | Level | Pattern | Description | Health |\n");
        out.push_str("|------|-------|---------|-------------|--------|\n");

        for dir in &annotated {
            let level = dir.c4_level.map(|l| l.to_string()).unwrap_or_default();
            let pattern = dir.pattern.as_deref().unwrap_or("--");
            let desc = dir.description.as_deref().unwrap_or("--");
            let health = health_summary(&dir.files);

            out.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                dir.path, level, pattern, desc, health
            ));
        }
        out.push('\n');
    }

    out
}

/// Walk the tree emitting directories only, with file counts in parentheses.
fn walk_dirs_only(out: &mut String, node: &DirNode, depth: usize, max_depth: Option<usize>) {
    if depth > 0 {
        let indent = "  ".repeat(depth - 1);

        // Build dir line: indent + name/ + description + [c4, pattern] + (N files)
        let mut line = format!("{}{}/", indent, node.name);

        // Description
        if let Some(desc) = &node.description {
            line.push_str(&format!("  — {}", desc));
        }

        // Tags: [c4_level, pattern]
        let mut tags = Vec::new();
        if let Some(level) = node.c4_level {
            tags.push(level.to_string());
        }
        if let Some(pat) = &node.pattern {
            tags.push(pat.clone());
        }
        if !tags.is_empty() {
            line.push_str(&format!("  [{}]", tags.join(", ")));
        }

        // File count hint
        let fc = node.file_count();
        if fc == 0 {
            if node.dirs.is_empty() {
                line.push_str("  (empty)");
            }
        } else if fc == 1 {
            line.push_str("  (1 file)");
        } else if node.is_scaffold() {
            line.push_str(&format!("  ({} files scaffold)", fc));
        } else {
            line.push_str(&format!("  ({} files)", fc));
        }

        out.push_str(&line);
        out.push('\n');
    }

    if let Some(max) = max_depth {
        if depth >= max {
            return;
        }
    }

    for child in &node.dirs {
        walk_dirs_only(out, child, depth + 1, max_depth);
    }
}

/// Summarize file health for an annotated dir's files.
fn health_summary(files: &[archidoc_types::ir::FileNode]) -> String {
    let attributed: Vec<_> = files.iter().filter(|f| f.health.is_some()).collect();
    if attributed.is_empty() {
        return "--".to_string();
    }

    let stable = attributed.iter().filter(|f| f.health == Some(HealthStatus::Stable)).count();
    let active = attributed.iter().filter(|f| f.health == Some(HealthStatus::Active)).count();
    let planned = attributed.iter().filter(|f| f.health == Some(HealthStatus::Planned)).count();

    let mut parts = Vec::new();
    if stable > 0 { parts.push(format!("{} stable", stable)); }
    if active > 0 { parts.push(format!("{} active", active)); }
    if planned > 0 { parts.push(format!("{} planned", planned)); }

    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::ir::{DirNode, FileNode};
    use archidoc_types::C4Level;

    fn make_ir(root: DirNode) -> ArchitectureIR {
        ArchitectureIR {
            version: "2.0".to_string(),
            scan_root: "/test".to_string(),
            root,
        }
    }

    #[test]
    fn empty_project_produces_minimal_output() {
        let ir = make_ir(DirNode::empty(".", "."));
        let out = generate(&ir, None);
        assert!(out.contains("— Directory Structure"));
        assert!(!out.contains("## Modules"));
    }

    #[test]
    fn annotated_dirs_appear_in_modules_table() {
        let mut root = DirNode::empty(".", ".");
        let mut api = DirNode::empty("api", "api");
        api.c4_level = Some(C4Level::Container);
        api.description = Some("REST API".to_string());
        api.pattern = Some("Facade".to_string());
        root.dirs.push(api);

        let out = generate(&make_ir(root), None);
        assert!(out.contains("## Modules"));
        assert!(out.contains("| api | container | Facade | REST API |"));
    }

    #[test]
    fn tree_shows_inline_annotation() {
        let mut root = DirNode::empty(".", ".");
        let mut api = DirNode::empty("api", "api");
        api.c4_level = Some(C4Level::Component);
        api.description = Some("HTTP handlers".to_string());
        root.dirs.push(api);

        let out = generate(&make_ir(root), None);
        assert!(out.contains("api/  — HTTP handlers  [component]"));
    }

    #[test]
    fn files_not_listed_in_tree() {
        let mut root = DirNode::empty(".", ".");
        let mut src = DirNode::empty("src", "src");
        src.files.push(FileNode::bare("mod.rs"));
        src.files.push(FileNode::bare("lib.rs"));
        root.dirs.push(src);

        let out = generate(&make_ir(root), None);
        // Files should NOT appear individually — only as count
        assert!(!out.contains("mod.rs"));
        assert!(!out.contains("lib.rs"));
        assert!(out.contains("(2 files") ); // may be "(2 files)" or "(2 files scaffold)"
    }

    #[test]
    fn health_summary_format() {
        let files = vec![
            FileNode { health: Some(HealthStatus::Stable), ..FileNode::bare("a.rs") },
            FileNode { health: Some(HealthStatus::Stable), ..FileNode::bare("b.rs") },
            FileNode { health: Some(HealthStatus::Active), ..FileNode::bare("c.rs") },
        ];
        assert_eq!(health_summary(&files), "2 stable, 1 active");
    }
}
