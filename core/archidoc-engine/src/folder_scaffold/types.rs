use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Deserialize;

/// Parsed from `.archidoc-template.toml` inside a scaffold folder template.
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateManifest {
    pub template: TemplateInfo,
    #[serde(default)]
    pub variables: Vec<TemplateVariable>,
    #[serde(default)]
    pub post_hooks: Vec<PostHook>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateInfo {
    pub name: String,
    pub description: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateVariable {
    pub name: String,
    pub description: String,
    #[serde(default = "default_true")]
    pub required: bool,
    pub default: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
pub struct PostHook {
    pub command: String,
    pub description: String,
}

/// A resolved action to perform during scaffolding.
#[derive(Debug, Clone)]
pub enum ScaffoldAction {
    CreateDir { path: PathBuf },
    CreateFile { path: PathBuf, contents: Vec<u8> },
}

/// The full plan — used for --dry-run output and for execution.
#[derive(Debug, Clone)]
pub struct ScaffoldPlan {
    pub template_name: String,
    pub template_source: PathBuf,
    pub target_root: PathBuf,
    pub variables: BTreeMap<String, String>,
    pub actions: Vec<ScaffoldAction>,
    pub post_hooks: Vec<PostHook>,
}

/// Per-action result.
#[derive(Debug)]
pub enum ActionOutcome {
    Created(PathBuf),
    Skipped { path: PathBuf, reason: String },
    Failed { path: PathBuf, error: String },
}

/// Overall scaffold result.
#[derive(Debug)]
pub struct ScaffoldResult {
    pub outcomes: Vec<ActionOutcome>,
    pub hook_results: Vec<HookResult>,
}

#[derive(Debug)]
pub struct HookResult {
    pub command: String,
    pub success: bool,
    pub output: String,
}

/// Error type for scaffold operations.
#[derive(Debug)]
pub enum ScaffoldError {
    TemplateNotFound(String),
    InvalidManifest { path: PathBuf, source: String },
    MissingVariables(Vec<String>),
    WouldOverwrite(PathBuf),
    SubstitutionError { context: String, detail: String },
    Io(std::io::Error),
}

impl std::fmt::Display for ScaffoldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TemplateNotFound(name) => {
                write!(f, "template '{}' not found (searched up to filesystem root)", name)
            }
            Self::InvalidManifest { path, source } => {
                write!(f, "invalid template manifest at {}: {}", path.display(), source)
            }
            Self::MissingVariables(vars) => {
                write!(f, "missing required variable(s): {}", vars.join(", "))
            }
            Self::WouldOverwrite(path) => {
                write!(f, "target path already exists and --force not set: {}", path.display())
            }
            Self::SubstitutionError { context, detail } => {
                write!(f, "variable substitution error in {}: {}", context, detail)
            }
            Self::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for ScaffoldError {}

impl From<std::io::Error> for ScaffoldError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
