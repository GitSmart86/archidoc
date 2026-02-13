use archidoc_types::ModuleDoc;

/// Unified protocol driver trait for the architecture compiler.
///
/// Combines annotation parsing and output generation into a single
/// pipeline — matching how the user experiences the tool:
/// "annotate source, compile, get documentation and diagrams."
///
/// Tests are stable; this driver is volatile. When the compiler's
/// internals change, update the driver — never the test cases.
pub trait ArchitectureDriver: Send {
    // =========================================================================
    // Setup — create annotated source files
    // =========================================================================

    /// Create an annotated source file for a container or component.
    fn create_annotated_source(&mut self, name: &str, content: &str);

    // =========================================================================
    // Action — run the architecture compiler
    // =========================================================================

    /// Compile: parse all annotated sources and generate all outputs.
    fn compile(&mut self);

    /// Get the parsed modules (for assertions that need the IR).
    fn compiled_modules(&self) -> &[ModuleDoc];

    // =========================================================================
    // Assertions — verify user-visible outcomes
    // =========================================================================

    // --- ARCHITECTURE.md ---

    /// Confirm the ARCHITECTURE.md was produced.
    fn confirm_architecture_produced(&self);

    /// Confirm the ARCHITECTURE.md contains expected text.
    fn confirm_architecture_contains(&self, expected: &str);

    /// Confirm the component index table lists a named element.
    fn confirm_index_lists(&self, name: &str);

    // --- Diagrams (inline in ARCHITECTURE.md) ---

    /// Confirm the architecture diagram shows a container.
    fn confirm_diagram_shows_container(&self, container: &str);

    /// Confirm the architecture diagram shows a component inside a container.
    fn confirm_diagram_shows_component(&self, component: &str, inside: &str);

    /// Confirm the architecture diagram shows a dependency arrow.
    fn confirm_diagram_shows_dependency(&self, from: &str, to: &str);

    // --- Architecture structure ---

    /// Confirm a named element is classified at the expected C4 level.
    fn confirm_element_level(&self, name: &str, expected_level: &str);

    /// Confirm a named element has the expected design pattern.
    fn confirm_design_pattern(&self, name: &str, expected_pattern: &str);

    /// Confirm pattern confidence (planned/verified).
    fn confirm_pattern_confidence(&self, name: &str, expected_confidence: &str);

    /// Confirm containment: a component lives inside a container.
    fn confirm_containment(&self, component: &str, inside: &str);

    /// Confirm an element is at the top level (no parent container).
    fn confirm_top_level(&self, name: &str);

    /// Confirm the total number of architectural elements compiled.
    fn confirm_total_elements(&self, expected: usize);

    // --- File catalog ---

    /// Confirm a file catalog entry exists with the given attributes.
    fn confirm_catalog_entry(
        &self,
        element: &str,
        filename: &str,
        design_pattern: &str,
        responsibility: &str,
        maturity: &str,
    );

    /// Confirm the number of files in a catalog.
    fn confirm_catalog_size(&self, element: &str, expected_count: usize);

    // --- Dependencies ---

    /// Confirm a dependency exists between elements.
    fn confirm_dependency(
        &self,
        from: &str,
        to: &str,
        label: &str,
        protocol: &str,
    );

    // =========================================================================
    // Phase B — Health reporting
    // =========================================================================

    /// Request a health report aggregation.
    fn request_health_report(&self) -> archidoc_types::HealthReport;

    /// Confirm the health report file maturity count for a given status.
    fn confirm_health_file_count(&self, maturity: &str, expected: usize);

    /// Confirm the health report pattern confidence count.
    fn confirm_health_pattern_count(&self, confidence: &str, expected: usize);

    /// Confirm the health report total file count.
    fn confirm_health_total_files(&self, expected: usize);

    // =========================================================================
    // Phase B — Validation (ghost/orphan detection)
    // =========================================================================

    /// Place an actual file on disk in an element's directory (not in catalog).
    fn place_file_on_disk(&mut self, element: &str, filename: &str);

    /// Remove a file from disk that IS in the catalog (create a ghost).
    fn remove_file_from_disk(&mut self, element: &str, filename: &str);

    /// Run file table validation and return the report.
    fn validate(&self) -> archidoc_types::ValidationReport;

    /// Confirm a ghost was detected.
    fn confirm_ghost(&self, element: &str, filename: &str);

    /// Confirm an orphan was detected.
    fn confirm_orphan(&self, element: &str, filename: &str);

    /// Confirm validation found no issues.
    fn confirm_validation_clean(&self);

    // =========================================================================
    // Phase B — Drift detection
    // =========================================================================

    /// Modify the source annotation of an already-compiled element (to create drift).
    fn modify_source_annotation(&mut self, name: &str, new_purpose: &str);

    /// Check for documentation drift against the current generated output.
    fn check_for_drift(&self) -> archidoc_types::DriftReport;

    /// Confirm drift was detected for at least one file.
    fn confirm_drift_detected(&self);

    /// Confirm no drift was detected.
    fn confirm_no_drift(&self);

    // =========================================================================
    // Phase D — Portable IR (JSON intermediate representation)
    // =========================================================================

    /// Serialize compiled modules to portable JSON IR.
    fn emit_ir(&mut self);

    /// Get the emitted IR JSON string.
    fn ir_json(&self) -> &str;

    /// Deserialize IR and regenerate all outputs from it (no source code access).
    fn compile_from_ir(&mut self);

    /// Confirm the emitted IR contains an element at the given C4 level.
    fn confirm_ir_contains_element(&self, name: &str, level: &str);

    /// Confirm serializing then deserializing preserves all architecture data.
    fn confirm_ir_round_trip_fidelity(&self);

    /// Confirm the emitted IR passes schema validation.
    fn confirm_ir_schema_valid(&self);

    /// Confirm that malformed JSON IR is rejected by the validator.
    fn confirm_ir_rejects(&self, json: &str);

    /// Write the emitted IR to a temporary file on disk.
    fn write_ir_to_file(&mut self);

    /// Deserialize IR from the previously written file and regenerate all outputs.
    fn compile_from_ir_file(&mut self);

    /// Confirm emitting IR twice (with a compile-from-IR in between) produces identical JSON.
    fn confirm_ir_idempotent(&mut self);

    // =========================================================================
    // Phase H — Pattern validation
    // =========================================================================

    /// Create a Rust code file (not annotations) in a module's source directory.
    fn create_code_file(&mut self, element: &str, filename: &str, code: &str);

    /// Run structural heuristics and auto-promote matching patterns.
    fn verify_patterns(&mut self);

    /// Run a named fitness function and confirm it passes.
    fn confirm_fitness_passes(&self, fitness_name: &str);

    /// Run a named fitness function and confirm it fails for the given module.
    fn confirm_fitness_fails(&self, fitness_name: &str, failing_module: &str);

    // =========================================================================
    // Phase L — Annotation scaffolding
    // =========================================================================

    /// Generate annotation template for a module's directory.
    fn suggest_for(&mut self, element: &str);

    /// Get the generated suggestion output.
    fn suggestion_output(&self) -> &str;

    /// Confirm the suggestion infers the expected C4 level.
    fn confirm_suggestion_level(&self, level: &str);

    /// Confirm the suggestion lists a source file.
    fn confirm_suggestion_lists_file(&self, filename: &str);

    // =========================================================================
    // Phase L — IR merging
    // =========================================================================

    /// Save current IR as a named snapshot.
    fn save_ir_snapshot(&mut self, name: &str);

    /// Merge named IR snapshots into a unified set.
    fn merge_ir_snapshots(&mut self, names: &[&str]);

    /// Confirm the merged IR has the expected element count.
    fn confirm_merged_element_count(&self, expected: usize);

    /// Confirm the merged IR contains a specific element at a given level.
    fn confirm_merged_contains(&self, name: &str, level: &str);
}
