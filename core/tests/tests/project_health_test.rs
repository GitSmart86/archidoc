//! Project Health — File Catalogs, Maturity, and Pattern Confidence
//!
//! Given an architecture with file catalogs and maturity indicators,
//! When compiled, the documentation reflects the health and maturity
//! of each element accurately.

use archidoc_tests::ArchitectureDsl;

// =========================================================================
// File catalog tracks element composition
// =========================================================================

#[test]
fn file_catalog_tracks_element_composition() {
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
        "file: calc.rs",
        "design_pattern: Strategy",
        "responsibility: Indicator calculations",
        "maturity: stable",
    ]);
    arch.compile();

    arch.assert_catalog_entry(&[
        "element: bus",
        "file: lanes.rs",
        "design_pattern: Observer",
        "responsibility: Event routing",
        "maturity: active",
    ]);
    arch.assert_catalog_entry(&[
        "element: bus",
        "file: calc.rs",
        "design_pattern: Strategy",
        "responsibility: Indicator calculations",
        "maturity: stable",
    ]);
    arch.assert_catalog_size(&["element: bus", "count: 2"]);
}

// =========================================================================
// Pattern confidence indicates maturity of design decisions
// =========================================================================

#[test]
fn pattern_confidence_distinguishes_planned_from_verified() {
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
    // engine has no confidence override — defaults to planned
    arch.compile();

    arch.assert_pattern_confidence(&["name: bus", "confidence: verified"]);
    arch.assert_pattern_confidence(&["name: engine", "confidence: planned"]);
}

// =========================================================================
// Health maturity reflects implementation progress
// =========================================================================

#[test]
fn health_maturity_reflects_implementation_progress() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: planned_feature.rs",
        "responsibility: Future work",
        "maturity: planned",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: active_work.rs",
        "responsibility: In development",
        "maturity: active",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: stable_core.rs",
        "responsibility: Core logic",
        "maturity: stable",
    ]);
    arch.compile();

    arch.assert_catalog_entry(&[
        "element: bus",
        "file: planned_feature.rs",
        "maturity: planned",
    ]);
    arch.assert_catalog_entry(&[
        "element: bus",
        "file: active_work.rs",
        "maturity: active",
    ]);
    arch.assert_catalog_entry(&[
        "element: bus",
        "file: stable_core.rs",
        "maturity: stable",
    ]);
}

// =========================================================================
// Catalog entries can include pattern confidence
// =========================================================================

#[test]
fn catalog_entries_can_include_pattern_confidence() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: lanes.rs",
        "design_pattern: Observer",
        "confidence: verified",
        "responsibility: Event routing",
        "maturity: stable",
    ]);
    arch.compile();

    arch.assert_catalog_entry(&[
        "element: bus",
        "file: lanes.rs",
        "design_pattern: Observer (verified)",
        "responsibility: Event routing",
        "maturity: stable",
    ]);
}
