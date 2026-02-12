use serde::{Deserialize, Serialize};
use std::fmt;

use crate::annotation::{HealthStatus, PatternStatus};

/// C4 architecture level for a module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum C4Level {
    Container,
    Component,
    Unknown,
}

impl fmt::Display for C4Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Container => write!(f, "container"),
            Self::Component => write!(f, "component"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl C4Level {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "container" => Self::Container,
            "component" => Self::Component,
            _ => Self::Unknown,
        }
    }
}

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
    pub relationships: Vec<Relationship>,
    pub files: Vec<FileEntry>,
}
