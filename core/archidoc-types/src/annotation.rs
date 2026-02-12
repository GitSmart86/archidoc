use serde::{Deserialize, Serialize};
use std::fmt;

/// Two-tier confidence for GoF pattern assignments.
///
/// `planned` — developer intent, not yet structurally validated.
/// `verified` — structural heuristic has confirmed pattern alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatternStatus {
    Planned,
    Verified,
}

impl Default for PatternStatus {
    fn default() -> Self {
        Self::Planned
    }
}

impl fmt::Display for PatternStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::Verified => write!(f, "verified"),
        }
    }
}

impl PatternStatus {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "verified" => Self::Verified,
            _ => Self::Planned,
        }
    }
}

/// Implementation maturity of a file.
///
/// Progression: `planned` -> `active` -> `stable`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Planned,
    Active,
    Stable,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Planned
    }
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Planned => write!(f, "planned"),
            Self::Active => write!(f, "active"),
            Self::Stable => write!(f, "stable"),
        }
    }
}

impl HealthStatus {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "active" => Self::Active,
            "stable" => Self::Stable,
            _ => Self::Planned,
        }
    }
}
