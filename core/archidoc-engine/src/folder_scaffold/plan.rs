use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::types::{PostHook, ScaffoldAction, ScaffoldError, ScaffoldPlan};

const MANIFEST_FILENAME: &str = ".archidoc-template.toml";

/// Build a scaffold plan by walking the template tree.
///
/// Applies `{{var}}` substitution to both paths and file contents.
/// Checks for overwrites unless `force` is true.
pub fn build_plan(
    template_name: &str,
    template_dir: &Path,
    target_root: &Path,
    variables: &BTreeMap<String, String>,
    post_hooks: Vec<PostHook>,
    force: bool,
) -> Result<ScaffoldPlan, ScaffoldError> {
    let mut actions: Vec<ScaffoldAction> = Vec::new();

    walk_template(template_dir, template_dir, target_root, variables, force, &mut actions)?;

    // Sort: directories before files, then by path
    actions.sort_by(|a, b| {
        let (a_is_dir, a_path) = match a {
            ScaffoldAction::CreateDir { path } => (true, path),
            ScaffoldAction::CreateFile { path, .. } => (false, path),
        };
        let (b_is_dir, b_path) = match b {
            ScaffoldAction::CreateDir { path } => (true, path),
            ScaffoldAction::CreateFile { path, .. } => (false, path),
        };
        b_is_dir.cmp(&a_is_dir).then_with(|| a_path.cmp(b_path))
    });

    Ok(ScaffoldPlan {
        template_name: template_name.to_string(),
        template_source: template_dir.to_path_buf(),
        target_root: target_root.to_path_buf(),
        variables: variables.clone(),
        actions,
        post_hooks,
    })
}

fn walk_template(
    template_root: &Path,
    current_dir: &Path,
    target_root: &Path,
    variables: &BTreeMap<String, String>,
    force: bool,
    actions: &mut Vec<ScaffoldAction>,
) -> Result<(), ScaffoldError> {
    let entries = std::fs::read_dir(current_dir).map_err(ScaffoldError::Io)?;

    for entry in entries.flatten() {
        let source_path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip the manifest file itself
        if file_name == MANIFEST_FILENAME {
            continue;
        }

        // Compute relative path from template root
        let rel_path = source_path
            .strip_prefix(template_root)
            .unwrap_or(&source_path);

        // Substitute variables in each path component
        let substituted_rel = substitute_path(rel_path, variables)?;
        let dest_path = target_root.join(&substituted_rel);

        if source_path.is_dir() {
            actions.push(ScaffoldAction::CreateDir {
                path: dest_path.clone(),
            });
            walk_template(template_root, &source_path, target_root, variables, force, actions)?;
        } else {
            // Check overwrite
            if !force && dest_path.exists() {
                return Err(ScaffoldError::WouldOverwrite(dest_path));
            }

            // Read file and substitute contents
            let raw = std::fs::read(&source_path).map_err(ScaffoldError::Io)?;

            // Only substitute in text files (heuristic: no null bytes in first 8KB)
            let check_len = raw.len().min(8192);
            let is_text = !raw[..check_len].contains(&0u8);

            let contents = if is_text {
                let text = String::from_utf8_lossy(&raw);
                let substituted = substitute_text(&text, variables);
                substituted.into_bytes()
            } else {
                raw
            };

            actions.push(ScaffoldAction::CreateFile {
                path: dest_path,
                contents,
            });
        }
    }

    Ok(())
}

/// Substitute `{{var}}` tokens in each component of a path.
fn substitute_path(
    rel_path: &Path,
    variables: &BTreeMap<String, String>,
) -> Result<PathBuf, ScaffoldError> {
    let mut result = PathBuf::new();
    for component in rel_path.components() {
        let s = component.as_os_str().to_string_lossy();
        let substituted = substitute_text(&s, variables);
        result.push(&substituted);
    }
    Ok(result)
}

/// Substitute `{{var}}` tokens in a text string.
///
/// Unknown variables are left intact (they may be intended literals).
fn substitute_text(text: &str, variables: &BTreeMap<String, String>) -> String {
    let mut result = text.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitute_text_replaces_known_vars() {
        let mut vars = BTreeMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        assert_eq!(substitute_text("Hello {{name}}", &vars), "Hello Alice");
    }

    #[test]
    fn substitute_text_leaves_unknown_intact() {
        let vars = BTreeMap::new();
        assert_eq!(substitute_text("{{unknown}}", &vars), "{{unknown}}");
    }

    #[test]
    fn substitute_path_replaces_components() {
        let mut vars = BTreeMap::new();
        vars.insert("id".to_string(), "2026-001".to_string());
        let path = Path::new("engagements/{{id}}/SPEC.md");
        let result = substitute_path(path, &vars).unwrap();
        assert_eq!(result, PathBuf::from("engagements/2026-001/SPEC.md"));
    }
}
