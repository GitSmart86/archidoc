use std::fs;
use std::path::PathBuf;

use archidoc_types::{HealthReport, ModuleDoc, ValidationReport, DriftReport};
use tempfile::TempDir;

use crate::drivers::protocol_driver::ArchitectureDriver;
use crate::fakes::fake_source_tree::FakeSourceTree;

/// In-memory architecture driver for unit tests.
///
/// Combines the full pipeline: creates annotated files in a temp dir,
/// runs the real parser, then runs the real generators to a temp output dir.
/// Assertions check both the parsed IR and the generated output files.
pub struct InMemoryArchitectureDriver {
    source_tree: FakeSourceTree,
    results: Vec<ModuleDoc>,
    output_dir: TempDir,
    compiled: bool,
    ir_json: Option<String>,
}

impl InMemoryArchitectureDriver {
    pub fn new() -> Self {
        Self {
            source_tree: FakeSourceTree::new(),
            results: Vec::new(),
            output_dir: TempDir::new().expect("failed to create output temp dir"),
            compiled: false,
            ir_json: None,
        }
    }

    fn find_module(&self, name: &str) -> &ModuleDoc {
        self.results
            .iter()
            .find(|doc| doc.module_path == name)
            .unwrap_or_else(|| {
                panic!(
                    "element '{}' not found. Available: {:?}",
                    name,
                    self.results.iter().map(|d| &d.module_path).collect::<Vec<_>>()
                )
            })
    }

    fn design_dir(&self) -> PathBuf {
        self.output_dir.path().join("design")
    }

    fn c4_dir(&self) -> PathBuf {
        self.output_dir.path().join("c4")
    }

    fn drawio_dir(&self) -> PathBuf {
        self.output_dir.path().join("drawio")
    }

    fn read_file(&self, path: &PathBuf) -> String {
        fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("failed to read: {}", path.display()))
    }
}

impl ArchitectureDriver for InMemoryArchitectureDriver {
    fn create_annotated_source(&mut self, name: &str, content: &str) {
        self.source_tree.create_module(name, content);
    }

    fn compile(&mut self) {
        // Parse
        self.results = archidoc_rust::walker::extract_all_docs(self.source_tree.root());

        // Generate
        let design = self.design_dir();
        let c4 = self.c4_dir();
        let drawio = self.drawio_dir();

        fs::create_dir_all(&design).expect("failed to create design dir");
        fs::create_dir_all(&c4).expect("failed to create c4 dir");
        fs::create_dir_all(&drawio).expect("failed to create drawio dir");

        archidoc_engine::markdown::generate_all(&design, &self.results);
        archidoc_engine::mermaid::generate_container(&c4, &self.results);
        archidoc_engine::mermaid::generate_component(&c4, &self.results);
        archidoc_engine::drawio::generate_container_csv(&drawio, &self.results);
        archidoc_engine::drawio::generate_component_csv(&drawio, &self.results);

        self.compiled = true;
    }

    fn compiled_modules(&self) -> &[ModuleDoc] {
        &self.results
    }

    // --- Documentation ---

    fn confirm_documentation_exists(&self, name: &str) {
        let path = self.design_dir().join(format!("{}.md", name));
        assert!(
            path.exists(),
            "documentation not produced for '{}': {}",
            name, path.display()
        );
    }

    fn confirm_documentation_describes(&self, name: &str, expected_content: &str) {
        let path = self.design_dir().join(format!("{}.md", name));
        let content = self.read_file(&path);
        assert!(
            content.contains(expected_content),
            "documentation for '{}' does not describe '{}'. Content:\n{}",
            name, expected_content, content
        );
    }

    fn confirm_index_exists(&self) {
        let path = self.design_dir().join("_intro.md");
        assert!(path.exists(), "architecture index not produced");
    }

    fn confirm_index_lists(&self, name: &str) {
        let path = self.design_dir().join("_intro.md");
        let content = self.read_file(&path);
        assert!(
            content.contains(name),
            "architecture index does not list '{}'. Content:\n{}",
            name, content
        );
    }

    // --- Diagrams ---

    fn confirm_diagram_shows_container(&self, container: &str) {
        let path = self.c4_dir().join("c4-container.md");
        let content = self.read_file(&path);
        // Mermaid IDs use underscores
        let mermaid_id = container.replace('.', "_");
        assert!(
            content.contains(&mermaid_id),
            "container diagram does not show '{}' (id: '{}'). Content:\n{}",
            container, mermaid_id, content
        );
    }

    fn confirm_diagram_shows_component(&self, component: &str, inside: &str) {
        let path = self.c4_dir().join("c4-component.md");
        let content = self.read_file(&path);
        let mermaid_id = component.replace('.', "_");
        assert!(
            content.contains(&mermaid_id),
            "component diagram does not show '{}' (id: '{}'). Content:\n{}",
            component, mermaid_id, content
        );
        if !inside.is_empty() {
            assert!(
                content.contains(inside),
                "component diagram does not show container '{}'. Content:\n{}",
                inside, content
            );
        }
    }

    fn confirm_diagram_shows_dependency(&self, from: &str, to: &str) {
        let container_path = self.c4_dir().join("c4-container.md");
        let component_path = self.c4_dir().join("c4-component.md");

        let container_content = fs::read_to_string(&container_path).unwrap_or_default();
        let component_content = fs::read_to_string(&component_path).unwrap_or_default();
        let combined = format!("{}\n{}", container_content, component_content);

        let from_id = from.replace('.', "_");
        let to_id = to.replace('.', "_");
        let rel_pattern = format!("Rel({}, {}", from_id, to_id);
        assert!(
            combined.contains(&rel_pattern),
            "no dependency arrow from '{}' to '{}' in diagrams (looked for '{}')",
            from, to, rel_pattern
        );
    }

    fn confirm_export_produced(&self, level: &str) {
        let filename = format!("c4-{}.csv", level);
        let path = self.drawio_dir().join(&filename);
        assert!(
            path.exists(),
            "export not produced for level '{}': {}",
            level, path.display()
        );
    }

    // --- Architecture structure ---

    fn confirm_element_level(&self, name: &str, expected_level: &str) {
        let doc = self.find_module(name);
        assert_eq!(
            doc.c4_level.to_string(), expected_level,
            "element '{}': expected level '{}', got '{}'",
            name, expected_level, doc.c4_level
        );
    }

    fn confirm_design_pattern(&self, name: &str, expected_pattern: &str) {
        let doc = self.find_module(name);
        assert_eq!(
            doc.pattern, expected_pattern,
            "element '{}': expected pattern '{}', got '{}'",
            name, expected_pattern, doc.pattern
        );
    }

    fn confirm_pattern_confidence(&self, name: &str, expected_confidence: &str) {
        let doc = self.find_module(name);
        assert_eq!(
            doc.pattern_status.to_string(), expected_confidence,
            "element '{}': expected confidence '{}', got '{}'",
            name, expected_confidence, doc.pattern_status
        );
    }

    fn confirm_containment(&self, component: &str, inside: &str) {
        let doc = self.find_module(component);
        let actual = doc.parent_container.as_deref().unwrap_or("");
        assert_eq!(
            actual, inside,
            "element '{}': expected inside '{}', got '{}'",
            component, inside, actual
        );
    }

    fn confirm_top_level(&self, name: &str) {
        let doc = self.find_module(name);
        assert!(
            doc.parent_container.is_none(),
            "element '{}': expected top-level but found parent '{}'",
            name, doc.parent_container.as_deref().unwrap_or("")
        );
    }

    fn confirm_total_elements(&self, expected: usize) {
        assert_eq!(
            self.results.len(), expected,
            "expected {} elements, got {}. Elements: {:?}",
            expected, self.results.len(),
            self.results.iter().map(|d| &d.module_path).collect::<Vec<_>>()
        );
    }

    // --- File catalog ---

    fn confirm_catalog_entry(
        &self,
        element: &str,
        filename: &str,
        design_pattern: &str,
        responsibility: &str,
        maturity: &str,
    ) {
        let doc = self.find_module(element);
        let entry = doc
            .files
            .iter()
            .find(|f| f.name == filename)
            .unwrap_or_else(|| {
                panic!(
                    "element '{}': catalog entry '{}' not found. Available: {:?}",
                    element, filename,
                    doc.files.iter().map(|f| &f.name).collect::<Vec<_>>()
                )
            });

        if !design_pattern.is_empty() {
            // Reconstruct combined form "Pattern (status)" for comparison
            // when the expected string includes a parenthetical confidence
            let actual = if design_pattern.contains('(') {
                format!("{} ({})", entry.pattern, entry.pattern_status)
            } else {
                entry.pattern.clone()
            };
            assert_eq!(
                actual, design_pattern,
                "element '{}', file '{}': expected pattern '{}', got '{}'",
                element, filename, design_pattern, actual
            );
        }
        if !responsibility.is_empty() {
            assert_eq!(
                entry.purpose, responsibility,
                "element '{}', file '{}': expected responsibility '{}', got '{}'",
                element, filename, responsibility, entry.purpose
            );
        }
        if !maturity.is_empty() {
            assert_eq!(
                entry.health.to_string(), maturity,
                "element '{}', file '{}': expected maturity '{}', got '{}'",
                element, filename, maturity, entry.health
            );
        }
    }

    fn confirm_catalog_size(&self, element: &str, expected_count: usize) {
        let doc = self.find_module(element);
        assert_eq!(
            doc.files.len(), expected_count,
            "element '{}': expected {} catalog entries, got {}",
            element, expected_count, doc.files.len()
        );
    }

    // --- Dependencies ---

    fn confirm_dependency(
        &self,
        from: &str,
        to: &str,
        label: &str,
        protocol: &str,
    ) {
        let doc = self.find_module(from);
        let found = doc.relationships.iter().any(|r| {
            r.target == to
                && (label.is_empty() || r.label == label)
                && (protocol.is_empty() || r.protocol == protocol)
        });
        assert!(
            found,
            "element '{}': dependency to '{}' (label: '{}', protocol: '{}') not found. Available: {:?}",
            from, to, label, protocol,
            doc.relationships.iter().map(|r| format!("{}/{}/{}", r.target, r.label, r.protocol)).collect::<Vec<_>>()
        );
    }

    // =========================================================================
    // Phase B — Health reporting
    // =========================================================================

    fn request_health_report(&self) -> HealthReport {
        archidoc_engine::health::aggregate_health(&self.results)
    }

    fn confirm_health_file_count(&self, maturity: &str, expected: usize) {
        let report = self.request_health_report();
        let actual = match maturity {
            "planned" => report.files_planned,
            "active" => report.files_active,
            "stable" => report.files_stable,
            _ => panic!("unknown maturity: '{}'", maturity),
        };
        assert_eq!(
            actual, expected,
            "health report: expected {} '{}' files, got {}",
            expected, maturity, actual
        );
    }

    fn confirm_health_pattern_count(&self, confidence: &str, expected: usize) {
        let report = self.request_health_report();
        let actual = match confidence {
            "planned" => report.patterns_planned,
            "verified" => report.patterns_verified,
            _ => panic!("unknown confidence: '{}'", confidence),
        };
        assert_eq!(
            actual, expected,
            "health report: expected {} '{}' patterns, got {}",
            expected, confidence, actual
        );
    }

    fn confirm_health_total_files(&self, expected: usize) {
        let report = self.request_health_report();
        assert_eq!(
            report.total_files, expected,
            "health report: expected {} total files, got {}",
            expected, report.total_files
        );
    }

    // =========================================================================
    // Phase B — Validation (ghost/orphan detection)
    // =========================================================================

    fn place_file_on_disk(&mut self, element: &str, filename: &str) {
        self.source_tree.place_extra_file(element, filename);
    }

    fn remove_file_from_disk(&mut self, element: &str, filename: &str) {
        self.source_tree.remove_file(element, filename);
    }

    fn validate(&self) -> ValidationReport {
        archidoc_engine::validate::validate_file_tables(&self.results)
    }

    fn confirm_ghost(&self, element: &str, filename: &str) {
        let report = self.validate();
        let found = report.ghosts.iter().any(|g| g.element == element && g.filename == filename);
        assert!(
            found,
            "expected ghost '{}' in element '{}'. Ghosts: {:?}",
            filename, element,
            report.ghosts.iter().map(|g| format!("{}/{}", g.element, g.filename)).collect::<Vec<_>>()
        );
    }

    fn confirm_orphan(&self, element: &str, filename: &str) {
        let report = self.validate();
        let found = report.orphans.iter().any(|o| o.element == element && o.filename == filename);
        assert!(
            found,
            "expected orphan '{}' in element '{}'. Orphans: {:?}",
            filename, element,
            report.orphans.iter().map(|o| format!("{}/{}", o.element, o.filename)).collect::<Vec<_>>()
        );
    }

    fn confirm_validation_clean(&self) {
        let report = self.validate();
        assert!(
            report.is_clean(),
            "expected clean validation but found {} ghosts and {} orphans",
            report.ghosts.len(), report.orphans.len()
        );
    }

    // =========================================================================
    // Phase B — Drift detection
    // =========================================================================

    fn modify_source_annotation(&mut self, name: &str, new_purpose: &str) {
        // Re-create the source file with a different description
        let content = format!(
            "# {} <<container>>\n\n{}\n",
            name.split('.').last().unwrap_or(name),
            new_purpose
        );
        self.source_tree.create_module(name, &content);
    }

    fn check_for_drift(&self) -> DriftReport {
        // Re-parse from the (possibly modified) source tree
        let fresh_docs = archidoc_rust::walker::extract_all_docs(self.source_tree.root());
        archidoc_engine::check::check_drift(&fresh_docs, self.output_dir.path())
    }

    fn confirm_drift_detected(&self) {
        let report = self.check_for_drift();
        assert!(
            report.has_drift(),
            "expected drift but documentation appears up to date"
        );
    }

    fn confirm_no_drift(&self) {
        let report = self.check_for_drift();
        assert!(
            !report.has_drift(),
            "expected no drift but found: {} drifted, {} missing, {} extra",
            report.drifted_files.len(),
            report.missing_files.len(),
            report.extra_files.len()
        );
    }

    // =========================================================================
    // Phase D — Portable IR
    // =========================================================================

    fn emit_ir(&mut self) {
        assert!(self.compiled, "must compile before emitting IR");
        self.ir_json = Some(archidoc_engine::ir::serialize(&self.results));
    }

    fn ir_json(&self) -> &str {
        self.ir_json
            .as_deref()
            .expect("IR not emitted yet — call emit_ir() first")
    }

    fn compile_from_ir(&mut self) {
        let json = self.ir_json().to_string();
        let docs = archidoc_engine::ir::deserialize(&json)
            .expect("failed to deserialize IR");

        self.results = docs;

        // Regenerate all outputs from deserialized IR
        let design = self.design_dir();
        let c4 = self.c4_dir();
        let drawio = self.drawio_dir();

        // Clear previous output
        if design.exists() {
            fs::remove_dir_all(&design).ok();
        }
        if c4.exists() {
            fs::remove_dir_all(&c4).ok();
        }
        if drawio.exists() {
            fs::remove_dir_all(&drawio).ok();
        }

        fs::create_dir_all(&design).expect("failed to create design dir");
        fs::create_dir_all(&c4).expect("failed to create c4 dir");
        fs::create_dir_all(&drawio).expect("failed to create drawio dir");

        archidoc_engine::markdown::generate_all(&design, &self.results);
        archidoc_engine::mermaid::generate_container(&c4, &self.results);
        archidoc_engine::mermaid::generate_component(&c4, &self.results);
        archidoc_engine::drawio::generate_container_csv(&drawio, &self.results);
        archidoc_engine::drawio::generate_component_csv(&drawio, &self.results);

        self.compiled = true;
    }

    fn confirm_ir_contains_element(&self, name: &str, level: &str) {
        let json = self.ir_json();
        let docs: Vec<ModuleDoc> = archidoc_engine::ir::deserialize(json)
            .expect("failed to parse IR for assertion");
        let found = docs.iter().any(|d| d.module_path == name && d.c4_level.to_string() == level);
        assert!(
            found,
            "IR does not contain element '{}' at level '{}'. Elements: {:?}",
            name, level,
            docs.iter().map(|d| format!("{} ({})", d.module_path, d.c4_level)).collect::<Vec<_>>()
        );
    }

    fn confirm_ir_round_trip_fidelity(&self) {
        let json = self.ir_json();
        let round_tripped: Vec<ModuleDoc> = archidoc_engine::ir::deserialize(json)
            .expect("failed to deserialize IR for round-trip check");

        assert_eq!(
            self.results.len(),
            round_tripped.len(),
            "round-trip changed element count: {} -> {}",
            self.results.len(),
            round_tripped.len()
        );

        for (original, restored) in self.results.iter().zip(round_tripped.iter()) {
            assert_eq!(
                original, restored,
                "round-trip fidelity lost for element '{}'",
                original.module_path
            );
        }
    }

    fn confirm_ir_schema_valid(&self) {
        let json = self.ir_json();
        archidoc_engine::ir::validate(json)
            .expect("IR schema validation failed");
    }

    // =========================================================================
    // Phase H — Pattern validation
    // =========================================================================

    fn create_code_file(&mut self, element: &str, filename: &str, code: &str) {
        self.source_tree.create_code_file(element, filename, code);
    }

    fn verify_patterns(&mut self) {
        assert!(self.compiled, "must compile before verifying patterns");
        archidoc_rust::promote::auto_promote(&mut self.results);
    }

    fn confirm_fitness_passes(&self, fitness_name: &str) {
        let result = archidoc_rust::fitness::run_fitness(fitness_name, &self.results)
            .unwrap_or_else(|| panic!("unknown fitness function: '{}'", fitness_name));
        assert!(
            result.passed,
            "expected fitness '{}' to pass but {} failure(s): {:?}",
            fitness_name,
            result.failures.len(),
            result.failures.iter().map(|f| &f.module_path).collect::<Vec<_>>()
        );
    }

    fn confirm_fitness_fails(&self, fitness_name: &str, failing_module: &str) {
        let result = archidoc_rust::fitness::run_fitness(fitness_name, &self.results)
            .unwrap_or_else(|| panic!("unknown fitness function: '{}'", fitness_name));
        assert!(
            !result.passed,
            "expected fitness '{}' to fail but it passed ({} checked)",
            fitness_name, result.checked
        );
        let has_module = result
            .failures
            .iter()
            .any(|f| f.module_path == failing_module);
        assert!(
            has_module,
            "expected '{}' in fitness failures but found: {:?}",
            failing_module,
            result.failures.iter().map(|f| &f.module_path).collect::<Vec<_>>()
        );
    }
}
