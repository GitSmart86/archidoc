//! Programmatic query commands against compiled ArchitectureIR.
//!
//! These functions read from the already-compiled IR JSON — no filesystem scan.
//! Output is plain text to stdout, designed for LLM consumption.

use archidoc_types::ir::{ArchitectureIR, DirNode};

// ---------------------------------------------------------------------------
// ls — list directory children
// ---------------------------------------------------------------------------

/// List children of a directory in the IR tree.
///
/// `depth` controls how many levels deep to show (default 1 = immediate children).
/// Returns formatted text or an error if the path is not found.
pub fn ls(ir: &ArchitectureIR, path: &str, depth: usize) -> Result<String, String> {
    let node = ir
        .find_dir(path)
        .ok_or_else(|| format!("path not found in IR: '{}'", path))?;

    let mut out = String::new();
    ls_walk(&mut out, node, 0, depth);
    Ok(out)
}

fn ls_walk(out: &mut String, node: &DirNode, current_depth: usize, max_depth: usize) {
    // Skip root (depth 0) — show contents, not the dir itself (Unix ls semantics)
    if current_depth > 0 {
        let indent = "  ".repeat(current_depth - 1);
        let mut line = format!("{}{}/", indent, node.name);

        if let Some(desc) = &node.description {
            line.push_str(&format!("  — {}", desc));
        }

        let fc = node.file_count();
        if fc > 0 {
            line.push_str(&format!("  ({} file{})", fc, if fc == 1 { "" } else { "s" }));
        }

        out.push_str(&line);
        out.push('\n');
    }

    if current_depth >= max_depth {
        return;
    }

    for child in &node.dirs {
        ls_walk(out, child, current_depth + 1, max_depth);
    }
}

// ---------------------------------------------------------------------------
// describe — full detail for one directory
// ---------------------------------------------------------------------------

/// Full detail view for a single directory node.
pub fn describe(ir: &ArchitectureIR, path: &str) -> Result<String, String> {
    let node = ir
        .find_dir(path)
        .ok_or_else(|| format!("path not found in IR: '{}'", path))?;

    let mut out = String::new();

    // Header
    out.push_str(&format!("# {}/\n\n", node.path));

    // Strategy fields
    if let Some(level) = node.c4_level {
        out.push_str(&format!("C4 level: {}\n", level));
    }
    if let Some(desc) = &node.description {
        out.push_str(&format!("Description: {}\n", desc));
    }
    if let Some(pat) = &node.pattern {
        out.push_str(&format!("Pattern: {}\n", pat));
    }
    if node.c4_level.is_some() || node.description.is_some() || node.pattern.is_some() {
        out.push('\n');
    }

    // File counts by extension
    let fc = node.file_count();
    if fc > 0 {
        out.push_str(&format!("Files: {} total\n", fc));
        for (ext, count) in node.extension_counts() {
            out.push_str(&format!("  .{}  {}\n", ext, count));
        }
        out.push('\n');
    }

    // Subdirs
    if !node.dirs.is_empty() {
        out.push_str(&format!("Subdirs: {}\n", node.dirs.len()));
        for child in &node.dirs {
            let mut line = format!("  {}/", child.name);
            if let Some(desc) = &child.description {
                line.push_str(&format!("  — {}", desc));
            }
            let cfc = child.file_count();
            if cfc > 0 {
                line.push_str(&format!("  ({} file{})", cfc, if cfc == 1 { "" } else { "s" }));
            }
            out.push_str(&line);
            out.push('\n');
        }
        out.push('\n');
    }

    // Relationships
    if !node.relationships.is_empty() {
        out.push_str("Relationships:\n");
        for rel in &node.relationships {
            out.push_str(&format!("  {} → {} ({})\n", rel.label, rel.target, rel.protocol));
        }
        out.push('\n');
    }

    // Source annotation file
    if let Some(src) = &node.source_file {
        out.push_str(&format!("Source: {}\n", src));
    }

    Ok(out)
}

// ---------------------------------------------------------------------------
// query — filter directories by state
// ---------------------------------------------------------------------------

/// Filter flags for the query command.
#[derive(Debug, Default)]
pub struct QueryFilter {
    pub empty: bool,
    pub populated: bool,
    pub scaffold: bool,
    pub annotated: bool,
}

impl QueryFilter {
    pub fn has_any_flag(&self) -> bool {
        self.empty || self.populated || self.scaffold || self.annotated
    }
}

/// Filter all directories in the IR by state flags.
///
/// When multiple flags are set, they combine with AND logic.
/// No flags = list all directories.
pub fn query(ir: &ArchitectureIR, filter: &QueryFilter) -> String {
    let all = ir.all_dirs();
    let mut out = String::new();

    for node in &all {
        // Skip root
        if node.path == "." {
            continue;
        }

        if filter.has_any_flag() {
            let matches = (!filter.empty || node.is_empty_leaf())
                && (!filter.populated || node.is_populated())
                && (!filter.scaffold || node.is_scaffold())
                && (!filter.annotated || node.is_annotated());
            if !matches {
                continue;
            }
        }

        let mut line = format!("{}/", node.path);
        if let Some(desc) = &node.description {
            line.push_str(&format!("  — {}", desc));
        }
        let fc = node.file_count();
        if fc > 0 {
            line.push_str(&format!("  ({} file{})", fc, if fc == 1 { "" } else { "s" }));
        }
        out.push_str(&line);
        out.push('\n');
    }

    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

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

    fn sample_tree() -> ArchitectureIR {
        let mut root = DirNode::empty(".", ".");
        let mut src = DirNode::empty("src", "src");
        src.c4_level = Some(C4Level::Container);
        src.description = Some("Source code".to_string());

        let mut api = DirNode::empty("api", "src/api");
        api.files.push(FileNode::bare("mod.rs"));
        api.files.push(FileNode::bare("handlers.rs"));
        src.dirs.push(api);

        let empty_dir = DirNode::empty("empty", "src/empty");
        src.dirs.push(empty_dir);

        root.dirs.push(src);

        let mut docs = DirNode::empty("docs", "docs");
        docs.files.push(FileNode::bare("README.md"));
        root.dirs.push(docs);

        make_ir(root)
    }

    // -- ls tests --

    #[test]
    fn ls_immediate_children() {
        let ir = sample_tree();
        let out = ls(&ir, ".", 1).unwrap();
        assert!(out.contains("src/"));
        assert!(out.contains("docs/"));
        // Should not show nested children at depth 1
        assert!(!out.contains("api/"));
    }

    #[test]
    fn ls_deep_children() {
        let ir = sample_tree();
        let out = ls(&ir, ".", 2).unwrap();
        assert!(out.contains("src/"));
        assert!(out.contains("api/"));
        assert!(out.contains("(2 files)"));
    }

    #[test]
    fn ls_path_not_found_returns_error() {
        let ir = sample_tree();
        let result = ls(&ir, "nonexistent", 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    // -- describe tests --

    #[test]
    fn describe_shows_all_fields() {
        let ir = sample_tree();
        let out = describe(&ir, "src").unwrap();
        assert!(out.contains("# src/"));
        assert!(out.contains("C4 level: container"));
        assert!(out.contains("Source code"));
        assert!(out.contains("Subdirs: 2"));
        assert!(out.contains("api/"));
        assert!(out.contains("empty/"));
    }

    #[test]
    fn describe_path_not_found_returns_error() {
        let ir = sample_tree();
        let result = describe(&ir, "nonexistent");
        assert!(result.is_err());
    }

    // -- query tests --

    #[test]
    fn query_no_filters_returns_all() {
        let ir = sample_tree();
        let filter = QueryFilter::default();
        let out = query(&ir, &filter);
        assert!(out.contains("src/"));
        assert!(out.contains("src/api/"));
        assert!(out.contains("src/empty/"));
        assert!(out.contains("docs/"));
    }

    #[test]
    fn query_annotated_filter() {
        let ir = sample_tree();
        let filter = QueryFilter { annotated: true, ..Default::default() };
        let out = query(&ir, &filter);
        assert!(out.contains("src/"));
        // api is NOT annotated
        assert!(!out.contains("src/api/"));
        assert!(!out.contains("docs/"));
    }

    #[test]
    fn query_empty_filter() {
        let ir = sample_tree();
        let filter = QueryFilter { empty: true, ..Default::default() };
        let out = query(&ir, &filter);
        assert!(out.contains("src/empty/"));
        assert!(!out.contains("src/api/"));
        assert!(!out.contains("docs/"));
    }

    #[test]
    fn query_populated_filter() {
        let ir = sample_tree();
        let filter = QueryFilter { populated: true, ..Default::default() };
        let out = query(&ir, &filter);
        assert!(out.contains("src/api/"));
        assert!(out.contains("docs/"));
        assert!(!out.contains("src/empty/"));
    }

    #[test]
    fn query_combined_filters() {
        let ir = sample_tree();
        // annotated AND populated — src has no files itself, so only annotated+populated would be empty
        let filter = QueryFilter { annotated: true, populated: true, ..Default::default() };
        let out = query(&ir, &filter);
        // src is annotated but not populated (no direct files)
        assert!(!out.contains("src/\n") && !out.contains("src/  —")); // src without children
    }
}
