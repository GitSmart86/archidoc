//! Phase D: File-Based IR Consumption (D10)
//!
//! These tests verify that architecture documentation can be produced
//! from a JSON IR file on disk — the `--from-json-file` path.
//! This proves the full file-based pipeline: annotate → compile →
//! emit IR → write to file → read from file → regenerate docs.

use archidoc_tests::ArchitectureDsl;

/// Documentation can be regenerated from an IR file on disk.
#[test]
fn documentation_regenerated_from_ir_file() {
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

    // Emit IR, write to file, then regenerate from that file
    arch.emit_ir();
    arch.write_ir_to_file();
    arch.compile_from_ir_file();

    arch.assert_documentation_exists(&["name: bus"]);
    arch.assert_documentation_exists(&["name: bus.calc"]);
    arch.assert_diagram_shows_container(&["name: bus"]);
    arch.assert_diagram_shows_component(&["name: bus.calc", "inside: bus"]);
}

/// Relationships survive the file-based IR round trip.
#[test]
fn relationships_survive_file_based_ir_round_trip() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.annotate_container(&[
        "name: agents",
        "purpose: Agent execution framework",
    ]);
    arch.declare_dependency(&[
        "from: bus",
        "to: agents",
        "label: Routes processed data",
        "protocol: crossbeam",
    ]);
    arch.compile();

    arch.emit_ir();
    arch.write_ir_to_file();
    arch.compile_from_ir_file();

    arch.assert_dependency(&[
        "from: bus",
        "to: agents",
        "label: Routes processed data",
        "protocol: crossbeam",
    ]);
    arch.assert_diagram_shows_dependency(&["from: bus", "to: agents"]);
}

/// File catalog entries survive the file-based IR round trip.
#[test]
fn file_catalog_survives_file_based_ir_round_trip() {
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
    arch.compile();

    arch.emit_ir();
    arch.write_ir_to_file();
    arch.compile_from_ir_file();

    arch.assert_catalog_entry(&[
        "element: bus",
        "file: lanes.rs",
        "design_pattern: Observer",
        "responsibility: Event routing",
        "maturity: active",
    ]);
}
