//! Component Relationships — Dependency and Communication Tests
//!
//! Given architecture elements with declared dependencies,
//! When compiled, the diagrams and documentation accurately
//! reflect how components communicate.

use archidoc_tests::ArchitectureDsl;

// =========================================================================
// Dependencies between containers
// =========================================================================

#[test]
fn declared_dependencies_appear_in_diagrams() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: engine",
        "purpose: Trade execution engine",
    ]);
    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
    ]);
    arch.declare_dependency(&[
        "from: engine",
        "to: bus",
        "label: Routes commands",
        "protocol: crossbeam",
    ]);
    arch.compile();

    arch.assert_dependency(&[
        "from: engine",
        "to: bus",
        "label: Routes commands",
        "protocol: crossbeam",
    ]);
    arch.assert_diagram_shows_dependency(&["from: engine", "to: bus"]);
}

// =========================================================================
// Multiple dependencies from one element
// =========================================================================

#[test]
fn element_with_multiple_dependencies_shows_all_arrows() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.annotate_container(&[
        "name: agents_internal",
        "purpose: Internal trading agents",
    ]);
    arch.annotate_container(&[
        "name: agents_external",
        "purpose: External broker connections",
    ]);
    arch.declare_dependency(&[
        "from: bus",
        "to: agents_internal",
        "label: Processed data",
        "protocol: crossbeam",
    ]);
    arch.declare_dependency(&[
        "from: bus",
        "to: agents_external",
        "label: Market feed",
        "protocol: crossbeam",
    ]);
    arch.compile();

    arch.assert_dependency(&["from: bus", "to: agents_internal"]);
    arch.assert_dependency(&["from: bus", "to: agents_external"]);
    arch.assert_diagram_shows_dependency(&["from: bus", "to: agents_internal"]);
    arch.assert_diagram_shows_dependency(&["from: bus", "to: agents_external"]);
}

// =========================================================================
// Component-level dependencies
// =========================================================================

#[test]
fn component_dependencies_carry_protocol_details() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
    ]);
    arch.annotate_component(&[
        "name: bus.calc",
        "purpose: Indicator calculations",
        "design_pattern: Strategy",
    ]);
    arch.annotate_component(&[
        "name: bus.lanes",
        "purpose: Event routing lanes",
        "design_pattern: Observer",
    ]);
    arch.declare_dependency(&[
        "from: bus.calc",
        "to: bus.lanes",
        "label: Calculation results",
        "protocol: channel",
    ]);
    arch.compile();

    arch.assert_dependency(&[
        "from: bus.calc",
        "to: bus.lanes",
        "label: Calculation results",
        "protocol: channel",
    ]);
}
