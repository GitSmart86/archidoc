use std::collections::HashSet;
use std::path::Path;

use archidoc_types::ir::{ArchitectureIR, DirNode};
use archidoc_types::{HealthStatus, PatternStatus};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    /// Short code displayed in brackets: E001, W002, etc.
    pub code: &'static str,
    /// One-line title shown on the first line.
    pub title: &'static str,
    /// File or directory path shown on the `-->` line.
    pub location: String,
    /// Optional extra detail shown below the location.
    pub detail: Option<String>,
    /// Optional suggested fix command.
    pub fix: Option<String>,
}

#[derive(Debug, Default)]
pub struct DiagnosticReport {
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReport {
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }

    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }

    pub fn has_warnings(&self) -> bool {
        self.warning_count() > 0
    }

    /// Whether this report should cause a non-zero exit.
    ///
    /// `strict` promotes warnings to failures.
    pub fn should_fail(&self, strict: bool) -> bool {
        self.has_errors() || (strict && self.has_warnings())
    }

    /// Render the report in compiler/linter style.
    pub fn format(&self) -> String {
        if self.diagnostics.is_empty() {
            return "0 errors, 0 warnings\n".to_string();
        }

        let mut out = String::new();

        for diag in &self.diagnostics {
            let severity_label = match diag.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            };

            out.push_str(&format!(
                "{}[{}]: {}\n",
                severity_label, diag.code, diag.title
            ));

            out.push_str(&format!("  --> {}\n", diag.location));

            if let Some(detail) = &diag.detail {
                out.push_str(&format!("     {}\n", detail));
            }

            if let Some(fix) = &diag.fix {
                out.push_str(&format!("     fix: {}\n", fix));
            }

            out.push('\n');
        }

        let errors = self.error_count();
        let warnings = self.warning_count();
        out.push_str(&format!("{} error{}, {} warning{}\n",
            errors,   if errors   == 1 { "" } else { "s" },
            warnings, if warnings == 1 { "" } else { "s" },
        ));

        out
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Run all diagnostic checks and return a unified report.
pub fn run(
    ir: &ArchitectureIR,
    arch_file: &Path,
    link_base: &Path,
) -> DiagnosticReport {
    let mut report = DiagnosticReport::default();

    check_drift(ir, arch_file, link_base, &mut report);

    let annotated = ir.annotated_dirs();
    check_file_tables(&annotated, &ir.scan_root, &mut report);
    check_health_warnings(&annotated, &ir.scan_root, &mut report);

    report
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// E001 — committed documentation file is out of sync with source annotations.
fn check_drift(
    ir: &ArchitectureIR,
    arch_file: &Path,
    link_base: &Path,
    report: &mut DiagnosticReport,
) {
    let generated = crate::architecture::generate(ir, &[]);

    let committed = match std::fs::read_to_string(arch_file) {
        Ok(s) => s,
        Err(_) => {
            report.diagnostics.push(Diagnostic {
                severity: Severity::Error,
                code: "E001",
                title: "documentation drift",
                location: arch_file.display().to_string(),
                detail: Some("file does not exist".to_string()),
                fix: Some(format!(
                    "archidoc compile architecture.md {}",
                    link_base.display()
                )),
            });
            return;
        }
    };

    if committed != generated {
        report.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "E001",
            title: "documentation drift",
            location: arch_file.display().to_string(),
            detail: Some("committed file does not match current source annotations".to_string()),
            fix: Some(format!(
                "archidoc compile architecture.md {}",
                link_base.display()
            )),
        });
    }
}

/// E002 — file listed in a module's table does not exist on disk.
/// E003 — file exists on disk but is not listed in any module's table.
fn check_file_tables(annotated: &[&DirNode], scan_root: &str, report: &mut DiagnosticReport) {
    let structural: HashSet<&str> = ["mod.rs", "lib.rs", "main.rs"].iter().copied().collect();

    for dir in annotated {
        let attributed_files: Vec<_> = dir.files.iter().filter(|f| f.health.is_some() || f.purpose.is_some() || f.pattern.is_some()).collect();
        if attributed_files.is_empty() {
            continue;
        }

        let source_file = resolve_source_path(dir, scan_root);
        let source_dir = match Path::new(&source_file).parent() {
            Some(d) => d,
            None => continue,
        };

        let cataloged: HashSet<&str> = attributed_files.iter().map(|f| f.name.as_str()).collect();

        // E002: ghost files
        for file in &attributed_files {
            if !source_dir.join(&file.name).exists() {
                report.diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    code: "E002",
                    title: "ghost file",
                    location: source_dir.join(&file.name).display().to_string(),
                    detail: Some(format!(
                        "referenced in {} but not found on disk",
                        source_file
                    )),
                    fix: None,
                });
            }
        }

        // E003: orphan files
        if let Ok(entries) = std::fs::read_dir(source_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_source = name.ends_with(".rs") || name.ends_with(".ts") || name.ends_with(".js");
                if is_source
                    && !structural.contains(name.as_str())
                    && !cataloged.contains(name.as_str())
                {
                    report.diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        code: "E003",
                        title: "orphan file",
                        location: source_dir.join(&name).display().to_string(),
                        detail: Some(format!(
                            "exists on disk but is not listed in {}",
                            source_file
                        )),
                        fix: Some(format!(
                            "add `{}` to the file table in {}",
                            name, source_file
                        )),
                    });
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Warnings
// ---------------------------------------------------------------------------

fn check_health_warnings(annotated: &[&DirNode], scan_root: &str, report: &mut DiagnosticReport) {
    for dir in annotated {
        check_placeholders(dir, scan_root, report);
        check_all_planned(dir, scan_root, report);
        check_unverified_patterns(dir, scan_root, report);
    }
}

/// W001 — description or file purposes still contain TODO placeholders.
fn check_placeholders(dir: &DirNode, scan_root: &str, report: &mut DiagnosticReport) {
    let attributed_files: Vec<_> = dir.files.iter().filter(|f| f.health.is_some() || f.purpose.is_some() || f.pattern.is_some()).collect();

    let todo_entries: Vec<&str> = attributed_files
        .iter()
        .filter(|f| {
            let purpose = f.purpose.as_deref().unwrap_or("");
            purpose.contains("TODO") || purpose.contains("--")
        })
        .map(|f| f.name.as_str())
        .collect();

    let desc = dir.description.as_deref().unwrap_or("");
    let desc_todo = desc.contains("TODO") || desc.contains("--");

    if todo_entries.is_empty() && !desc_todo {
        return;
    }

    let source_file = resolve_source_path(dir, scan_root);
    let count = todo_entries.len() + if desc_todo { 1 } else { 0 };
    report.diagnostics.push(Diagnostic {
        severity: Severity::Warning,
        code: "W001",
        title: "unresolved placeholders",
        location: source_file.clone(),
        detail: Some(format!(
            "{} entr{} still contain TODO or placeholder values",
            count,
            if count == 1 { "y" } else { "ies" }
        )),
        fix: Some(format!("fill in descriptions in {}", source_file)),
    });
}

/// W002 — every file in the module is still at `planned` health.
fn check_all_planned(dir: &DirNode, scan_root: &str, report: &mut DiagnosticReport) {
    let attributed_files: Vec<_> = dir.files.iter().filter(|f| f.health.is_some()).collect();
    if attributed_files.is_empty() {
        return;
    }

    let all_planned = attributed_files
        .iter()
        .all(|f| f.health == Some(HealthStatus::Planned));

    if all_planned {
        let source_file = resolve_source_path(dir, scan_root);
        report.diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "W002",
            title: "all files planned",
            location: source_file,
            detail: Some(format!(
                "all {} file{} are at `planned` health — update as implementation progresses",
                attributed_files.len(),
                if attributed_files.len() == 1 { "" } else { "s" }
            )),
            fix: None,
        });
    }
}

/// W003 — no patterns in the module have been verified by structural analysis.
fn check_unverified_patterns(dir: &DirNode, scan_root: &str, report: &mut DiagnosticReport) {
    let attributed_files: Vec<_> = dir.files.iter().filter(|f| f.health.is_some()).collect();
    if attributed_files.is_empty() {
        return;
    }

    let has_verified = attributed_files
        .iter()
        .any(|f| f.pattern_status == Some(PatternStatus::Verified));

    let dir_pattern_status = dir.pattern_status.unwrap_or_default();

    if !has_verified && dir_pattern_status == PatternStatus::Planned {
        let source_file = resolve_source_path(dir, scan_root);
        report.diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "W003",
            title: "unverified patterns",
            location: source_file,
            detail: Some(
                "no patterns verified — consider running structural analysis".to_string(),
            ),
            fix: None,
        });
    }
}

/// Reconstruct an absolute source file path from a DirNode.
fn resolve_source_path(dir: &DirNode, scan_root: &str) -> String {
    match &dir.source_file {
        Some(rel) => {
            let root = Path::new(scan_root);
            let native_rel = rel.replace('/', std::path::MAIN_SEPARATOR_STR);
            root.join(native_rel).to_string_lossy().to_string()
        }
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::ir::{ArchitectureIR, DirNode, FileNode};
    use archidoc_types::C4Level;
    use tempfile::TempDir;

    fn make_dir_with_files(source_file: &str, files: Vec<FileNode>) -> DirNode {
        let mut dir = DirNode::empty("test", "test");
        dir.c4_level = Some(C4Level::Container);
        dir.pattern = Some("Repository".to_string());
        dir.pattern_status = Some(PatternStatus::Planned);
        dir.description = Some("A test module".to_string());
        dir.source_file = Some(source_file.to_string());
        dir.files = files;
        dir
    }

    fn make_ir(scan_root: &str, dirs: Vec<DirNode>) -> ArchitectureIR {
        let mut root = DirNode::empty(".", ".");
        root.dirs = dirs;
        ArchitectureIR {
            version: "2.0".to_string(),
            scan_root: scan_root.to_string(),
            root,
        }
    }

    fn make_file_node(name: &str, purpose: &str, health: HealthStatus) -> FileNode {
        FileNode {
            name: name.to_string(),
            pattern: None,
            pattern_status: Some(PatternStatus::Planned),
            purpose: if purpose.is_empty() { None } else { Some(purpose.to_string()) },
            health: Some(health),
            extra: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn clean_project_produces_no_diagnostics() {
        let tmp = TempDir::new().unwrap();
        let arch_file = tmp.path().join("ARCHITECTURE.md");

        let ir = make_ir("", vec![]);
        let generated = crate::architecture::generate(&ir, &[]);
        std::fs::write(&arch_file, &generated).unwrap();

        let report = run(&ir, &arch_file, tmp.path());
        assert_eq!(report.error_count(), 0);
        assert_eq!(report.warning_count(), 0);
    }

    #[test]
    fn drift_detected_when_file_is_stale() {
        let tmp = TempDir::new().unwrap();
        let arch_file = tmp.path().join("ARCHITECTURE.md");
        std::fs::write(&arch_file, "stale content").unwrap();

        let ir = make_ir("", vec![]);
        let report = run(&ir, &arch_file, tmp.path());
        assert!(report.diagnostics.iter().any(|d| d.code == "E001"));
    }

    #[test]
    fn drift_detected_when_file_missing() {
        let tmp = TempDir::new().unwrap();
        let arch_file = tmp.path().join("ARCHITECTURE.md");

        let ir = make_ir("", vec![]);
        let report = run(&ir, &arch_file, tmp.path());
        assert!(report.diagnostics.iter().any(|d| d.code == "E001"));
    }

    #[test]
    fn ghost_file_detected() {
        let tmp = TempDir::new().unwrap();
        let test_dir = tmp.path().join("test");
        std::fs::create_dir_all(&test_dir).unwrap();
        let source = test_dir.join("mod.rs");
        std::fs::write(&source, "").unwrap();

        let source_rel = "test/mod.rs";
        let dir = make_dir_with_files(
            source_rel,
            vec![make_file_node("ghost.rs", "TODO", HealthStatus::Planned)],
        );

        let ir = make_ir(&tmp.path().to_string_lossy(), vec![dir]);
        let arch_file = tmp.path().join("ARCHITECTURE.md");
        let generated = crate::architecture::generate(&ir, &[]);
        std::fs::write(&arch_file, generated).unwrap();

        let report = run(&ir, &arch_file, tmp.path());
        assert!(report.diagnostics.iter().any(|d| d.code == "E002"));
    }

    #[test]
    fn placeholder_warning_fires_on_todo() {
        let tmp = TempDir::new().unwrap();
        let test_dir = tmp.path().join("test");
        std::fs::create_dir_all(&test_dir).unwrap();
        let source = test_dir.join("mod.rs");
        std::fs::write(&source, "").unwrap();
        let real = test_dir.join("routes.rs");
        std::fs::write(&real, "").unwrap();

        let source_rel = "test/mod.rs";
        let dir = make_dir_with_files(
            source_rel,
            vec![make_file_node("routes.rs", "TODO", HealthStatus::Active)],
        );

        let ir = make_ir(&tmp.path().to_string_lossy(), vec![dir]);
        let arch_file = tmp.path().join("ARCHITECTURE.md");
        let generated = crate::architecture::generate(&ir, &[]);
        std::fs::write(&arch_file, generated).unwrap();

        let report = run(&ir, &arch_file, tmp.path());
        assert!(report.diagnostics.iter().any(|d| d.code == "W001"));
    }

    #[test]
    fn all_planned_warning_fires() {
        let tmp = TempDir::new().unwrap();
        let test_dir = tmp.path().join("test");
        std::fs::create_dir_all(&test_dir).unwrap();
        let source = test_dir.join("mod.rs");
        std::fs::write(&source, "").unwrap();
        let real = test_dir.join("routes.rs");
        std::fs::write(&real, "").unwrap();

        let source_rel = "test/mod.rs";
        let dir = make_dir_with_files(
            source_rel,
            vec![make_file_node("routes.rs", "HTTP routing", HealthStatus::Planned)],
        );

        let ir = make_ir(&tmp.path().to_string_lossy(), vec![dir]);
        let arch_file = tmp.path().join("ARCHITECTURE.md");
        let generated = crate::architecture::generate(&ir, &[]);
        std::fs::write(&arch_file, generated).unwrap();

        let report = run(&ir, &arch_file, tmp.path());
        assert!(report.diagnostics.iter().any(|d| d.code == "W002"));
    }

    #[test]
    fn strict_mode_fails_on_warnings() {
        let mut report = DiagnosticReport::default();
        report.diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            code: "W001",
            title: "test warning",
            location: "src/lib.rs".to_string(),
            detail: None,
            fix: None,
        });

        assert!(!report.should_fail(false));
        assert!(report.should_fail(true));
    }

    #[test]
    fn format_output_matches_expected_shape() {
        let mut report = DiagnosticReport::default();
        report.diagnostics.push(Diagnostic {
            severity: Severity::Error,
            code: "E001",
            title: "documentation drift",
            location: "_context/archidoc/ARCHITECTURE.md".to_string(),
            detail: Some("committed file does not match source".to_string()),
            fix: Some("archidoc compile architecture.md .".to_string()),
        });

        let output = report.format();
        assert!(output.contains("error[E001]"));
        assert!(output.contains("documentation drift"));
        assert!(output.contains("-->"));
        assert!(output.contains("fix:"));
        assert!(output.contains("1 error"));
    }
}
