//! Design Validation — diff a design IR (intent) against an actual IR (reality).
//!
//! Two ArchitectureIR files with the same schema, different semantics:
//! - `design.json` — what SHOULD exist (output of Phase 5 / architect's intent)
//! - `architecture.json` — what DOES exist (output of `archidoc compile ir`)
//!
//! The validator produces three categories of findings:
//! - **Unimplemented**: in design but not reality (planned work not yet done)
//! - **Undocumented**: in reality but not design (growth without design update)
//! - **Diverged**: in both but attributes conflict (structural drift)

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use archidoc_types::ir::{ArchitectureIR, DirNode, FileNode};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => write!(f, "ERROR"),
            Self::Warning => write!(f, "WARN"),
            Self::Info => write!(f, "INFO"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingKind {
    /// In design but not in reality — planned work not yet done.
    Unimplemented,
    /// In reality but not in design — undocumented growth.
    Undocumented,
    /// In both but attributes conflict — structural drift.
    Diverged,
    /// Health moved backward (e.g., stable → active).
    Regression,
}

impl fmt::Display for FindingKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unimplemented => write!(f, "UNIMPLEMENTED"),
            Self::Undocumented => write!(f, "UNDOCUMENTED"),
            Self::Diverged => write!(f, "DIVERGED"),
            Self::Regression => write!(f, "REGRESSION"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub severity: Severity,
    pub kind: FindingKind,
    /// Path to the directory or file (relative to scan_root).
    pub path: String,
    /// Human-readable description of the finding.
    pub message: String,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    pub findings: Vec<Finding>,
}

impl ValidationReport {
    pub fn is_clean(&self) -> bool {
        self.findings.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Error)
    }

    pub fn has_warnings(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Warning)
    }

    /// Should the validation fail (exit non-zero)?
    pub fn should_fail(&self, strict: bool) -> bool {
        if strict {
            self.has_errors() || self.has_warnings()
        } else {
            self.has_errors()
        }
    }

    pub fn format(&self) -> String {
        if self.is_clean() {
            return "Design validation passed — actual matches design.\n".to_string();
        }

        let mut out = String::new();
        let errors = self.findings.iter().filter(|f| f.severity == Severity::Error).count();
        let warnings = self.findings.iter().filter(|f| f.severity == Severity::Warning).count();
        let infos = self.findings.iter().filter(|f| f.severity == Severity::Info).count();

        out.push_str(&format!(
            "Design validation: {} error(s), {} warning(s), {} info(s)\n\n",
            errors, warnings, infos
        ));

        for finding in &self.findings {
            out.push_str(&format!(
                "  [{}] {} — {} — {}\n",
                finding.severity, finding.kind, finding.path, finding.message
            ));
        }

        out.push('\n');
        out
    }

    /// Summary counts for machine consumption.
    pub fn summary(&self) -> ValidationSummary {
        let mut s = ValidationSummary::default();
        for f in &self.findings {
            match f.kind {
                FindingKind::Unimplemented => s.unimplemented += 1,
                FindingKind::Undocumented => s.undocumented += 1,
                FindingKind::Diverged => s.diverged += 1,
                FindingKind::Regression => s.regressions += 1,
            }
        }
        s
    }
}

#[derive(Debug, Clone, Default)]
pub struct ValidationSummary {
    pub unimplemented: usize,
    pub undocumented: usize,
    pub diverged: usize,
    pub regressions: usize,
}

// ---------------------------------------------------------------------------
// Core validation logic
// ---------------------------------------------------------------------------

/// Validate actual IR against a design IR.
///
/// The design is the source of truth for INTENT.
/// The actual is the source of truth for REALITY.
pub fn validate(design: &ArchitectureIR, actual: &ArchitectureIR) -> ValidationReport {
    let mut report = ValidationReport::default();
    diff_dir(&design.root, &actual.root, &mut report);
    report
}

fn diff_dir(design: &DirNode, actual: &DirNode, report: &mut ValidationReport) {
    // Compare strategy fields
    diff_dir_strategy(design, actual, report);

    // Compare files
    let design_files: BTreeMap<&str, &FileNode> =
        design.files.iter().map(|f| (f.name.as_str(), f)).collect();
    let actual_files: BTreeMap<&str, &FileNode> =
        actual.files.iter().map(|f| (f.name.as_str(), f)).collect();

    let design_names: BTreeSet<&str> = design_files.keys().copied().collect();
    let actual_names: BTreeSet<&str> = actual_files.keys().copied().collect();

    // Files in design but not actual
    for name in design_names.difference(&actual_names) {
        let design_file = design_files[name];
        let file_path = if design.path == "." {
            name.to_string()
        } else {
            format!("{}/{}", design.path, name)
        };
        // Only flag as unimplemented if it was attributed (not just structural)
        if design_file.health.is_some() || design_file.purpose.is_some() {
            report.findings.push(Finding {
                severity: Severity::Info,
                kind: FindingKind::Unimplemented,
                path: file_path,
                message: format!(
                    "planned in design (health: {}) but not present in actual",
                    design_file.health.map(|h| h.to_string()).unwrap_or_else(|| "none".to_string())
                ),
            });
        }
    }

    // Files in actual but not design
    for name in actual_names.difference(&design_names) {
        let actual_file = actual_files[name];
        let file_path = if design.path == "." {
            name.to_string()
        } else {
            format!("{}/{}", design.path, name)
        };
        // Only flag if the actual file is attributed (not just bare structural)
        if actual_file.health.is_some() || actual_file.purpose.is_some() {
            report.findings.push(Finding {
                severity: Severity::Warning,
                kind: FindingKind::Undocumented,
                path: file_path,
                message: "present in actual but not in design — update design to match".to_string(),
            });
        }
    }

    // Files in both — check for divergence and regression
    for name in design_names.intersection(&actual_names) {
        let d = design_files[name];
        let a = actual_files[name];
        let file_path = if design.path == "." {
            name.to_string()
        } else {
            format!("{}/{}", design.path, name)
        };
        diff_file(d, a, &file_path, report);
    }

    // Compare child directories
    let design_dirs: BTreeMap<&str, &DirNode> =
        design.dirs.iter().map(|d| (d.name.as_str(), d)).collect();
    let actual_dirs: BTreeMap<&str, &DirNode> =
        actual.dirs.iter().map(|d| (d.name.as_str(), d)).collect();

    let design_dir_names: BTreeSet<&str> = design_dirs.keys().copied().collect();
    let actual_dir_names: BTreeSet<&str> = actual_dirs.keys().copied().collect();

    // Dirs in design but not actual
    for name in design_dir_names.difference(&actual_dir_names) {
        let d = design_dirs[name];
        if d.is_annotated() {
            report.findings.push(Finding {
                severity: Severity::Warning,
                kind: FindingKind::Unimplemented,
                path: d.path.clone(),
                message: format!(
                    "designed as {} but directory not present in actual",
                    d.c4_level.map(|l| l.to_string()).unwrap_or_else(|| "unknown".to_string())
                ),
            });
        }
    }

    // Dirs in actual but not design
    for name in actual_dir_names.difference(&design_dir_names) {
        let a = actual_dirs[name];
        if a.is_annotated() {
            report.findings.push(Finding {
                severity: Severity::Warning,
                kind: FindingKind::Undocumented,
                path: a.path.clone(),
                message: "annotated directory exists in actual but not in design".to_string(),
            });
        }
    }

    // Dirs in both — recurse
    for name in design_dir_names.intersection(&actual_dir_names) {
        let d = design_dirs[name];
        let a = actual_dirs[name];
        diff_dir(d, a, report);
    }
}

fn diff_dir_strategy(design: &DirNode, actual: &DirNode, report: &mut ValidationReport) {
    // C4 level conflict
    if let (Some(d_level), Some(a_level)) = (design.c4_level, actual.c4_level) {
        if d_level != a_level {
            report.findings.push(Finding {
                severity: Severity::Error,
                kind: FindingKind::Diverged,
                path: design.path.clone(),
                message: format!(
                    "C4 level: design='{}' vs actual='{}'",
                    d_level, a_level
                ),
            });
        }
    }

    // Pattern divergence
    if let (Some(d_pat), Some(a_pat)) = (&design.pattern, &actual.pattern) {
        if d_pat != a_pat {
            report.findings.push(Finding {
                severity: Severity::Warning,
                kind: FindingKind::Diverged,
                path: design.path.clone(),
                message: format!(
                    "pattern: design='{}' vs actual='{}'",
                    d_pat, a_pat
                ),
            });
        }
    }

    // Design is annotated but actual is not (regression — lost annotation)
    if design.is_annotated() && !actual.is_annotated() {
        report.findings.push(Finding {
            severity: Severity::Warning,
            kind: FindingKind::Regression,
            path: design.path.clone(),
            message: "directory was annotated in design but lost annotation in actual".to_string(),
        });
    }
}

fn diff_file(design: &FileNode, actual: &FileNode, path: &str, report: &mut ValidationReport) {
    // Health regression check
    if let (Some(d_health), Some(a_health)) = (design.health, actual.health) {
        if is_regression(d_health, a_health) {
            report.findings.push(Finding {
                severity: Severity::Warning,
                kind: FindingKind::Regression,
                path: path.to_string(),
                message: format!(
                    "health regressed: design='{}' vs actual='{}'",
                    d_health, a_health
                ),
            });
        }
    }

    // Pattern divergence
    if let (Some(d_pat), Some(a_pat)) = (&design.pattern, &actual.pattern) {
        if d_pat != a_pat {
            report.findings.push(Finding {
                severity: Severity::Info,
                kind: FindingKind::Diverged,
                path: path.to_string(),
                message: format!(
                    "pattern: design='{}' vs actual='{}'",
                    d_pat, a_pat
                ),
            });
        }
    }
}

/// Check if health moved backward.
/// Progression: Planned(0) → Active(1) → Stable(2).
/// Regression = actual rank < design rank.
fn is_regression(
    design: archidoc_types::annotation::HealthStatus,
    actual: archidoc_types::annotation::HealthStatus,
) -> bool {
    use archidoc_types::annotation::HealthStatus;
    let rank = |h: HealthStatus| -> u8 {
        match h {
            HealthStatus::Planned => 0,
            HealthStatus::Active => 1,
            HealthStatus::Stable => 2,
        }
    };
    rank(actual) < rank(design)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::annotation::HealthStatus;
    use archidoc_types::ir::DirNode;
    use archidoc_types::C4Level;

    fn make_ir(root: DirNode) -> ArchitectureIR {
        ArchitectureIR {
            version: "2.0".to_string(),
            scan_root: "/test".to_string(),
            root,
        }
    }

    fn bare_root() -> DirNode {
        DirNode::empty(".", ".")
    }

    fn annotated_dir(name: &str, path: &str, level: C4Level) -> DirNode {
        let mut d = DirNode::empty(name, path);
        d.c4_level = Some(level);
        d.description = Some(format!("{} module", name));
        d
    }

    fn file_with_health(name: &str, health: HealthStatus) -> FileNode {
        FileNode {
            name: name.to_string(),
            health: Some(health),
            purpose: Some("test file".to_string()),
            ..FileNode::bare(name)
        }
    }

    #[test]
    fn identical_irs_produce_clean_report() {
        let mut root = bare_root();
        root.dirs.push(annotated_dir("api", "api", C4Level::Container));
        root.files.push(file_with_health("lib.rs", HealthStatus::Stable));

        let design = make_ir(root.clone());
        let actual = make_ir(root);

        let report = validate(&design, &actual);
        assert!(report.is_clean());
    }

    #[test]
    fn unimplemented_dir_detected() {
        let mut design_root = bare_root();
        design_root.dirs.push(annotated_dir("api", "api", C4Level::Container));
        design_root.dirs.push(annotated_dir("auth", "auth", C4Level::Component));

        let mut actual_root = bare_root();
        actual_root.dirs.push(annotated_dir("api", "api", C4Level::Container));
        // auth is missing from actual

        let report = validate(&make_ir(design_root), &make_ir(actual_root));
        assert!(!report.is_clean());
        let finding = report.findings.iter().find(|f| f.path == "auth").unwrap();
        assert_eq!(finding.kind, FindingKind::Unimplemented);
    }

    #[test]
    fn undocumented_dir_detected() {
        let mut design_root = bare_root();
        design_root.dirs.push(annotated_dir("api", "api", C4Level::Container));

        let mut actual_root = bare_root();
        actual_root.dirs.push(annotated_dir("api", "api", C4Level::Container));
        actual_root.dirs.push(annotated_dir("utils", "utils", C4Level::Component));

        let report = validate(&make_ir(design_root), &make_ir(actual_root));
        let finding = report.findings.iter().find(|f| f.path == "utils").unwrap();
        assert_eq!(finding.kind, FindingKind::Undocumented);
    }

    #[test]
    fn c4_level_conflict_is_error() {
        let mut design_root = bare_root();
        design_root.dirs.push(annotated_dir("api", "api", C4Level::Container));

        let mut actual_root = bare_root();
        actual_root.dirs.push(annotated_dir("api", "api", C4Level::Component));

        let report = validate(&make_ir(design_root), &make_ir(actual_root));
        let finding = report.findings.iter().find(|f| f.path == "api").unwrap();
        assert_eq!(finding.kind, FindingKind::Diverged);
        assert_eq!(finding.severity, Severity::Error);
    }

    #[test]
    fn health_regression_detected() {
        let mut design_root = bare_root();
        design_root.files.push(file_with_health("lib.rs", HealthStatus::Stable));

        let mut actual_root = bare_root();
        actual_root.files.push(file_with_health("lib.rs", HealthStatus::Active));

        let report = validate(&make_ir(design_root), &make_ir(actual_root));
        let finding = report.findings.iter().find(|f| f.path == "lib.rs").unwrap();
        assert_eq!(finding.kind, FindingKind::Regression);
    }

    #[test]
    fn health_progression_is_fine() {
        let mut design_root = bare_root();
        design_root.files.push(file_with_health("lib.rs", HealthStatus::Planned));

        let mut actual_root = bare_root();
        actual_root.files.push(file_with_health("lib.rs", HealthStatus::Stable));

        let report = validate(&make_ir(design_root), &make_ir(actual_root));
        assert!(report.is_clean());
    }

    #[test]
    fn unimplemented_file_detected() {
        let mut design_root = bare_root();
        design_root.files.push(file_with_health("auth.rs", HealthStatus::Planned));

        let actual_root = bare_root(); // file missing from actual

        let report = validate(&make_ir(design_root), &make_ir(actual_root));
        let finding = report.findings.iter().find(|f| f.path == "auth.rs").unwrap();
        assert_eq!(finding.kind, FindingKind::Unimplemented);
    }

    #[test]
    fn format_output_readable() {
        let mut design_root = bare_root();
        design_root.dirs.push(annotated_dir("api", "api", C4Level::Container));

        let actual_root = bare_root();

        let report = validate(&make_ir(design_root), &make_ir(actual_root));
        let output = report.format();
        assert!(output.contains("UNIMPLEMENTED"));
        assert!(output.contains("api"));
    }
}
