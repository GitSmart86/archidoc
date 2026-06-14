use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::annotation::{HealthStatus, PatternStatus};
pub use crate::ir::C4Level;

/// A runtime dependency between modules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relationship {
    pub target: String,
    pub label: String,
    pub protocol: String,
}

/// A file entry from the module's file table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub pattern: String,
    pub pattern_status: PatternStatus,
    pub purpose: String,
    pub health: HealthStatus,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, String>,
}

/// A parsed module documentation unit.
///
/// This is the core data structure — the JSON IR contract between
/// language adapters and the core generator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleDoc {
    pub module_path: String,
    pub content: String,
    pub source_file: String,
    pub c4_level: C4Level,
    pub pattern: String,
    pub pattern_status: PatternStatus,
    pub description: String,
    pub parent_container: Option<String>,
    #[serde(default)]
    pub layer: Option<String>,
    pub relationships: Vec<Relationship>,
    pub files: Vec<FileEntry>,
}
