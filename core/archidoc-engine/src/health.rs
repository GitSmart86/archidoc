use archidoc_types::ir::ArchitectureIR;
use archidoc_types::{
    C4Level, ElementHealth, HealthReport, HealthStatus, PatternStatus,
};

/// Aggregate health across all architectural elements.
///
/// Counts files by maturity (planned/active/stable) and patterns by
/// confidence (planned/verified), both project-wide and per-element.
pub fn aggregate_health(ir: &ArchitectureIR) -> HealthReport {
    let annotated = ir.annotated_dirs();
    let mut report = HealthReport::default();

    report.total_elements = annotated.len();
    report.container_count = annotated.iter().filter(|d| d.c4_level == Some(C4Level::Container)).count();
    report.component_count = annotated.iter().filter(|d| d.c4_level == Some(C4Level::Component)).count();

    for dir in &annotated {
        let c4_level = dir.c4_level.unwrap_or(C4Level::Unknown);
        let pattern = dir.pattern.as_deref().unwrap_or("--").to_string();
        let pattern_status = dir.pattern_status.unwrap_or_default();

        // Only count files that have health attributes (same as old ModuleDoc filtering)
        let attributed_files: Vec<_> = dir.files.iter()
            .filter(|f| f.health.is_some())
            .collect();

        let mut elem = ElementHealth {
            name: dir.path.clone(),
            c4_level: c4_level.to_string(),
            file_count: attributed_files.len(),
            files_planned: 0,
            files_active: 0,
            files_stable: 0,
            pattern: pattern.clone(),
            pattern_confidence: pattern_status.to_string(),
        };

        for file in &attributed_files {
            match file.health.unwrap_or_default() {
                HealthStatus::Planned => {
                    report.files_planned += 1;
                    elem.files_planned += 1;
                }
                HealthStatus::Active => {
                    report.files_active += 1;
                    elem.files_active += 1;
                }
                HealthStatus::Stable => {
                    report.files_stable += 1;
                    elem.files_stable += 1;
                }
            }
        }

        report.total_files += attributed_files.len();

        if pattern != "--" && !pattern.is_empty() {
            report.patterns_total += 1;
            match pattern_status {
                PatternStatus::Planned => report.patterns_planned += 1,
                PatternStatus::Verified => report.patterns_verified += 1,
            }
        }

        report.per_element.push(elem);
    }

    report
}

/// Format a health report as human-readable text.
pub fn format_health_report(report: &HealthReport) -> String {
    let mut out = String::new();

    out.push_str("Architecture Health Report\n");
    out.push_str("==========================\n");
    out.push_str(&format!(
        "Elements:    {} total ({} containers, {} components)\n",
        report.total_elements, report.container_count, report.component_count
    ));
    out.push_str(&format!("Files:       {} total\n", report.total_files));

    if report.total_files > 0 {
        out.push_str(&format!(
            "  planned:   {} ({:.1}%)\n",
            report.files_planned,
            percent(report.files_planned, report.total_files)
        ));
        out.push_str(&format!(
            "  active:    {} ({:.1}%)\n",
            report.files_active,
            percent(report.files_active, report.total_files)
        ));
        out.push_str(&format!(
            "  stable:    {} ({:.1}%)\n",
            report.files_stable,
            percent(report.files_stable, report.total_files)
        ));
    }

    out.push_str(&format!("Patterns:    {} assigned\n", report.patterns_total));
    if report.patterns_total > 0 {
        out.push_str(&format!(
            "  planned:   {} ({:.1}%)\n",
            report.patterns_planned,
            percent(report.patterns_planned, report.patterns_total)
        ));
        out.push_str(&format!(
            "  verified:  {} ({:.1}%)\n",
            report.patterns_verified,
            percent(report.patterns_verified, report.patterns_total)
        ));
    }

    out
}

fn percent(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}
