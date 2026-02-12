use std::collections::HashMap;

use crate::drivers::in_memory::InMemoryArchitectureDriver;
use crate::drivers::protocol_driver::ArchitectureDriver;
use crate::params::Params;

/// Unified DSL for architecture compilation tests.
///
/// Domain vocabulary throughout: annotate, compile, dependency, maturity,
/// confidence, catalog, responsibility. No parser or generator internals leak.
///
/// # Example
/// ```ignore
/// let mut arch = ArchitectureDsl::setup();
/// arch.annotate_container(&["name: bus", "purpose: Central messaging", "design_pattern: Mediator"]);
/// arch.compile();
/// arch.assert_diagram_shows_container(&["name: bus"]);
/// ```
pub struct ArchitectureDsl {
    driver: Box<dyn ArchitectureDriver>,
    /// Accumulated element definitions: name -> ElementSetup
    elements: HashMap<String, ElementSetup>,
    /// Pending file catalog entries
    catalog_entries: Vec<CatalogEntry>,
    /// Pending dependency declarations
    dependencies: Vec<DependencyDecl>,
    /// Pattern confidence overrides
    confidence_overrides: HashMap<String, String>,
}

struct ElementSetup {
    c4_level: String,
    purpose: String,
    design_pattern: String,
}

struct CatalogEntry {
    element: String,
    filename: String,
    design_pattern: String,
    responsibility: String,
    maturity: String,
}

struct DependencyDecl {
    from: String,
    to: String,
    label: String,
    protocol: String,
}

impl ArchitectureDsl {
    /// Setup with default in-memory driver.
    pub fn setup() -> Self {
        Self {
            driver: Box::new(InMemoryArchitectureDriver::new()),
            elements: HashMap::new(),
            catalog_entries: Vec::new(),
            dependencies: Vec::new(),
            confidence_overrides: HashMap::new(),
        }
    }

    // =========================================================================
    // Setup — annotate architecture
    // =========================================================================

    /// Annotate a container in the architecture.
    /// Format: "name: bus, purpose: Central messaging backbone, design_pattern: Mediator"
    pub fn annotate_container(&mut self, args: &[&str]) {
        let params = Params::parse(args);
        self.elements.insert(
            params.get("name"),
            ElementSetup {
                c4_level: "container".to_string(),
                purpose: params.get("purpose"),
                design_pattern: params.get_opt("design_pattern").unwrap_or_default(),
            },
        );
    }

    /// Annotate a component in the architecture.
    /// Format: "name: bus.calc, purpose: Indicator calculations, design_pattern: Strategy"
    pub fn annotate_component(&mut self, args: &[&str]) {
        let params = Params::parse(args);
        self.elements.insert(
            params.get("name"),
            ElementSetup {
                c4_level: "component".to_string(),
                purpose: params.get("purpose"),
                design_pattern: params.get_opt("design_pattern").unwrap_or_default(),
            },
        );
    }

    /// Declare a dependency between elements.
    /// Format: "from: engine, to: bus, label: Routes commands, protocol: crossbeam"
    pub fn declare_dependency(&mut self, args: &[&str]) {
        let params = Params::parse(args);
        self.dependencies.push(DependencyDecl {
            from: params.get("from"),
            to: params.get("to"),
            label: params.get("label"),
            protocol: params.get("protocol"),
        });
    }

    /// Add a file to an element's catalog.
    /// Format: "element: bus, file: lanes.rs, design_pattern: Observer, responsibility: Event routing, maturity: active"
    pub fn catalog_file(&mut self, args: &[&str]) {
        let params = Params::parse(args);
        let design_pattern = params.get_opt("design_pattern").unwrap_or_else(|| "--".to_string());
        let confidence = params.get_opt("confidence");

        let pattern_field = if let Some(c) = confidence {
            format!("{} ({})", design_pattern, c)
        } else {
            design_pattern
        };

        self.catalog_entries.push(CatalogEntry {
            element: params.get("element"),
            filename: params.get("file"),
            design_pattern: pattern_field,
            responsibility: params.get("responsibility"),
            maturity: params.get("maturity"),
        });
    }

    /// Set the pattern confidence for an element.
    /// Format: "name: bus, confidence: verified"
    pub fn set_pattern_confidence(&mut self, args: &[&str]) {
        let params = Params::parse(args);
        self.confidence_overrides
            .insert(params.get("name"), params.get("confidence"));
    }

    // =========================================================================
    // Action — compile the architecture
    // =========================================================================

    /// Build annotated source files and compile to documentation + diagrams.
    pub fn compile(&mut self) {
        self.build_source_files();
        self.driver.compile();
    }

    // =========================================================================
    // Assertions — verify user-visible outcomes
    // =========================================================================

    /// Assert documentation was produced for a named element.
    /// Format: "name: bus"
    pub fn assert_documentation_exists(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver.confirm_documentation_exists(&params.get("name"));
    }

    /// Assert documentation describes expected content.
    /// Format: "name: bus, describes: Central messaging backbone"
    pub fn assert_documentation_describes(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_documentation_describes(&params.get("name"), &params.get("describes"));
    }

    /// Assert the architecture index was produced.
    pub fn assert_index_exists(&self) {
        self.driver.confirm_index_exists();
    }

    /// Assert the architecture index lists an element.
    /// Format: "name: bus"
    pub fn assert_index_lists(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver.confirm_index_lists(&params.get("name"));
    }

    /// Assert the architecture diagram shows a container.
    /// Format: "name: bus"
    pub fn assert_diagram_shows_container(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver.confirm_diagram_shows_container(&params.get("name"));
    }

    /// Assert the architecture diagram shows a component inside a container.
    /// Format: "name: bus.calc, inside: bus"
    pub fn assert_diagram_shows_component(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver.confirm_diagram_shows_component(
            &params.get("name"),
            &params.get_opt("inside").unwrap_or_default(),
        );
    }

    /// Assert the architecture diagram shows a dependency.
    /// Format: "from: engine, to: bus"
    pub fn assert_diagram_shows_dependency(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_diagram_shows_dependency(&params.get("from"), &params.get("to"));
    }

    /// Assert a diagram export was produced.
    /// Format: "level: container"
    pub fn assert_export_produced(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver.confirm_export_produced(&params.get("level"));
    }

    /// Assert an element's C4 level.
    /// Format: "name: bus, level: container"
    pub fn assert_element_level(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_element_level(&params.get("name"), &params.get("level"));
    }

    /// Assert an element's design pattern.
    /// Format: "name: bus, design_pattern: Mediator"
    pub fn assert_design_pattern(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_design_pattern(&params.get("name"), &params.get("design_pattern"));
    }

    /// Assert pattern confidence (planned/verified).
    /// Format: "name: bus, confidence: planned"
    pub fn assert_pattern_confidence(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_pattern_confidence(&params.get("name"), &params.get("confidence"));
    }

    /// Assert a component lives inside a container.
    /// Format: "name: bus.calc, inside: bus"
    pub fn assert_containment(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_containment(&params.get("name"), &params.get("inside"));
    }

    /// Assert an element is at the top level (no parent).
    /// Format: "name: bus"
    pub fn assert_top_level(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver.confirm_top_level(&params.get("name"));
    }

    /// Assert the total number of compiled elements.
    /// Format: "count: 3"
    pub fn assert_total_elements(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver.confirm_total_elements(params.get_usize("count"));
    }

    /// Assert a catalog entry.
    /// Format: "element: bus, file: lanes.rs, design_pattern: Observer, responsibility: Event routing, maturity: active"
    pub fn assert_catalog_entry(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver.confirm_catalog_entry(
            &params.get("element"),
            &params.get("file"),
            &params.get_opt("design_pattern").unwrap_or_default(),
            &params.get_opt("responsibility").unwrap_or_default(),
            &params.get_opt("maturity").unwrap_or_default(),
        );
    }

    /// Assert the size of a file catalog.
    /// Format: "element: bus, count: 3"
    pub fn assert_catalog_size(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_catalog_size(&params.get("element"), params.get_usize("count"));
    }

    /// Assert a dependency exists.
    /// Format: "from: bus, to: agents_internal, label: Processed data, protocol: crossbeam"
    pub fn assert_dependency(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver.confirm_dependency(
            &params.get("from"),
            &params.get("to"),
            &params.get_opt("label").unwrap_or_default(),
            &params.get_opt("protocol").unwrap_or_default(),
        );
    }

    // =========================================================================
    // Phase B — Health reporting
    // =========================================================================

    /// Assert the number of files at a given maturity level.
    /// Format: "maturity: active, count: 3"
    pub fn assert_health_file_count(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_health_file_count(&params.get("maturity"), params.get_usize("count"));
    }

    /// Assert the number of patterns at a given confidence level.
    /// Format: "confidence: verified, count: 2"
    pub fn assert_health_pattern_count(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_health_pattern_count(&params.get("confidence"), params.get_usize("count"));
    }

    /// Assert the total number of files in the health report.
    /// Format: "count: 5"
    pub fn assert_health_total_files(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_health_total_files(params.get_usize("count"));
    }

    // =========================================================================
    // Phase B — Validation (ghost/orphan detection)
    // =========================================================================

    /// Place a file on disk that is NOT in any catalog (creates an orphan scenario).
    /// Format: "element: bus, file: extra.rs"
    pub fn place_file_on_disk(&mut self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .place_file_on_disk(&params.get("element"), &params.get("file"));
    }

    /// Remove a file from disk that IS in the catalog (creates a ghost scenario).
    /// Format: "element: bus, file: lanes.rs"
    pub fn remove_file_from_disk(&mut self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .remove_file_from_disk(&params.get("element"), &params.get("file"));
    }

    /// Assert a ghost was detected (cataloged file doesn't exist on disk).
    /// Format: "element: bus, file: deleted.rs"
    pub fn assert_ghost_detected(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_ghost(&params.get("element"), &params.get("file"));
    }

    /// Assert an orphan was detected (file on disk not in catalog).
    /// Format: "element: bus, file: extra.rs"
    pub fn assert_orphan_detected(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_orphan(&params.get("element"), &params.get("file"));
    }

    /// Assert file validation found no issues.
    pub fn assert_validation_clean(&self) {
        self.driver.confirm_validation_clean();
    }

    // =========================================================================
    // Phase B — Drift detection
    // =========================================================================

    /// Modify an element's source annotation after compilation (to simulate drift).
    /// Format: "name: bus, purpose: Changed description"
    pub fn modify_source_annotation(&mut self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .modify_source_annotation(&params.get("name"), &params.get("purpose"));
    }

    /// Assert documentation drift was detected.
    pub fn assert_drift_detected(&self) {
        self.driver.confirm_drift_detected();
    }

    /// Assert no documentation drift was detected.
    pub fn assert_no_drift(&self) {
        self.driver.confirm_no_drift();
    }

    // =========================================================================
    // Phase D — Portable IR (cross-language intermediate representation)
    // =========================================================================

    /// Serialize compiled architecture to portable IR.
    pub fn emit_ir(&mut self) {
        self.driver.emit_ir();
    }

    /// Regenerate documentation from portable IR (no source code access).
    pub fn compile_from_ir(&mut self) {
        self.driver.compile_from_ir();
    }

    /// Assert the portable IR contains an element at the expected level.
    /// Format: "name: bus, level: container"
    pub fn assert_ir_contains_element(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_ir_contains_element(&params.get("name"), &params.get("level"));
    }

    /// Assert serializing then deserializing preserves all architecture data.
    pub fn assert_ir_round_trip_preserves_fidelity(&self) {
        self.driver.confirm_ir_round_trip_fidelity();
    }

    /// Assert the emitted IR passes schema validation.
    pub fn assert_ir_schema_valid(&self) {
        self.driver.confirm_ir_schema_valid();
    }

    // =========================================================================
    // Phase H — Pattern validation (structural heuristics)
    // =========================================================================

    /// Place a Rust code file (actual code, not annotations) in a module's directory.
    /// Format: "element: bus.calc, file: indicators.rs"
    /// The code parameter is passed separately.
    pub fn place_code_file(&mut self, element: &str, filename: &str, code: &str) {
        self.driver.create_code_file(element, filename, code);
    }

    /// Run structural heuristics and auto-promote matching patterns.
    pub fn verify_patterns(&mut self) {
        self.driver.verify_patterns();
    }

    /// Assert a named fitness function passes for all matching modules.
    /// Format: "fitness: all_strategy_modules_define_a_trait"
    pub fn assert_fitness_passes(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver.confirm_fitness_passes(&params.get("fitness"));
    }

    /// Assert a named fitness function fails, with a specific module failing.
    /// Format: "fitness: all_strategy_modules_define_a_trait, failing_module: bus.calc"
    pub fn assert_fitness_fails(&self, args: &[&str]) {
        let params = Params::parse(args);
        self.driver
            .confirm_fitness_fails(&params.get("fitness"), &params.get("failing_module"));
    }

    // =========================================================================
    // Internal — build source files from accumulated setup
    // =========================================================================

    fn build_source_files(&mut self) {
        for (name, setup) in &self.elements {
            let mut content = String::new();

            // Header with C4 marker
            let title = to_title_case(name.split('.').last().unwrap_or(name));
            content.push_str(&format!("# {} <<{}>>\n\n", title, setup.c4_level));

            // Purpose
            if !setup.purpose.is_empty() {
                content.push_str(&format!("{}\n\n", setup.purpose));
            }

            // Design pattern
            if !setup.design_pattern.is_empty() {
                if let Some(confidence) = self.confidence_overrides.get(name) {
                    content.push_str(&format!("GoF: {} ({})\n\n", setup.design_pattern, confidence));
                } else {
                    content.push_str(&format!("GoF: {}\n\n", setup.design_pattern));
                }
            }

            // Dependencies
            let deps: Vec<&DependencyDecl> = self
                .dependencies
                .iter()
                .filter(|d| d.from == *name)
                .collect();
            for dep in &deps {
                content.push_str(&format!(
                    "<<uses: {}, \"{}\", \"{}\">>\n",
                    dep.to, dep.label, dep.protocol
                ));
            }
            if !deps.is_empty() {
                content.push('\n');
            }

            // File catalog
            let entries: Vec<&CatalogEntry> = self
                .catalog_entries
                .iter()
                .filter(|e| e.element == *name)
                .collect();
            if !entries.is_empty() {
                content.push_str("| File | Pattern | Purpose | Health |\n");
                content.push_str("|------|---------|---------|--------|\n");
                for entry in &entries {
                    content.push_str(&format!(
                        "| `{}` | {} | {} | {} |\n",
                        entry.filename, entry.design_pattern, entry.responsibility, entry.maturity
                    ));
                }
            }

            self.driver.create_annotated_source(name, &content);
        }
    }
}

fn to_title_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + &chars.as_str().replace('_', " "),
    }
}
