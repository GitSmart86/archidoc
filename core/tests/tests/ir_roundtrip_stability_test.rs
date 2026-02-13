//! Phase D: Double Roundtrip Stability (D11)
//!
//! These tests prove that the JSON IR is idempotent:
//! IR → core → IR produces identical output. This guarantees
//! that no data is lost or mutated during deserialization/reserialization.

use archidoc_tests::ArchitectureDsl;

/// A single container round-trips identically through IR.
#[test]
fn single_container_ir_is_idempotent() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.compile();

    arch.emit_ir();
    arch.assert_ir_idempotent();
}

/// A complex architecture with containers, components, dependencies,
/// and file catalogs round-trips identically through IR.
#[test]
fn complex_architecture_ir_is_idempotent() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.annotate_component(&[
        "name: bus.calc",
        "purpose: Indicator calculations",
        "design_pattern: Strategy",
    ]);
    arch.annotate_container(&[
        "name: agents",
        "purpose: Agent execution framework",
        "design_pattern: Observer",
    ]);
    arch.declare_dependency(&[
        "from: bus",
        "to: agents",
        "label: Routes processed data",
        "protocol: crossbeam",
    ]);
    arch.catalog_file(&[
        "element: bus",
        "file: lanes.rs",
        "design_pattern: Observer",
        "responsibility: Event routing",
        "maturity: active",
    ]);
    arch.catalog_file(&[
        "element: bus.calc",
        "file: indicators.rs",
        "design_pattern: Strategy",
        "responsibility: Technical indicators",
        "maturity: stable",
    ]);
    arch.compile();

    arch.emit_ir();
    arch.assert_ir_idempotent();
}

/// An architecture with no patterns or dependencies round-trips identically.
#[test]
fn minimal_architecture_ir_is_idempotent() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: core",
        "purpose: Core domain logic",
    ]);
    arch.compile();

    arch.emit_ir();
    arch.assert_ir_idempotent();
}
