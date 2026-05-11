//! Compile a directory tree into a ScaffoldIR JSON template.
//!
//! Walks a folder-based template directory, reads all files, and produces
//! a single ScaffoldIR with every directory as a `dir` node and every file
//! as a `file` node with its content embedded.
//!
//! If `.archidoc-template.toml` exists, reads template metadata (name,
//! description, variables, post_hooks). Otherwise infers variables from
//! `{{variable}}` patterns found in file contents.

use std::collections::BTreeSet;
use std::path::Path;

use archidoc_types::scaffold_ir::{
    ScaffoldIR, ScaffoldNode, ScaffoldPostHook, ScaffoldTemplate, ScaffoldVariable,
};

/// Compile a directory into a ScaffoldIR.
///
/// - Reads `.archidoc-template.toml` for metadata if present
/// - Walks the tree, creating dir and file nodes
/// - Auto-detects `{{variable}}` patterns in content
/// - Skips `.archidoc-template.toml` itself from the output nodes
pub fn compile(source_dir: &Path) -> Result<ScaffoldIR, String> {
    let toml_path = source_dir.join(".archidoc-template.toml");

    // Read TOML metadata if available
    let toml_meta = if toml_path.exists() {
        Some(parse_toml(&toml_path)?)
    } else {
        None
    };

    // Walk the tree and collect nodes
    let mut nodes = Vec::new();
    let mut detected_vars: BTreeSet<String> = BTreeSet::new();

    walk_dir(source_dir, source_dir, &mut nodes, &mut detected_vars)?;

    // Build template metadata
    let template = if let Some(meta) = toml_meta {
        // Merge auto-detected variables with TOML-declared ones
        let declared_names: BTreeSet<String> =
            meta.variables.iter().map(|v| v.name.clone()).collect();
        let mut variables = meta.variables;

        for var_name in &detected_vars {
            if !declared_names.contains(var_name) {
                variables.push(ScaffoldVariable {
                    name: var_name.clone(),
                    description: format!("Auto-detected from {{{{{}}}}} in template content", var_name),
                    required: true,
                    default: None,
                });
            }
        }

        Some(ScaffoldTemplate {
            name: meta.name,
            description: meta.description,
            variables,
            post_hooks: meta.post_hooks,
        })
    } else {
        // No TOML — derive name from directory, auto-detect all variables
        let name = source_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("template")
            .to_string();

        let variables: Vec<ScaffoldVariable> = detected_vars
            .iter()
            .map(|var_name| ScaffoldVariable {
                name: var_name.clone(),
                description: format!("Auto-detected from {{{{{}}}}} in template content", var_name),
                required: true,
                default: None,
            })
            .collect();

        if variables.is_empty() && nodes.is_empty() {
            None
        } else {
            Some(ScaffoldTemplate {
                name,
                description: String::new(),
                variables,
                post_hooks: vec![],
            })
        }
    };

    Ok(ScaffoldIR {
        version: "1.0".to_string(),
        template,
        nodes,
    })
}

// ---------------------------------------------------------------------------
// TOML parsing
// ---------------------------------------------------------------------------

struct TomlMeta {
    name: String,
    description: String,
    variables: Vec<ScaffoldVariable>,
    post_hooks: Vec<ScaffoldPostHook>,
}

fn parse_toml(path: &Path) -> Result<TomlMeta, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;

    let table: toml::Table = content
        .parse()
        .map_err(|e| format!("invalid TOML in {}: {}", path.display(), e))?;

    let tmpl = table
        .get("template")
        .and_then(|v| v.as_table())
        .ok_or_else(|| format!("missing [template] section in {}", path.display()))?;

    let name = tmpl
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let description = tmpl
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Parse [[variables]]
    let variables = match table.get("variables") {
        Some(toml::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| {
                let tbl = v.as_table()?;
                let var_name = tbl.get("name")?.as_str()?.to_string();
                let desc = tbl
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let required = tbl
                    .get("required")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let default = tbl
                    .get("default")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                Some(ScaffoldVariable {
                    name: var_name,
                    description: desc,
                    required,
                    default,
                })
            })
            .collect(),
        _ => vec![],
    };

    // Parse [[post_hooks]] if present
    let post_hooks = match table.get("post_hooks") {
        Some(toml::Value::Array(arr)) => arr
            .iter()
            .filter_map(|v| {
                let tbl = v.as_table()?;
                let command = tbl.get("command")?.as_str()?.to_string();
                let desc = tbl
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Some(ScaffoldPostHook {
                    command,
                    description: desc,
                })
            })
            .collect(),
        _ => vec![],
    };

    Ok(TomlMeta {
        name,
        description,
        variables,
        post_hooks,
    })
}

// ---------------------------------------------------------------------------
// Directory walker
// ---------------------------------------------------------------------------

/// Files to skip when converting a folder template to ScaffoldIR.
const SKIP_FILES: &[&str] = &[".archidoc-template.toml"];

fn walk_dir(
    root: &Path,
    dir: &Path,
    nodes: &mut Vec<ScaffoldNode>,
    detected_vars: &mut BTreeSet<String>,
) -> Result<(), String> {
    let mut entries: Vec<std::fs::DirEntry> = std::fs::read_dir(dir)
        .map_err(|e| format!("failed to read {}: {}", dir.display(), e))?
        .filter_map(|e| e.ok())
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        if path.is_dir() {
            // Emit dir node
            nodes.push(ScaffoldNode {
                node_type: Some("dir".to_string()),
                path: Some(rel.clone()),
                content: None,
                ref_path: None,
            });

            // Recurse
            walk_dir(root, &path, nodes, detected_vars)?;
        } else {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            if SKIP_FILES.contains(&filename) {
                continue;
            }

            // Read file content
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;

            // Detect {{variable}} patterns
            detect_variables(&content, detected_vars);

            nodes.push(ScaffoldNode {
                node_type: Some("file".to_string()),
                path: Some(rel),
                content: Some(content),
                ref_path: None,
            });
        }
    }

    Ok(())
}

/// Detect `{{variable_name}}` patterns in text.
fn detect_variables(content: &str, vars: &mut BTreeSet<String>) {
    let mut rest = content;
    while let Some(start) = rest.find("{{") {
        let after_open = &rest[start + 2..];
        if let Some(end) = after_open.find("}}") {
            let var_name = after_open[..end].trim();
            if !var_name.is_empty() && is_valid_var_name(var_name) {
                vars.insert(var_name.to_string());
            }
            rest = &after_open[end + 2..];
        } else {
            break;
        }
    }
}

/// Check if a string looks like a variable name (alphanumeric + underscore).
fn is_valid_var_name(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn compile_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let ir = compile(tmp.path()).unwrap();
        assert!(ir.nodes.is_empty());
    }

    #[test]
    fn compile_dir_with_files() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("README.md"), "# Hello\n").unwrap();
        std::fs::create_dir(tmp.path().join("src")).unwrap();
        std::fs::write(tmp.path().join("src/main.rs"), "fn main() {}\n").unwrap();

        let ir = compile(tmp.path()).unwrap();
        assert_eq!(ir.nodes.len(), 3); // src/ dir + README.md + src/main.rs

        let dir_node = ir.nodes.iter().find(|n| n.node_type.as_deref() == Some("dir")).unwrap();
        assert_eq!(dir_node.path.as_deref(), Some("src"));

        let readme = ir.nodes.iter().find(|n| n.path.as_deref() == Some("README.md")).unwrap();
        assert_eq!(readme.content.as_deref(), Some("# Hello\n"));
    }

    #[test]
    fn compile_detects_variables() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join("hello.md"),
            "# {{project_name}}\n\nCreated on {{date}}.\n",
        )
        .unwrap();

        let ir = compile(tmp.path()).unwrap();
        let tmpl = ir.template.as_ref().unwrap();
        let var_names: Vec<&str> = tmpl.variables.iter().map(|v| v.name.as_str()).collect();
        assert!(var_names.contains(&"project_name"));
        assert!(var_names.contains(&"date"));
    }

    #[test]
    fn compile_reads_toml_metadata() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".archidoc-template.toml"),
            r#"
[template]
name = "test"
description = "Test template"
version = "1.0.0"

[[variables]]
name = "name"
description = "Project name"
required = true
"#,
        )
        .unwrap();

        std::fs::write(tmp.path().join("README.md"), "# {{name}}\n").unwrap();

        let ir = compile(tmp.path()).unwrap();
        let tmpl = ir.template.as_ref().unwrap();
        assert_eq!(tmpl.name, "test");
        assert_eq!(tmpl.description, "Test template");
        assert_eq!(tmpl.variables.len(), 1);
        assert_eq!(tmpl.variables[0].name, "name");
    }

    #[test]
    fn compile_skips_toml_from_nodes() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".archidoc-template.toml"),
            "[template]\nname = \"test\"\n",
        )
        .unwrap();
        std::fs::write(tmp.path().join("file.md"), "content\n").unwrap();

        let ir = compile(tmp.path()).unwrap();
        // Only file.md should appear, not .archidoc-template.toml
        assert_eq!(ir.nodes.len(), 1);
        assert_eq!(ir.nodes[0].path.as_deref(), Some("file.md"));
    }

    #[test]
    fn compile_merges_toml_and_detected_vars() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".archidoc-template.toml"),
            r#"
[template]
name = "test"
description = "Test"

[[variables]]
name = "declared_var"
description = "Declared in TOML"
required = true
"#,
        )
        .unwrap();

        std::fs::write(
            tmp.path().join("file.md"),
            "{{declared_var}} and {{auto_var}}\n",
        )
        .unwrap();

        let ir = compile(tmp.path()).unwrap();
        let tmpl = ir.template.as_ref().unwrap();
        let var_names: Vec<&str> = tmpl.variables.iter().map(|v| v.name.as_str()).collect();
        assert!(var_names.contains(&"declared_var"));
        assert!(var_names.contains(&"auto_var"));
        assert_eq!(tmpl.variables.len(), 2);
    }
}
