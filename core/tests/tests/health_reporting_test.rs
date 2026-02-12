//! Health Reporting — Project-Wide Maturity and Confidence Tracking (B6)
//!
//! Given an architecture with file catalogs at various maturity levels,
//! When a health report is requested, the report accurately reflects
//! the project's implementation progress and pattern confidence.

use archidoc_tests::ArchitectureDsl;

// =========================================================================
// Basic health aggregation
// =========================================================================

#[test]
fn health_report_counts_files_by_maturity() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: lanes.rs",
        "design_pattern: Observer",
        "responsibility: Event routing",
        "maturity: active",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: store.rs",
        "design_pattern: Repository",
        "responsibility: Lock-free cache",
        "maturity: stable",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: future.rs",
        "responsibility: Future feature",
        "maturity: planned",
    ]);
    arch.compile();

    arch.assert_health_total_files(&["count: 3"]);
    arch.assert_health_file_count(&["maturity: planned", "count: 1"]);
    arch.assert_health_file_count(&["maturity: active", "count: 1"]);
    arch.assert_health_file_count(&["maturity: stable", "count: 1"]);
}

// =========================================================================
// Pattern confidence aggregation
// =========================================================================

#[test]
fn health_report_counts_patterns_by_confidence() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.set_pattern_confidence(&["name: bus", "confidence: verified"]);

    arch.annotate_container(&[
        "name: engine",
        "purpose: Trade execution engine",
        "design_pattern: Facade",
    ]);
    // engine defaults to planned confidence

    arch.annotate_container(&[
        "name: store",
        "purpose: Data persistence",
    ]);
    // store has no pattern at all

    arch.compile();

    arch.assert_health_pattern_count(&["confidence: verified", "count: 1"]);
    arch.assert_health_pattern_count(&["confidence: planned", "count: 1"]);
}

// =========================================================================
// Cross-element aggregation
// =========================================================================

#[test]
fn health_report_aggregates_across_multiple_elements() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: lanes.rs",
        "responsibility: Event routing",
        "maturity: stable",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: calc.rs",
        "responsibility: Calculations",
        "maturity: active",
    ]);

    arch.annotate_container(&[
        "name: engine",
        "purpose: Trade execution engine",
        "design_pattern: Facade",
    ]);
    arch.catalog_file(&[
        "element: engine",
        "file: runner.rs",
        "responsibility: Trade execution",
        "maturity: stable",
    ]);
    arch.catalog_file(&[
        "element: engine",
        "file: planner.rs",
        "responsibility: Trade planning",
        "maturity: planned",
    ]);
    arch.compile();

    arch.assert_health_total_files(&["count: 4"]);
    arch.assert_health_file_count(&["maturity: planned", "count: 1"]);
    arch.assert_health_file_count(&["maturity: active", "count: 1"]);
    arch.assert_health_file_count(&["maturity: stable", "count: 2"]);
}
