use serde::{Deserialize, Serialize};

/// Aggregated health report across all architectural elements.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HealthReport {
    pub total_elements: usize,
    pub container_count: usize,
    pub component_count: usize,
    pub total_files: usize,
    pub files_planned: usize,
    pub files_active: usize,
    pub files_stable: usize,
    pub patterns_total: usize,
    pub patterns_planned: usize,
    pub patterns_verified: usize,
    pub per_element: Vec<ElementHealth>,
}

/// Health summary for a single architectural element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementHealth {
    pub name: String,
    pub c4_level: String,
    pub file_count: usize,
    pub files_planned: usize,
    pub files_active: usize,
    pub files_stable: usize,
    pub pattern: String,
    pub pattern_confidence: String,
}

/// Validation report for file table integrity.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationReport {
    pub ghosts: Vec<GhostEntry>,
    pub orphans: Vec<OrphanEntry>,
}

impl ValidationReport {
    pub fn is_clean(&self) -> bool {
        self.ghosts.is_empty() && self.orphans.is_empty()
    }
}

/// A file listed in a catalog but not present on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GhostEntry {
    pub element: String,
    pub filename: String,
    pub source_dir: String,
}

/// A file present on disk but not listed in any catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanEntry {
    pub element: String,
    pub filename: String,
    pub source_dir: String,
}

/// Drift detection report — comparison of generated vs existing docs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DriftReport {
    pub drifted_files: Vec<DriftedFile>,
    pub missing_files: Vec<String>,
    pub extra_files: Vec<String>,
}

impl DriftReport {
    pub fn has_drift(&self) -> bool {
        !self.drifted_files.is_empty()
            || !self.missing_files.is_empty()
            || !self.extra_files.is_empty()
    }
}

/// A single file that differs between generated and existing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftedFile {
    pub path: String,
    pub expected_lines: usize,
    pub actual_lines: usize,
}
