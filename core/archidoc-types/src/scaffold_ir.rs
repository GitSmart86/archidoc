use serde::{Deserialize, Serialize};
use std::path::Path;

fn default_true() -> bool {
    true
}

/// Universal intermediate representation for a scaffold template.
///
/// Produced by the user (or an LLM), consumed by `archidoc scaffold`.
/// A template is a single `.json` file. Large templates compose via
/// `$ref` nodes that inline another ScaffoldIR file at resolution time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldIR {
    pub version: String,
    /// Present in full templates; absent in partials.
    #[serde(default)]
    pub template: Option<ScaffoldTemplate>,
    pub nodes: Vec<ScaffoldNode>,
}

/// Metadata block — only required in top-level template files, not in partials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldTemplate {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub variables: Vec<ScaffoldVariable>,
    #[serde(default)]
    pub post_hooks: Vec<ScaffoldPostHook>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldVariable {
    pub name: String,
    pub description: String,
    /// Defaults to `true` if omitted.
    #[serde(default = "default_true")]
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldPostHook {
    pub command: String,
    pub description: String,
}

/// A single node in the template's `nodes` array.
///
/// Three valid shapes:
/// - `{ "type": "dir",  "path": "some/path" }`
/// - `{ "type": "file", "path": "some/file.rs", "content": "// stub\n" }`
/// - `{ "$ref": "./partials/other.json" }`
///
/// The flat-struct approach lets serde handle all three without custom logic.
/// Node kind is validated at the executor layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldNode {
    /// `"dir"` or `"file"`. Present on typed nodes, absent on `$ref` nodes.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub node_type: Option<String>,

    /// Destination path (relative to target root). Used by `dir` and `file` nodes.
    /// Supports `{{variable}}` substitution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// File content string. Used only by `file` nodes.
    /// Supports `{{variable}}` substitution.
    /// Convention: keep under ~20 lines. Longer content belongs in a post-hook
    /// or a generator tool, not embedded here.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Path to a partial ScaffoldIR JSON file, relative to this file's directory.
    /// The resolver inlines the partial's `nodes` array in place of this node.
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_path: Option<String>,
}

impl ScaffoldIR {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            version: "1.0".to_string(),
            template: Some(ScaffoldTemplate {
                name: name.to_string(),
                description: description.to_string(),
                variables: vec![],
                post_hooks: vec![],
            }),
            nodes: vec![],
        }
    }

    /// Load and deserialize a ScaffoldIR from a JSON file.
    pub fn load(path: &Path) -> Result<Self, String> {
        let json = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        serde_json::from_str(&json).map_err(|e| format!("invalid ScaffoldIR: {}", e))
    }

    /// True if this file has no `template` block (i.e. it is a partial).
    pub fn is_partial(&self) -> bool {
        self.template.is_none()
    }
}
