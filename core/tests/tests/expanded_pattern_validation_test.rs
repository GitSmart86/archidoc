use archidoc_tests::ArchitectureDsl;

// =============================================================================
// Builder pattern verification
// =============================================================================

#[test]
fn should_verify_builder_pattern_when_fluent_interface_found() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.config",
        "purpose: Configuration assembly",
        "design_pattern: Builder",
    ]);
    arch.place_code_file(
        "bus.config",
        "settings.rs",
        r#"
            pub struct Config { name: String, port: u16 }
            impl Config {
                pub fn new() -> Self { Config { name: String::new(), port: 0 } }
                pub fn name(mut self, name: &str) -> Self { self.name = name.to_string(); self }
                pub fn port(mut self, port: u16) -> Self { self.port = port; self }
                pub fn build(self) -> Config { self }
            }
        "#,
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.config", "confidence: verified"]);
}

#[test]
fn should_keep_builder_planned_when_no_fluent_interface() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.config",
        "purpose: Configuration",
        "design_pattern: Builder",
    ]);
    arch.place_code_file(
        "bus.config",
        "settings.rs",
        r#"
            pub struct Config { name: String }
            impl Config {
                pub fn name(&self) -> &str { &self.name }
            }
        "#,
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.config", "confidence: planned"]);
}

// =============================================================================
// Factory pattern verification
// =============================================================================

#[test]
fn should_verify_factory_pattern_when_trait_object_returned() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.shapes",
        "purpose: Shape creation",
        "design_pattern: Factory",
    ]);
    arch.place_code_file(
        "bus.shapes",
        "creator.rs",
        r#"
            pub trait Shape { fn area(&self) -> f64; }
            pub fn create(kind: &str) -> Box<dyn Shape> {
                todo!()
            }
        "#,
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.shapes", "confidence: verified"]);
}

#[test]
fn should_keep_factory_planned_when_only_concrete_returns() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.shapes",
        "purpose: Shape creation",
        "design_pattern: Factory",
    ]);
    arch.place_code_file(
        "bus.shapes",
        "creator.rs",
        r#"
            pub struct Circle { radius: f64 }
            impl Circle {
                pub fn new(radius: f64) -> Circle { Circle { radius } }
            }
        "#,
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.shapes", "confidence: planned"]);
}

// =============================================================================
// Adapter pattern verification
// =============================================================================

#[test]
fn should_verify_adapter_pattern_when_wrapper_implements_trait() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.logging",
        "purpose: Log output adaptation",
        "design_pattern: Adapter",
    ]);
    arch.place_code_file(
        "bus.logging",
        "file_adapter.rs",
        r#"
            pub trait Logger { fn log(&self, msg: &str); }
            pub struct FileLogger { inner: std::fs::File }
            impl Logger for FileLogger {
                fn log(&self, msg: &str) { }
            }
        "#,
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.logging", "confidence: verified"]);
}

#[test]
fn should_keep_adapter_planned_when_no_trait_implementation() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.logging",
        "purpose: Logging utilities",
        "design_pattern: Adapter",
    ]);
    arch.place_code_file(
        "bus.logging",
        "helpers.rs",
        r#"
            pub struct FileLogger { path: String, level: u8, buffer: Vec<u8> }
        "#,
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.logging", "confidence: planned"]);
}

// =============================================================================
// Decorator pattern verification
// =============================================================================

#[test]
fn should_verify_decorator_pattern_when_wrapping_trait_object() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.middleware",
        "purpose: Request processing decoration",
        "design_pattern: Decorator",
    ]);
    arch.place_code_file(
        "bus.middleware",
        "timestamp.rs",
        r#"
            pub trait Logger { fn log(&self, msg: &str); }
            pub struct TimestampLogger { inner: Box<dyn Logger> }
            impl Logger for TimestampLogger {
                fn log(&self, msg: &str) { self.inner.log(msg); }
            }
        "#,
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.middleware", "confidence: verified"]);
}

#[test]
fn should_keep_decorator_planned_when_no_trait_wrapping() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.middleware",
        "purpose: Request utilities",
        "design_pattern: Decorator",
    ]);
    arch.place_code_file(
        "bus.middleware",
        "wrapper.rs",
        r#"
            pub struct Wrapper { data: Vec<u8> }
            impl Wrapper {
                pub fn wrap(data: Vec<u8>) -> Self { Wrapper { data } }
            }
        "#,
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.middleware", "confidence: planned"]);
}

// =============================================================================
// Singleton pattern verification
// =============================================================================

#[test]
fn should_verify_singleton_pattern_when_static_initialization_found() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.pool",
        "purpose: Connection pool management",
        "design_pattern: Singleton",
    ]);
    arch.place_code_file(
        "bus.pool",
        "instance.rs",
        r#"
            use std::sync::OnceLock;
            static INSTANCE: OnceLock<String> = OnceLock::new();
            pub fn get_instance() -> &'static str {
                INSTANCE.get_or_init(|| String::from("pool"))
            }
        "#,
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.pool", "confidence: verified"]);
}

#[test]
fn should_keep_singleton_planned_when_no_static_initialization() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.pool",
        "purpose: Pool management",
        "design_pattern: Singleton",
    ]);
    arch.place_code_file(
        "bus.pool",
        "manager.rs",
        r#"
            pub struct Config { port: u16 }
            impl Config {
                pub fn new(port: u16) -> Self { Config { port } }
            }
        "#,
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.pool", "confidence: planned"]);
}

// =============================================================================
// Command pattern verification
// =============================================================================

#[test]
fn should_verify_command_pattern_when_execute_trait_found() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.actions",
        "purpose: Undoable operations",
        "design_pattern: Command",
    ]);
    arch.place_code_file(
        "bus.actions",
        "commands.rs",
        r#"
            pub trait Command {
                fn execute(&self);
                fn undo(&self);
            }
        "#,
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.actions", "confidence: verified"]);
}

#[test]
fn should_keep_command_planned_when_no_execute_method() {
    let mut arch = ArchitectureDsl::setup();

    arch.annotate_component(&[
        "name: bus.actions",
        "purpose: Calculations",
        "design_pattern: Command",
    ]);
    arch.place_code_file(
        "bus.actions",
        "calc.rs",
        r#"
            pub trait Calculator {
                fn calculate(&self, values: &[f64]) -> f64;
            }
        "#,
    );

    arch.compile();
    arch.verify_patterns();
    arch.assert_pattern_confidence(&["name: bus.actions", "confidence: planned"]);
}
