//! Explicit file map generator — every file listed per directory.
//!
//! Produces a compact listing where every directory shows ALL its files
//! with no numeric compression. Designed for LLM grep/search — when the
//! LLM needs to find a specific file by name, every filename must be visible.
//!
//! Output format:
//! ```text
//! ./ {Cargo.toml, README.md, lib.rs}
//! src/ {main.rs, config.rs, utils.rs}
//! src/api/ {mod.rs, handlers.rs, types.rs, middleware.rs}
//! src/engine/ {mod.rs, pipeline.rs, transform.rs}
//! tests/ {integration.rs, unit.rs}
//! ```

use archidoc_types::ir::DirNode;

/// Generate an explicit file map — every file in every directory, no compression.
pub fn generate(root: &DirNode, max_depth: Option<usize>) -> String {
    let mut out = String::new();
    walk(&mut out, root, 0, max_depth);
    out
}

fn walk(out: &mut String, node: &DirNode, depth: usize, max_depth: Option<usize>) {
    let file_names: Vec<&str> = node
        .files
        .iter()
        .map(|f| f.name.as_str())
        .collect();

    // Always emit the directory line with its files
    if !file_names.is_empty() {
        out.push_str(&format!(
            "{}/ {{{}}}\n",
            node.path,
            file_names.join(", ")
        ));
    } else if !node.dirs.is_empty() {
        // Dir with subdirs but no files — still show it
        out.push_str(&format!("{}/\n", node.path));
    }

    if let Some(max) = max_depth {
        if depth >= max {
            return;
        }
    }

    for child in &node.dirs {
        walk(out, child, depth + 1, max_depth);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::ir::{DirNode, FileNode};

    fn bare_root() -> DirNode {
        DirNode::empty(".", ".")
    }

    #[test]
    fn empty_dir_produces_nothing() {
        let root = bare_root();
        let out = generate(&root, None);
        assert!(out.is_empty() || out.trim().is_empty());
    }

    #[test]
    fn lists_all_files_explicitly() {
        let mut root = bare_root();
        root.files.push(FileNode::bare("Cargo.toml"));
        root.files.push(FileNode::bare("README.md"));
        root.files.push(FileNode::bare("lib.rs"));

        let out = generate(&root, None);
        assert!(out.contains("./ {Cargo.toml, README.md, lib.rs}"));
    }

    #[test]
    fn nested_dirs_show_full_path() {
        let mut root = bare_root();
        let mut src = DirNode::empty("src", "src");
        src.files.push(FileNode::bare("main.rs"));
        src.files.push(FileNode::bare("config.rs"));
        root.dirs.push(src);

        let out = generate(&root, None);
        assert!(out.contains("src/ {main.rs, config.rs}"));
    }

    #[test]
    fn no_compression_even_with_many_files() {
        let mut root = bare_root();
        let mut dir = DirNode::empty("big", "big");
        for i in 0..20 {
            dir.files.push(FileNode::bare(&format!("file{}.rs", i)));
        }
        root.dirs.push(dir);

        let out = generate(&root, None);
        // All 20 files should be listed, no "[20 files: ...]"
        assert!(out.contains("file0.rs"));
        assert!(out.contains("file19.rs"));
        assert!(!out.contains("["));
    }

    #[test]
    fn depth_limit_respected() {
        let mut root = bare_root();
        let mut a = DirNode::empty("a", "a");
        let mut b = DirNode::empty("b", "a/b");
        b.files.push(FileNode::bare("deep.rs"));
        a.dirs.push(b);
        a.files.push(FileNode::bare("shallow.rs"));
        root.dirs.push(a);

        let out = generate(&root, Some(1));
        assert!(out.contains("a/ {shallow.rs}"));
        assert!(!out.contains("deep.rs"));
    }
}
