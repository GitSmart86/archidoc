use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use archidoc_types::scaffold_ir::{ScaffoldIR, ScaffoldPostHook};

#[derive(Debug)]
pub enum ExecuteError {
    InvalidNode { detail: String },
    Io(std::io::Error),
}

impl fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecuteError::InvalidNode { detail } => write!(f, "invalid node: {}", detail),
            ExecuteError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

#[derive(Debug)]
pub enum ActionOutcome {
    Created(PathBuf),
    Skipped { path: PathBuf, reason: String },
}

#[derive(Debug)]
pub struct HookResult {
    pub command: String,
    pub success: bool,
    pub output: String,
}

#[derive(Debug)]
pub struct ExecuteResult {
    pub outcomes: Vec<ActionOutcome>,
    pub hook_results: Vec<HookResult>,
}

/// A planned action for dry-run output.
#[derive(Debug)]
pub struct PlannedAction {
    pub kind: &'static str,
    pub path: PathBuf,
}

/// Dry-run: return planned actions without creating anything.
pub fn dry_run(
    ir: &ScaffoldIR,
    target: &Path,
    variables: &BTreeMap<String, String>,
) -> Result<Vec<PlannedAction>, ExecuteError> {
    let mut actions = Vec::new();

    for node in &ir.nodes {
        match node.node_type.as_deref() {
            Some("dir") => {
                let raw = node.path.as_deref().ok_or_else(|| ExecuteError::InvalidNode {
                    detail: "dir node missing 'path'".to_string(),
                })?;
                let rel = substitute_path(raw, variables)?;
                actions.push(PlannedAction {
                    kind: "dir",
                    path: target.join(rel),
                });
            }
            Some("file") => {
                let raw = node.path.as_deref().ok_or_else(|| ExecuteError::InvalidNode {
                    detail: "file node missing 'path'".to_string(),
                })?;
                let rel = substitute_path(raw, variables)?;
                actions.push(PlannedAction {
                    kind: "file",
                    path: target.join(rel),
                });
            }
            Some(other) => {
                return Err(ExecuteError::InvalidNode {
                    detail: format!("unknown node type '{}'", other),
                });
            }
            None if node.ref_path.is_some() => {
                return Err(ExecuteError::InvalidNode {
                    detail: "$ref node found during execution — run resolver first".to_string(),
                });
            }
            None => {
                return Err(ExecuteError::InvalidNode {
                    detail: "node missing 'type' and '$ref'".to_string(),
                });
            }
        }
    }

    Ok(actions)
}

/// Instantiate the resolved ScaffoldIR into `target`.
///
/// `ir` must be fully resolved (no `$ref` nodes remain).
/// Existing files are skipped when `force` is false; overwritten when true.
pub fn execute(
    ir: &ScaffoldIR,
    target: &Path,
    variables: &BTreeMap<String, String>,
    force: bool,
) -> Result<ExecuteResult, ExecuteError> {
    let mut outcomes = Vec::new();

    for node in &ir.nodes {
        match node.node_type.as_deref() {
            Some("dir") => {
                let raw = node.path.as_deref().ok_or_else(|| ExecuteError::InvalidNode {
                    detail: "dir node missing 'path'".to_string(),
                })?;
                let rel = substitute_path(raw, variables)?;
                let abs = target.join(&rel);

                if abs.exists() {
                    outcomes.push(ActionOutcome::Skipped {
                        path: abs,
                        reason: "directory already exists".to_string(),
                    });
                } else {
                    std::fs::create_dir_all(&abs).map_err(ExecuteError::Io)?;
                    outcomes.push(ActionOutcome::Created(abs));
                }
            }
            Some("file") => {
                let raw_path = node.path.as_deref().ok_or_else(|| ExecuteError::InvalidNode {
                    detail: "file node missing 'path'".to_string(),
                })?;
                let rel = substitute_path(raw_path, variables)?;
                let abs = target.join(&rel);

                if !force && abs.exists() {
                    outcomes.push(ActionOutcome::Skipped {
                        path: abs,
                        reason: "file exists (use --force to overwrite)".to_string(),
                    });
                    continue;
                }

                let raw_content = node.content.as_deref().unwrap_or("");
                let content = substitute_text(raw_content, variables);

                if let Some(parent) = abs.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent).map_err(ExecuteError::Io)?;
                    }
                }

                std::fs::write(&abs, &content).map_err(ExecuteError::Io)?;
                outcomes.push(ActionOutcome::Created(abs));
            }
            Some(other) => {
                return Err(ExecuteError::InvalidNode {
                    detail: format!("unknown node type '{}'", other),
                });
            }
            None if node.ref_path.is_some() => {
                return Err(ExecuteError::InvalidNode {
                    detail: "$ref node found during execution — run resolver first".to_string(),
                });
            }
            None => {
                return Err(ExecuteError::InvalidNode {
                    detail: "node missing 'type' and '$ref'".to_string(),
                });
            }
        }
    }

    let post_hooks = ir
        .template
        .as_ref()
        .map(|t| t.post_hooks.as_slice())
        .unwrap_or(&[]);
    let hook_results = run_post_hooks(post_hooks, target);

    Ok(ExecuteResult {
        outcomes,
        hook_results,
    })
}

// ---------------------------------------------------------------------------
// Substitution
// ---------------------------------------------------------------------------

pub fn substitute_text(text: &str, variables: &BTreeMap<String, String>) -> String {
    let mut result = text.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

fn substitute_path(
    rel: &str,
    variables: &BTreeMap<String, String>,
) -> Result<String, ExecuteError> {
    Ok(substitute_text(rel, variables))
}

// ---------------------------------------------------------------------------
// Post-hooks
// ---------------------------------------------------------------------------

fn run_post_hooks(hooks: &[ScaffoldPostHook], working_dir: &Path) -> Vec<HookResult> {
    hooks
        .iter()
        .map(|hook| {
            let output = if cfg!(windows) {
                std::process::Command::new("cmd")
                    .args(["/C", &hook.command])
                    .current_dir(working_dir)
                    .output()
            } else {
                std::process::Command::new("sh")
                    .args(["-c", &hook.command])
                    .current_dir(working_dir)
                    .output()
            };

            match output {
                Ok(result) => {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    HookResult {
                        command: hook.command.clone(),
                        success: result.status.success(),
                        output: format!("{}{}", stdout, stderr),
                    }
                }
                Err(e) => HookResult {
                    command: hook.command.clone(),
                    success: false,
                    output: e.to_string(),
                },
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::scaffold_ir::{ScaffoldIR, ScaffoldNode};
    use tempfile::TempDir;

    fn ir(nodes: Vec<ScaffoldNode>) -> ScaffoldIR {
        ScaffoldIR {
            version: "1.0".to_string(),
            template: None,
            nodes,
        }
    }

    fn dir_node(path: &str) -> ScaffoldNode {
        ScaffoldNode {
            node_type: Some("dir".to_string()),
            path: Some(path.to_string()),
            content: None,
            ref_path: None,
        }
    }

    fn file_node(path: &str, content: &str) -> ScaffoldNode {
        ScaffoldNode {
            node_type: Some("file".to_string()),
            path: Some(path.to_string()),
            content: Some(content.to_string()),
            ref_path: None,
        }
    }

    fn vars(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn creates_dir_and_file() {
        let tmp = TempDir::new().unwrap();
        let template = ir(vec![
            dir_node("src"),
            file_node("src/lib.rs", "// stub\n"),
        ]);
        let result = execute(&template, tmp.path(), &BTreeMap::new(), false).unwrap();

        assert!(tmp.path().join("src").is_dir());
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("src/lib.rs")).unwrap(),
            "// stub\n"
        );
        assert_eq!(result.outcomes.len(), 2);
    }

    #[test]
    fn variable_substitution_in_path_and_content() {
        let tmp = TempDir::new().unwrap();
        let template = ir(vec![file_node(
            "{{name}}/mod.rs",
            "//! # {{name}}\n",
        )]);
        let v = vars(&[("name", "auth")]);
        execute(&template, tmp.path(), &v, false).unwrap();

        let content = std::fs::read_to_string(tmp.path().join("auth/mod.rs")).unwrap();
        assert_eq!(content, "//! # auth\n");
    }

    #[test]
    fn existing_file_skipped_without_force() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("file.rs"), "original").unwrap();

        let template = ir(vec![file_node("file.rs", "new content")]);
        let result = execute(&template, tmp.path(), &BTreeMap::new(), false).unwrap();

        assert!(matches!(result.outcomes[0], ActionOutcome::Skipped { .. }));
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("file.rs")).unwrap(),
            "original"
        );
    }

    #[test]
    fn existing_file_overwritten_with_force() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join("file.rs"), "original").unwrap();

        let template = ir(vec![file_node("file.rs", "new content")]);
        execute(&template, tmp.path(), &BTreeMap::new(), true).unwrap();

        assert_eq!(
            std::fs::read_to_string(tmp.path().join("file.rs")).unwrap(),
            "new content"
        );
    }

    #[test]
    fn dry_run_returns_actions_without_creating() {
        let tmp = TempDir::new().unwrap();
        let template = ir(vec![dir_node("src"), file_node("src/lib.rs", "// stub")]);
        let actions = dry_run(&template, tmp.path(), &BTreeMap::new()).unwrap();

        assert_eq!(actions.len(), 2);
        assert!(!tmp.path().join("src").exists());
    }

    #[test]
    fn substitute_text_replaces_tokens() {
        let v = vars(&[("name", "world")]);
        assert_eq!(substitute_text("hello {{name}}", &v), "hello world");
    }

    #[test]
    fn substitute_text_leaves_unknown_tokens() {
        let v = vars(&[]);
        assert_eq!(substitute_text("{{unknown}}", &v), "{{unknown}}");
    }
}
