//! Architecture Compilation — End-to-End Behavioral Tests
//!
//! Given annotated source files describing an architecture,
//! When compiled, the tool produces documentation and diagrams
//! that accurately reflect the declared architecture.

use archidoc_tests::ArchitectureDsl;

// =========================================================================
// Single container — the simplest useful architecture
// =========================================================================

#[test]
fn annotated_container_produces_documentation_and_diagram() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
        "design_pattern: Mediator",
    ]);
    arch.compile();

    arch.assert_architecture_produced();
    arch.assert_architecture_contains(&["contains: Central messaging backbone"]);
    arch.assert_diagram_shows_container(&["name: bus"]);
    arch.assert_element_level(&["name: bus", "level: container"]);
    arch.assert_design_pattern(&["name: bus", "design_pattern: Mediator"]);
}

// =========================================================================
// Container with nested components
// =========================================================================

#[test]
fn components_appear_inside_their_parent_container() {
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
    arch.annotate_component(&[
        "name: bus.lanes",
        "purpose: Event routing lanes",
        "design_pattern: Observer",
    ]);
    arch.compile();

    arch.assert_containment(&["name: bus.calc", "inside: bus"]);
    arch.assert_containment(&["name: bus.lanes", "inside: bus"]);
    arch.assert_diagram_shows_component(&["name: bus.calc", "inside: bus"]);
    arch.assert_diagram_shows_component(&["name: bus.lanes", "inside: bus"]);
    arch.assert_top_level(&["name: bus"]);
    arch.assert_total_elements(&["count: 3"]);
}

// =========================================================================
// Architecture index lists all elements
// =========================================================================

#[test]
fn compiled_architecture_produces_navigable_index() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: engine",
        "purpose: Trade execution engine",
    ]);
    arch.annotate_container(&[
        "name: bus",
        "purpose: Central messaging backbone",
    ]);
    arch.compile();

    arch.assert_architecture_produced();
    arch.assert_index_lists(&["name: engine"]);
    arch.assert_index_lists(&["name: bus"]);
}
