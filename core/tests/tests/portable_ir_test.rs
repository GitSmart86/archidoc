//! Phase D: Portable IR — Cross-language intermediate representation
//!
//! These tests verify that architecture annotations can be serialized
//! to a portable JSON format and consumed by the core generator
//! independently of any language adapter.

use archidoc_tests::ArchitectureDsl;

/// An adapter can serialize compiled architecture to portable JSON.
#[test]
fn compiled_architecture_produces_portable_ir() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.compile();

    arch.emit_ir();
    arch.assert_ir_contains_element(&["name: bus", "level: container"]);
}

/// Serializing then deserializing preserves all architecture information —
/// levels, patterns, dependencies, catalog entries, and relationships.
#[test]
fn portable_ir_preserves_architecture_fidelity() {
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
    arch.compile();

    arch.emit_ir();
    arch.assert_ir_round_trip_preserves_fidelity();
}

/// The core generator can produce documentation from portable IR alone,
/// without access to the original source code.
#[test]
fn documentation_can_be_generated_from_ir_alone() {
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
    arch.compile();

    // Emit IR, then regenerate from IR only (no source code)
    arch.emit_ir();
    arch.compile_from_ir();

    arch.assert_architecture_produced();
    arch.assert_architecture_contains(&["contains: bus"]);
    arch.assert_architecture_contains(&["contains: bus.calc"]);
    arch.assert_diagram_shows_container(&["name: bus"]);
    arch.assert_diagram_shows_component(&["name: bus.calc", "inside: bus"]);
}

/// Well-formed IR passes schema validation.
#[test]
fn well_formed_ir_passes_schema_validation() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.compile();

    arch.emit_ir();
    arch.assert_ir_schema_valid();
}
