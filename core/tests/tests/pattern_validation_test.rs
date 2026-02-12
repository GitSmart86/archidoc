use archidoc_tests::ArchitectureDsl;

// =============================================================================
// H2: Strategy pattern heuristic
// =============================================================================

#[test]
fn should_verify_strategy_pattern_when_trait_found() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.calc",
        "purpose: Pluggable indicator calculations",
        "design_pattern: Strategy",
    ]);
    arch.place_code_file(
        "bus.calc",
        "indicators.rs",
        "pub trait IndicatorCalc { fn calculate(&self, prices: &[f64]) -> f64; }",
    );

    arch.compile();
    arch.assert_pattern_confidence(&["name: bus.calc", "confidence: planned"]);

    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.calc", "confidence: verified"]);
}

#[test]
fn should_keep_strategy_planned_when_no_trait() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.calc",
        "purpose: Indicator calculations",
        "design_pattern: Strategy",
    ]);
    arch.place_code_file(
        "bus.calc",
        "indicators.rs",
        "pub struct SimpleCalc;\nimpl SimpleCalc { pub fn calc(&self) -> f64 { 0.0 } }",
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.calc", "confidence: planned"]);
}

// =============================================================================
// H3: Facade pattern heuristic
// =============================================================================

#[test]
fn should_verify_facade_pattern_when_pub_use_found() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.api",
        "purpose: Public API surface",
        "design_pattern: Facade",
    ]);
    arch.place_code_file(
        "bus.api",
        "surface.rs",
        "pub use crate::calc::Calculator;\npub use crate::store::DataStore;",
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.api", "confidence: verified"]);
}

#[test]
fn should_keep_facade_planned_when_no_reexports() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.api",
        "purpose: Public API",
        "design_pattern: Facade",
    ]);
    arch.place_code_file(
        "bus.api",
        "surface.rs",
        "struct Internal { data: Vec<u8> }",
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.api", "confidence: planned"]);
}

// =============================================================================
// H1: Observer pattern heuristic
// =============================================================================

#[test]
fn should_verify_observer_pattern_when_channel_found() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.events",
        "purpose: Event distribution",
        "design_pattern: Observer",
    ]);
    arch.place_code_file(
        "bus.events",
        "dispatcher.rs",
        "use std::sync::mpsc::Sender;\nuse std::sync::mpsc::Receiver;\npub fn create() -> (mpsc::Sender<i32>, mpsc::Receiver<i32>) { std::sync::mpsc::channel() }",
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.events", "confidence: verified"]);
}

#[test]
fn should_verify_observer_pattern_when_callback_trait_found() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.events",
        "purpose: Event distribution",
        "design_pattern: Observer",
    ]);
    arch.place_code_file(
        "bus.events",
        "bus.rs",
        "pub trait EventBus {\n    fn subscribe(&mut self, handler: Box<dyn Fn(i32)>);\n    fn notify(&self, event: i32);\n}",
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.events", "confidence: verified"]);
}

#[test]
fn should_keep_observer_planned_when_no_channels_or_callbacks() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.events",
        "purpose: Event distribution",
        "design_pattern: Observer",
    ]);
    arch.place_code_file(
        "bus.events",
        "logger.rs",
        "pub struct Logger { path: String }\nimpl Logger { pub fn log(&self, msg: &str) { println!(\"{}\", msg); } }",
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.events", "confidence: planned"]);
}

// =============================================================================
// H7: Auto-promote skips already-verified and non-verifiable patterns
// =============================================================================

#[test]
fn should_not_re_verify_already_verified_pattern() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.calc",
        "purpose: Calculations",
        "design_pattern: Strategy",
    ]);
    arch.set_pattern_confidence(&["name: bus.calc", "confidence: verified"]);
    arch.place_code_file(
        "bus.calc",
        "traits.rs",
        "pub trait Algo { fn run(&self); }",
    );

    arch.compile();
    arch.assert_pattern_confidence(&["name: bus.calc", "confidence: verified"]);

    // Verify again — should remain verified (not regress)
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.calc", "confidence: verified"]);
}

#[test]
fn should_skip_patterns_without_heuristics() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.core",
        "purpose: Core orchestration",
        "design_pattern: Mediator",
    ]);

    arch.compile();
    arch.verify_patterns();
    // Mediator has no heuristic — stays planned
    arch.assert_pattern_confidence(&["name: bus.core", "confidence: planned"]);
}

// =============================================================================
// H4: Fitness function — all_strategy_modules_define_a_trait
// =============================================================================

#[test]
fn fitness_should_pass_when_all_strategy_modules_have_traits() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.calc",
        "purpose: Calculations",
        "design_pattern: Strategy",
    ]);
    arch.place_code_file(
        "bus.calc",
        "algo.rs",
        "pub trait CalcAlgorithm { fn compute(&self) -> f64; }",
    );

    arch.annotate_component(&[
        "name: bus.routing",
        "purpose: Message routing",
        "design_pattern: Strategy",
    ]);
    arch.place_code_file(
        "bus.routing",
        "router.rs",
        "pub trait Router { fn route(&self, msg: &str) -> String; }",
    );

    arch.compile();
    arch.assert_fitness_passes(&["fitness: all_strategy_modules_define_a_trait"]);
}

#[test]
fn fitness_should_fail_when_strategy_module_lacks_trait() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.calc",
        "purpose: Calculations",
        "design_pattern: Strategy",
    ]);
    arch.place_code_file(
        "bus.calc",
        "algo.rs",
        "pub struct HardcodedCalc;\nimpl HardcodedCalc { pub fn compute(&self) -> f64 { 42.0 } }",
    );

    arch.compile();
    arch.assert_fitness_fails(&[
        "fitness: all_strategy_modules_define_a_trait",
        "failing_module: bus.calc",
    ]);
}

// =============================================================================
// H5: Fitness function — all_facade_modules_reexport_submodules
// =============================================================================

#[test]
fn fitness_should_pass_when_all_facade_modules_reexport() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.api",
        "purpose: Public API",
        "design_pattern: Facade",
    ]);
    arch.place_code_file(
        "bus.api",
        "exports.rs",
        "pub use crate::internal::Widget;\npub use crate::internal::Config;",
    );

    arch.compile();
    arch.assert_fitness_passes(&["fitness: all_facade_modules_reexport_submodules"]);
}

#[test]
fn fitness_should_fail_when_facade_module_has_no_reexports() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.api",
        "purpose: Public API",
        "design_pattern: Facade",
    ]);
    arch.place_code_file(
        "bus.api",
        "internal.rs",
        "struct Private { data: i32 }",
    );

    arch.compile();
    arch.assert_fitness_fails(&[
        "fitness: all_facade_modules_reexport_submodules",
        "failing_module: bus.api",
    ]);
}

// =============================================================================
// H6: Fitness function — all_observer_modules_have_channels_or_callbacks
// =============================================================================

#[test]
fn fitness_should_pass_when_all_observer_modules_have_channels() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.events",
        "purpose: Event system",
        "design_pattern: Observer",
    ]);
    arch.place_code_file(
        "bus.events",
        "channels.rs",
        "use std::sync::mpsc::Sender;\nuse std::sync::mpsc::Receiver;",
    );

    arch.compile();
    arch.assert_fitness_passes(&["fitness: all_observer_modules_have_channels_or_callbacks"]);
}

#[test]
fn fitness_should_fail_when_observer_module_has_no_channels() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.events",
        "purpose: Event system",
        "design_pattern: Observer",
    ]);
    arch.place_code_file(
        "bus.events",
        "plain.rs",
        "pub fn process(data: &[u8]) -> Vec<u8> { data.to_vec() }",
    );

    arch.compile();
    arch.assert_fitness_fails(&[
        "fitness: all_observer_modules_have_channels_or_callbacks",
        "failing_module: bus.events",
    ]);
}

// =============================================================================
// Edge cases — multi-file modules, empty directories, boundary conditions
// =============================================================================

#[test]
fn should_verify_when_only_one_of_multiple_files_has_evidence() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.calc",
        "purpose: Pluggable calculations",
        "design_pattern: Strategy",
    ]);
    // First file: no trait
    arch.place_code_file(
        "bus.calc",
        "helpers.rs",
        "pub fn add(a: f64, b: f64) -> f64 { a + b }",
    );
    // Second file: no trait
    arch.place_code_file(
        "bus.calc",
        "constants.rs",
        "pub const PI: f64 = 3.14159;",
    );
    // Third file: has a trait
    arch.place_code_file(
        "bus.calc",
        "algo.rs",
        "pub trait CalcAlgorithm { fn compute(&self, values: &[f64]) -> f64; }",
    );

    arch.compile();
    arch.verify_patterns();
    // Should verify — any file with evidence is enough
    arch.assert_pattern_confidence(&["name: bus.calc", "confidence: verified"]);
}

#[test]
fn should_stay_planned_when_module_directory_has_no_rs_files() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.calc",
        "purpose: Calculations",
        "design_pattern: Strategy",
    ]);
    // Don't place any code files — only the mod.rs from annotate exists

    arch.compile();
    arch.verify_patterns();
    // mod.rs contains //! annotations, not a trait definition
    arch.assert_pattern_confidence(&["name: bus.calc", "confidence: planned"]);
}

#[test]
fn should_verify_facade_with_pub_mod_only_no_pub_use() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.api",
        "purpose: Public API surface",
        "design_pattern: Facade",
    ]);
    arch.place_code_file(
        "bus.api",
        "entry.rs",
        "pub mod calc;\npub mod store;\npub mod config;",
    );

    arch.compile();
    arch.verify_patterns();
    // 3 pub mod declarations (>= 2 threshold) should verify
    arch.assert_pattern_confidence(&["name: bus.api", "confidence: verified"]);
}

#[test]
fn should_not_verify_facade_with_single_pub_mod() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.api",
        "purpose: Public API",
        "design_pattern: Facade",
    ]);
    arch.place_code_file(
        "bus.api",
        "entry.rs",
        "pub mod internal;",
    );

    arch.compile();
    arch.verify_patterns();
    // Only 1 pub mod — not enough for Facade (need >= 2 or any pub use)
    arch.assert_pattern_confidence(&["name: bus.api", "confidence: planned"]);
}
