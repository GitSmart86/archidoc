use std::fs;
use std::path::Path;

use archidoc_types::{DriftReport, DriftedFile, ModuleDoc};

/// Check for documentation drift against a single ARCHITECTURE.md file.
///
/// Generates the expected content in memory and compares it to the
/// existing file on disk. Returns a report of differences.
pub fn check_drift(docs: &[ModuleDoc], architecture_file: &Path, root: &Path) -> DriftReport {
    let expected = crate::architecture::generate(docs, root);

    let mut report = DriftReport::default();

    if !architecture_file.exists() {
        report
            .missing_files
            .push("ARCHITECTURE.md".to_string());
        return report;
    }

    let actual = fs::read_to_string(architecture_file).unwrap_or_default();

    if expected != actual {
        report.drifted_files.push(DriftedFile {
            path: "ARCHITECTURE.md".to_string(),
            expected_lines: expected.lines().count(),
            actual_lines: actual.lines().count(),
        });
    }

    report
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
