//! Documentation Drift — Detecting Stale Documentation (B1)
//!
//! Given compiled architecture documentation, when the source annotations
//! change but the documentation is not regenerated, drift detection
//! identifies the stale files.

use archidoc_tests::ArchitectureDsl;

// =========================================================================
// No drift — docs are current
// =========================================================================

#[test]
fn no_drift_when_documentation_is_current() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.compile();

    // Documentation was just compiled — no drift expected
    arch.assert_no_drift();
}

// =========================================================================
// Drift detected — source changed after compilation
// =========================================================================

#[test]
fn drift_detected_when_source_changes_after_compilation() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.compile();

    // Simulate changing the source annotation without recompiling
    arch.modify_source_annotation(&[
        "name: bus",
        "purpose: CHANGED description that differs from compiled docs",
    ]);

    arch.assert_drift_detected();
}

// =========================================================================
// Multi-element drift
// =========================================================================

#[test]
fn drift_detected_across_multiple_elements() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
    ]);
    arch.annotate_container(&[
        "name: engine",
        "purpose: Trade execution engine",
    ]);
    arch.compile();

    // Change only one element
    arch.modify_source_annotation(&[
        "name: engine",
        "purpose: Completely rewritten engine purpose",
    ]);

    arch.assert_drift_detected();
}
