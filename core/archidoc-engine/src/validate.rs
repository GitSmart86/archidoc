use std::collections::HashSet;
use std::path::Path;

use archidoc_types::ir::ArchitectureIR;
use archidoc_types::{GhostEntry, OrphanEntry, ValidationReport};

/// Validate file tables against the actual filesystem.
///
/// For each annotated directory with a file catalog:
/// - **Ghost detection** (B4): catalog entries pointing to files that don't exist on disk
/// - **Orphan detection** (B3): `.rs` files on disk not listed in any catalog
///
/// Directories without file catalogs are silently skipped.
pub fn validate_file_tables(ir: &ArchitectureIR) -> ValidationReport {
    let annotated = ir.annotated_dirs();
    let mut report = ValidationReport::default();

    for dir in &annotated {
        let attributed_files: Vec<_> = dir.files.iter()
            .filter(|f| f.health.is_some() || f.purpose.is_some() || f.pattern.is_some())
            .collect();

        if attributed_files.is_empty() {
            continue;
        }

        let source_path = resolve_source_path(dir, &ir.scan_root);
        let source_dir = match Path::new(&source_path).parent() {
            Some(d) => d,
            None => continue,
        };

        let source_dir_str = source_dir.to_string_lossy().to_string();

        // Ghost detection: catalog entries pointing to non-existent files
        let cataloged_names: HashSet<&str> = attributed_files.iter().map(|f| f.name.as_str()).collect();

        for file in &attributed_files {
            let file_path = source_dir.join(&file.name);
            if !file_path.exists() {
                report.ghosts.push(GhostEntry {
                    element: dir.path.clone(),
                    filename: file.name.clone(),
                    source_dir: source_dir_str.clone(),
                });
            }
        }

        // Orphan detection: .rs files on disk not in the catalog
        let structural_files: HashSet<&str> =
            ["mod.rs", "lib.rs", "main.rs"].iter().copied().collect();

        if let Ok(entries) = std::fs::read_dir(source_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let filename = entry.file_name();
                let name = filename.to_string_lossy();

                if name.ends_with(".rs")
                    && !structural_files.contains(name.as_ref())
                    && !cataloged_names.contains(name.as_ref())
                {
                    report.orphans.push(OrphanEntry {
                        element: dir.path.clone(),
                        filename: name.to_string(),
                        source_dir: source_dir_str.clone(),
                    });
                }
            }
        }
    }

    report
}

/// Reconstruct an absolute source file path from a DirNode.
fn resolve_source_path(dir: &archidoc_types::ir::DirNode, scan_root: &str) -> String {
    match &dir.source_file {
        Some(rel) => {
            let root = Path::new(scan_root);
            let native_rel = rel.replace('/', std::path::MAIN_SEPARATOR_STR);
            root.join(native_rel).to_string_lossy().to_string()
        }
        None => String::new(),
    }
}

/// Format a validation report as human-readable text.
pub fn format_validation_report(report: &ValidationReport) -> String {
    let mut out = String::new();

    if report.is_clean() {
        out.push_str("File validation: all clear\n");
        return out;
    }

    if !report.ghosts.is_empty() {
        out.push_str(&format!("Ghost entries ({} found):\n", report.ghosts.len()));
        for ghost in &report.ghosts {
            out.push_str(&format!(
                "  {} — '{}' listed in catalog but not found on disk\n",
                ghost.element, ghost.filename
            ));
        }
    }

    if !report.orphans.is_empty() {
        out.push_str(&format!(
            "Orphan files ({} found):\n",
            report.orphans.len()
        ));
        for orphan in &report.orphans {
            out.push_str(&format!(
                "  {} — '{}' exists on disk but not in catalog\n",
                orphan.element, orphan.filename
            ));
        }
    }

    out
}
