use archidoc_types::ir::{ArchitectureIR, DirNode};
use archidoc_types::report::{AnnotationStatus, CoverageReport, DirCoverage};

/// Classify a single directory's annotation status from its IR fields.
pub fn classify_dir(node: &DirNode) -> AnnotationStatus {
    if node.c4_level.is_none() {
        return AnnotationStatus::None;
    }
    match &node.description {
        None => AnnotationStatus::Stub,
        Some(desc) => {
            let trimmed = desc.trim();
            if trimmed.is_empty()
                || trimmed.starts_with("TODO")
                || trimmed == "describe this module's responsibility."
                || trimmed == "describe this directory's purpose"
            {
                AnnotationStatus::Stub
            } else {
                AnnotationStatus::Populated
            }
        }
    }
}

/// Compute annotation coverage from a compiled IR.
///
/// If `max_depth` is set, only directories whose path depth (number of `/`
/// separators) is at most that value are included.
pub fn compute_coverage(ir: &ArchitectureIR, max_depth: Option<usize>) -> CoverageReport {
    let all = ir.all_dirs();

    let filtered: Vec<&DirNode> = match max_depth {
        Some(max) => all
            .into_iter()
            .filter(|d| path_depth(&d.path) <= max)
            .collect(),
        None => all,
    };

    let mut report = CoverageReport {
        total_dirs: filtered.len(),
        ..Default::default()
    };

    for node in &filtered {
        let status = classify_dir(node);
        report.per_dir.push(DirCoverage {
            path: node.path.clone(),
            status,
        });
        match status {
            AnnotationStatus::None => report.unannotated_count += 1,
            AnnotationStatus::Stub => {
                report.annotated_count += 1;
                report.stub_count += 1;
            }
            AnnotationStatus::Populated => {
                report.annotated_count += 1;
                report.populated_count += 1;
            }
        }
    }

    report.coverage_percent = if report.total_dirs > 0 {
        report.annotated_count as f64 / report.total_dirs as f64 * 100.0
    } else {
        0.0
    };

    report
}

/// Format a coverage report as human-readable terminal text.
pub fn format_coverage_report(report: &CoverageReport) -> String {
    let mut out = String::new();

    out.push_str("Annotation Coverage\n");
    out.push_str("====================\n");
    out.push_str(&format!("Directories:  {} total\n", report.total_dirs));

    if report.total_dirs == 0 {
        out.push_str("  (no directories found)\n");
        return out;
    }

    let pct = |n: usize| n as f64 / report.total_dirs as f64 * 100.0;

    out.push_str(&format!(
        "  populated:    {:>3} ({:.1}%)\n",
        report.populated_count,
        pct(report.populated_count)
    ));
    out.push_str(&format!(
        "  stub:         {:>3} ({:.1}%)\n",
        report.stub_count,
        pct(report.stub_count)
    ));
    out.push_str(&format!(
        "  unannotated:  {:>3} ({:.1}%)\n",
        report.unannotated_count,
        pct(report.unannotated_count)
    ));

    // List unannotated directories
    let unannotated: Vec<&DirCoverage> = report
        .per_dir
        .iter()
        .filter(|d| d.status == AnnotationStatus::None)
        .collect();

    if !unannotated.is_empty() {
        out.push_str("\nUnannotated:\n");
        for d in &unannotated {
            out.push_str(&format!("  {}/\n", d.path));
        }
    }

    out
}

fn path_depth(path: &str) -> usize {
    if path == "." {
        return 0;
    }
    path.matches('/').count()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::ir::{ArchitectureIR, C4Level, DirNode};

    fn make_dir(name: &str, path: &str, c4: Option<C4Level>, desc: Option<&str>) -> DirNode {
        DirNode {
            name: name.to_string(),
            path: path.to_string(),
            c4_level: c4,
            description: desc.map(|s| s.to_string()),
            ..DirNode::empty(name, path)
        }
    }

    #[test]
    fn classify_none_when_no_c4_level() {
        let node = make_dir("x", "x", None, None);
        assert_eq!(classify_dir(&node), AnnotationStatus::None);
    }

    #[test]
    fn classify_stub_when_no_description() {
        let node = make_dir("x", "x", Some(C4Level::Container), None);
        assert_eq!(classify_dir(&node), AnnotationStatus::Stub);
    }

    #[test]
    fn classify_stub_when_todo_description() {
        let node = make_dir("x", "x", Some(C4Level::Container), Some("TODO — describe this module's responsibility."));
        assert_eq!(classify_dir(&node), AnnotationStatus::Stub);
    }

    #[test]
    fn classify_populated_when_real_description() {
        let node = make_dir("api", "api", Some(C4Level::Container), Some("REST API gateway"));
        assert_eq!(classify_dir(&node), AnnotationStatus::Populated);
    }

    #[test]
    fn coverage_empty_ir() {
        let ir = ArchitectureIR::default();
        let report = compute_coverage(&ir, None);
        assert_eq!(report.total_dirs, 1); // root "." node
    }

    #[test]
    fn coverage_mixed_tree() {
        let mut ir = ArchitectureIR::new("/tmp".to_string());
        ir.root.c4_level = Some(C4Level::Container);
        ir.root.description = Some("Root project".to_string());
        ir.root.dirs = vec![
            make_dir("api", "api", Some(C4Level::Container), Some("REST API")),
            make_dir("utils", "utils", None, None),
            make_dir("db", "db", Some(C4Level::Component), None), // stub
        ];

        let report = compute_coverage(&ir, None);
        assert_eq!(report.total_dirs, 4);
        assert_eq!(report.populated_count, 2); // root + api
        assert_eq!(report.stub_count, 1);      // db
        assert_eq!(report.unannotated_count, 1); // utils
        assert!((report.coverage_percent - 75.0).abs() < 0.1);
    }

    #[test]
    fn coverage_respects_max_depth() {
        let mut ir = ArchitectureIR::new("/tmp".to_string());
        ir.root.dirs = vec![DirNode {
            dirs: vec![make_dir("deep", "a/deep", None, None)],
            ..make_dir("a", "a", Some(C4Level::Container), Some("A"))
        }];

        let report_full = compute_coverage(&ir, None);
        assert_eq!(report_full.total_dirs, 3); // root + a + a/deep

        let report_depth1 = compute_coverage(&ir, Some(1));
        assert_eq!(report_depth1.total_dirs, 3); // root + a + a/deep (depth 0, 0, 1)

        let report_depth0 = compute_coverage(&ir, Some(0));
        assert_eq!(report_depth0.total_dirs, 2); // root + a (both at depth 0)
    }

    #[test]
    fn path_depth_calculation() {
        assert_eq!(path_depth("."), 0);
        assert_eq!(path_depth("src"), 0);
        assert_eq!(path_depth("src/api"), 1);
        assert_eq!(path_depth("src/api/auth"), 2);
    }
}
