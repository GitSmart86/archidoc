use std::fs;
use std::path::Path;

use archidoc_types::ModuleDoc;
use walkdir::WalkDir;

use crate::parser;
use crate::path_resolver;

/// Walk a source tree and extract ModuleDocs from all module entry files.
///
/// Finds `lib.rs` and `mod.rs` files, extracts `//!` doc comments,
/// and builds ModuleDoc structs from the parsed annotations.
pub fn extract_all_docs(root: &Path) -> Vec<ModuleDoc> {
    let mut docs = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) if name == "lib.rs" || name == "mod.rs" => name,
            _ => continue,
        };

        let content = match parser::archidoc_from_file(path) {
            Some(c) if !c.trim().is_empty() => c,
            _ => continue,
        };

        let module_path = path_resolver::path_to_module_name(path, root, filename);
        let c4_level = parser::extract_c4_level(&content);
        let pattern = parser::extract_pattern(&content);
        let pattern_status = parser::extract_pattern_status(&content);
        let description = parser::extract_description(&content);
        let parent_container = parser::extract_parent_container(&module_path);
        let relationships = parser::extract_relationships(&content);
        let files = parser::extract_file_table(&content);

        docs.push(ModuleDoc {
            module_path,
            content,
            source_file: path.to_string_lossy().to_string(),
            c4_level,
            pattern,
            pattern_status,
            description,
            parent_container,
            relationships,
            files,
        });
    }

    docs.sort_by(|a, b| a.module_path.cmp(&b.module_path));
    docs
}

/// Read all `.rs` source files in a directory and return their contents.
///
/// Returns a vec of `(filename, source_code)` pairs. Skips files that
/// cannot be read. Used by pattern heuristics to scan a module directory
/// for structural evidence without coupling AST analysis to filesystem I/O.
pub fn read_rs_sources(dir: &Path) -> Vec<(String, String)> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    entries
        .filter_map(|e| e.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let filename = path.file_name()?.to_string_lossy().to_string();
                let source = fs::read_to_string(&path).ok()?;
                Some((filename, source))
            } else {
                None
            }
        })
        .collect()
}
