use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::types::{ScaffoldAction, ScaffoldError, ScaffoldPlan, TemplateInfo, TemplateManifest};

/// Built-in scaffold templates that ship with the binary.
///
/// These work even when no `.archidoc/` directory exists — they bootstrap it.
/// Disk templates shadow built-ins with the same name.

/// Names of all built-in templates.
pub const BUILTIN_NAMES: &[&str] = &["custom-scaffolds", "custom-inits", "custom-trees"];

/// Check if a name matches a built-in template.
pub fn is_builtin(name: &str) -> bool {
    BUILTIN_NAMES.contains(&name)
}

/// Get the manifest for a built-in template.
pub fn builtin_manifest(name: &str) -> Option<TemplateManifest> {
    match name {
        "custom-scaffolds" => Some(TemplateManifest {
            template: TemplateInfo {
                name: "custom-scaffolds".to_string(),
                description: "Create .archidoc/scaffold-templates/ for your own folder templates".to_string(),
                version: "0.1.0".to_string(),
            },
            variables: vec![],
            post_hooks: vec![],
        }),
        "custom-inits" => Some(TemplateManifest {
            template: TemplateInfo {
                name: "custom-inits".to_string(),
                description: "Add init-overrides/ with default override files for customizing init handler output".to_string(),
                version: "0.1.0".to_string(),
            },
            variables: vec![],
            post_hooks: vec![],
        }),
        "custom-trees" => Some(TemplateManifest {
            template: TemplateInfo {
                name: "custom-trees".to_string(),
                description: "Add config.tree.json for customizing directory tree generation (exclusions, icons, thresholds)".to_string(),
                version: "0.1.0".to_string(),
            },
            variables: vec![],
            post_hooks: vec![],
        }),
        _ => None,
    }
}

/// Build a scaffold plan for a built-in template.
pub fn builtin_plan(
    name: &str,
    target: &Path,
    force: bool,
) -> Result<ScaffoldPlan, ScaffoldError> {
    let archidoc_dir = target.join(".archidoc");
    let mut actions: Vec<ScaffoldAction> = Vec::new();

    match name {
        "custom-scaffolds" => {
            actions.push(ScaffoldAction::CreateDir {
                path: archidoc_dir.clone(),
            });
            actions.push(ScaffoldAction::CreateDir {
                path: archidoc_dir.join("scaffold-templates"),
            });
        }
        "custom-inits" => {
            let overrides_dir = archidoc_dir.join("init-overrides");
            actions.push(ScaffoldAction::CreateDir {
                path: archidoc_dir.clone(),
            });
            actions.push(ScaffoldAction::CreateDir {
                path: overrides_dir.clone(),
            });

            let files: &[(&str, &str)] = &[
                ("mod.rs", crate::custom::DEFAULT_SUGGEST_RUST),
                ("index.ts", crate::custom::DEFAULT_SUGGEST_TS),
                ("_index.md", crate::custom::DEFAULT_SUGGEST_MD),
                ("architecture-table.md", crate::custom::DEFAULT_ARCHITECTURE_TABLE),
            ];

            for (filename, content) in files {
                let path = overrides_dir.join(filename);
                if !force && path.exists() {
                    continue;
                }
                actions.push(ScaffoldAction::CreateFile {
                    path,
                    contents: content.as_bytes().to_vec(),
                });
            }
        }
        "custom-trees" => {
            actions.push(ScaffoldAction::CreateDir {
                path: archidoc_dir.clone(),
            });

            let config_path = archidoc_dir.join("config.tree.json");
            if !force && config_path.exists() {
                return Err(ScaffoldError::WouldOverwrite(config_path));
            }
            actions.push(ScaffoldAction::CreateFile {
                path: config_path,
                contents: DEFAULT_CONFIG_TREE_JSON.as_bytes().to_vec(),
            });
        }
        _ => return Err(ScaffoldError::TemplateNotFound(name.to_string())),
    }

    Ok(ScaffoldPlan {
        template_name: name.to_string(),
        template_source: PathBuf::from("(built-in)"),
        target_root: target.to_path_buf(),
        variables: BTreeMap::new(),
        actions,
        post_hooks: vec![],
    })
}

const DEFAULT_CONFIG_TREE_JSON: &str = r#"{
  "exclude_dirs": [],
  "exclude_files": [],
  "include_extensions": [],
  "inline_threshold": 6,
  "icons": {
    "directory": "📁",
    "file": "📄",
    "by_ext": {
      ".md": "📖",
      ".rs": "🔷",
      ".ts": "🟦",
      ".js": "🟨",
      ".json": "⚙️",
      ".toml": "⚙️",
      ".yaml": "🗂️",
      ".yml": "🗂️",
      ".py": "🐍",
      ".sh": "📜",
      ".ps1": "📜"
    }
  }
}
"#;
