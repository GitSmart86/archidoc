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

// =========================================================================
// Short-name resolution in Mermaid diagrams
// =========================================================================

#[test]
fn container_dependency_short_name_resolves_to_full_path_in_diagram() {
    // Regression: @c4 uses targets like "session_store" must resolve to the
    // full module path "app.backend.session_store" so the Mermaid Rel() node
    // ID matches the Container() declaration ID.
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: app.frontend",
        "purpose: User-facing views",
    ]);
    arch.annotate_container(&[
        "name: app.backend.session_store",
        "purpose: SQLite persistence layer",
    ]);
    // Dependency uses the short name "session_store", not the full path.
    arch.declare_dependency(&[
        "from: app.frontend",
        "to: session_store",
        "label: Settings CRUD",
        "protocol: IPC",
    ]);
    arch.compile();

    // The diagram Rel() must use the full-path-derived ID, not the short name.
    arch.assert_diagram_shows_dependency(&[
        "from: app.frontend",
        "to: app.backend.session_store",
    ]);
}

#[test]
fn component_dependency_short_name_resolves_to_full_path_in_diagram() {
    // Same regression at component level: short-name targets in @c4 uses
    // must resolve to the full module path for Mermaid Rel() node IDs.
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: app.backend",
        "purpose: Backend services",
    ]);
    arch.annotate_component(&[
        "name: app.backend.capture",
        "purpose: Screen capture",
        "design_pattern: Strategy",
    ]);
    arch.annotate_component(&[
        "name: app.backend.store",
        "purpose: Data persistence",
        "design_pattern: Factory",
    ]);
    // Dependency uses short name "store"
    arch.declare_dependency(&[
        "from: app.backend.capture",
        "to: store",
        "label: Persists captures",
        "protocol: rusqlite",
    ]);
    arch.compile();

    arch.assert_diagram_shows_dependency(&[
        "from: app.backend.capture",
        "to: app.backend.store",
    ]);
}

#[test]
fn dependency_with_qualified_path_works_in_diagram() {
    // When the full qualified path is used in @c4 uses, it should also work.
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: sys.api",
        "purpose: REST API gateway",
    ]);
    arch.annotate_container(&[
        "name: sys.db",
        "purpose: Database layer",
    ]);
    arch.declare_dependency(&[
        "from: sys.api",
        "to: sys.db",
        "label: Queries",
        "protocol: SQL",
    ]);
    arch.compile();

    arch.assert_diagram_shows_dependency(&["from: sys.api", "to: sys.db"]);
}

#[test]
fn ambiguous_short_name_falls_back_to_literal_in_diagram() {
    // When two modules share the same short name (e.g., "types" appears in
    // both "app.frontend.types" and "app.backend.types"), the short name is
    // ambiguous and the target should be used literally (not resolved).
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: app.frontend",
        "purpose: User-facing views",
    ]);
    arch.annotate_container(&[
        "name: app.backend",
        "purpose: Backend services",
    ]);
    arch.annotate_component(&[
        "name: app.frontend.types",
        "purpose: Frontend domain types",
    ]);
    arch.annotate_component(&[
        "name: app.backend.types",
        "purpose: Backend domain types",
    ]);
    // Use the qualified path since "types" is ambiguous
    arch.declare_dependency(&[
        "from: app.frontend",
        "to: app.backend.types",
        "label: Shared types",
        "protocol: import",
    ]);
    arch.compile();

    arch.assert_diagram_shows_dependency(&[
        "from: app.frontend",
        "to: app.backend.types",
    ]);
}

#[test]
fn undeclared_sub_component_target_collapses_to_nearest_declared_parent() {
    // Regression: archidoc-ts auto-discovers import relationships to files
    // that aren't declared as @c4 components (e.g. stub drivers). The Mermaid
    // generator must collapse these to the nearest declared parent node,
    // not emit a Rel() to a non-existent node.
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: app.tests",
        "purpose: Test infrastructure",
    ]);
    arch.annotate_component(&[
        "name: app.tests.dsl",
        "purpose: DSL functions",
        "design_pattern: Facade",
    ]);
    arch.annotate_component(&[
        "name: app.tests.drivers",
        "purpose: Protocol drivers",
        "design_pattern: Facade",
    ]);
    // This target is NOT a declared component — it's a sub-file.
    // The Mermaid Rel should collapse to "app.tests.drivers".
    arch.declare_dependency(&[
        "from: app.tests.dsl",
        "to: app.tests.drivers.stubs.stubFoo",
        "label: imports",
        "protocol: ES module",
    ]);
    arch.compile();

    // The collapsed Rel should point to the nearest declared parent.
    arch.assert_diagram_shows_dependency(&[
        "from: app.tests.dsl",
        "to: app.tests.drivers",
    ]);
}

#[test]
fn multiple_undeclared_sub_targets_deduplicate_after_collapse() {
    // When several Rel targets under the same parent collapse to the
    // same declared node, only one Rel line should appear in the diagram.
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_container(&[
        "name: sys.core",
        "purpose: Core system",
    ]);
    arch.annotate_component(&[
        "name: sys.core.api",
        "purpose: API layer",
        "design_pattern: Facade",
    ]);
    arch.annotate_component(&[
        "name: sys.core.drivers",
        "purpose: Driver layer",
        "design_pattern: Facade",
    ]);
    // Three undeclared sub-targets, all under sys.core.drivers
    arch.declare_dependency(&[
        "from: sys.core.api",
        "to: sys.core.drivers.stubs.alpha",
        "label: imports",
        "protocol: ES module",
    ]);
    arch.declare_dependency(&[
        "from: sys.core.api",
        "to: sys.core.drivers.stubs.beta",
        "label: imports",
        "protocol: ES module",
    ]);
    arch.declare_dependency(&[
        "from: sys.core.api",
        "to: sys.core.drivers.stubs.gamma",
        "label: imports",
        "protocol: ES module",
    ]);
    arch.compile();

    // All three should collapse to one relationship to sys.core.drivers
    arch.assert_diagram_shows_dependency(&[
        "from: sys.core.api",
        "to: sys.core.drivers",
    ]);

    // And the diagram should NOT contain the undeclared stubs
    arch.assert_architecture_contains(&[
        "contains: Rel(sys_core_api, sys_core_drivers",
    ]);
}
