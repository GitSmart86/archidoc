use archidoc_types::{
    C4Level, ElementHealth, HealthReport, HealthStatus, ModuleDoc, PatternStatus,
};

/// Aggregate health across all architectural elements.
///
/// Counts files by maturity (planned/active/stable) and patterns by
/// confidence (planned/verified), both project-wide and per-element.
pub fn aggregate_health(docs: &[ModuleDoc]) -> HealthReport {
    let mut report = HealthReport::default();

    report.total_elements = docs.len();
    report.container_count = docs.iter().filter(|d| d.c4_level == C4Level::Container).count();
    report.component_count = docs.iter().filter(|d| d.c4_level == C4Level::Component).count();

    for doc in docs {
        let mut elem = ElementHealth {
            name: doc.module_path.clone(),
            c4_level: doc.c4_level.to_string(),
            file_count: doc.files.len(),
            files_planned: 0,
            files_active: 0,
            files_stable: 0,
            pattern: doc.pattern.clone(),
            pattern_confidence: doc.pattern_status.to_string(),
        };

        for file in &doc.files {
            match file.health {
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

        report.total_files += doc.files.len();

        if doc.pattern != "--" && !doc.pattern.is_empty() {
            report.patterns_total += 1;
            match doc.pattern_status {
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
