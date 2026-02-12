use std::fs;
use std::path::Path;

use archidoc_types::{DriftReport, DriftedFile, ModuleDoc};

/// Check for documentation drift.
///
/// Generates all outputs to a temp directory, then compares against the
/// existing output directory file-by-file. Returns a report of differences.
///
/// This is the core logic shared by `--check` CLI mode (B1) and the
/// fitness function API (B5).
pub fn check_drift(docs: &[ModuleDoc], existing_output: &Path) -> DriftReport {
    let temp_dir = tempfile::TempDir::new().expect("failed to create temp dir for drift check");
    let temp_path = temp_dir.path();

    // Generate to temp directory
    let design = temp_path.join("design");
    let c4 = temp_path.join("c4");
    let drawio = temp_path.join("drawio");

    fs::create_dir_all(&design).expect("failed to create temp design dir");
    fs::create_dir_all(&c4).expect("failed to create temp c4 dir");
    fs::create_dir_all(&drawio).expect("failed to create temp drawio dir");

    crate::markdown::generate_all(&design, docs);
    crate::mermaid::generate_container(&c4, docs);
    crate::mermaid::generate_component(&c4, docs);
    crate::drawio::generate_container_csv(&drawio, docs);
    crate::drawio::generate_component_csv(&drawio, docs);

    // Compare generated vs existing
    compare_directories(temp_path, existing_output)
}

/// Compare two directory trees recursively.
fn compare_directories(generated: &Path, existing: &Path) -> DriftReport {
    let mut report = DriftReport::default();

    if !existing.exists() {
        // No existing output at all — everything is "missing"
        collect_all_files(generated, generated, &mut report.missing_files);
        return report;
    }

    // Check generated files against existing
    visit_generated(generated, generated, existing, &mut report);

    // Check for extra files in existing that weren't generated
    visit_extra(existing, existing, generated, &mut report);

    report
}

fn visit_generated(
    base: &Path,
    current: &Path,
    existing_base: &Path,
    report: &mut DriftReport,
) {
    let entries = match fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative = path.strip_prefix(base).unwrap_or(&path);
        let existing_path = existing_base.join(relative);

        if path.is_dir() {
            visit_generated(base, &path, existing_base, report);
        } else if !existing_path.exists() {
            report
                .missing_files
                .push(relative.to_string_lossy().to_string());
        } else {
            let generated_content = fs::read_to_string(&path).unwrap_or_default();
            let existing_content = fs::read_to_string(&existing_path).unwrap_or_default();

            if generated_content != existing_content {
                report.drifted_files.push(DriftedFile {
                    path: relative.to_string_lossy().to_string(),
                    expected_lines: generated_content.lines().count(),
                    actual_lines: existing_content.lines().count(),
                });
            }
        }
    }
}

fn visit_extra(
    base: &Path,
    current: &Path,
    generated_base: &Path,
    report: &mut DriftReport,
) {
    let entries = match fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative = path.strip_prefix(base).unwrap_or(&path);
        let generated_path = generated_base.join(relative);

        if path.is_dir() {
            visit_extra(base, &path, generated_base, report);
        } else if !generated_path.exists() {
            report
                .extra_files
                .push(relative.to_string_lossy().to_string());
        }
    }
}

fn collect_all_files(base: &Path, current: &Path, files: &mut Vec<String>) {
    let entries = match fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            collect_all_files(base, &path, files);
        } else {
            let relative = path.strip_prefix(base).unwrap_or(&path);
            files.push(relative.to_string_lossy().to_string());
        }
    }
}

/// Format a drift report as human-readable text.
pub fn format_drift_report(report: &DriftReport) -> String {
    let mut out = String::new();

    if !report.has_drift() {
        out.push_str("Documentation is up to date.\n");
        return out;
    }

    out.push_str("Documentation drift detected!\n\n");

    if !report.drifted_files.is_empty() {
        out.push_str(&format!(
            "Changed files ({}):\n",
            report.drifted_files.len()
        ));
        for file in &report.drifted_files {
            out.push_str(&format!("  {} (expected {} lines, got {})\n",
                file.path, file.expected_lines, file.actual_lines));
        }
    }

    if !report.missing_files.is_empty() {
        out.push_str(&format!(
            "Missing files ({}):\n",
            report.missing_files.len()
        ));
        for file in &report.missing_files {
            out.push_str(&format!("  {}\n", file));
        }
    }

    if !report.extra_files.is_empty() {
        out.push_str(&format!(
            "Extra files ({}):\n",
            report.extra_files.len()
        ));
        for file in &report.extra_files {
            out.push_str(&format!("  {}\n", file));
        }
    }

    out.push_str("\nRun `archidoc` to regenerate.\n");
    out
}
