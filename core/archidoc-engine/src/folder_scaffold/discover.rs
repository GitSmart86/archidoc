use std::path::{Path, PathBuf};

use super::types::{ScaffoldError, TemplateManifest};

const MANIFEST_FILENAME: &str = ".archidoc-template.toml";
const TEMPLATES_DIR: &str = "scaffold-templates";

/// Walk up from `start_dir` to find a scaffold folder template by name.
///
/// Searches `.archidoc/scaffold-templates/<name>/` at each
/// directory level, returning the first match. Nearest wins.
pub fn discover_template(name: &str, start_dir: &Path) -> Result<PathBuf, ScaffoldError> {
    let mut cursor = if start_dir.is_absolute() {
        start_dir.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(ScaffoldError::Io)?
            .join(start_dir)
    };

    loop {
        let candidate = cursor
            .join(".archidoc")
            .join(TEMPLATES_DIR)
            .join(name);

        if candidate.join(MANIFEST_FILENAME).exists() {
            return Ok(candidate);
        }

        if !cursor.pop() {
            return Err(ScaffoldError::TemplateNotFound(name.to_string()));
        }
    }
}

/// List all available scaffold folder templates, walking up from `start_dir`.
///
/// Returns `(name, template_dir, manifest)` tuples. Nearest templates shadow
/// outer ones with the same name.
pub fn list_templates(
    start_dir: &Path,
) -> Vec<(String, PathBuf, TemplateManifest)> {
    let mut found: Vec<(String, PathBuf, TemplateManifest)> = Vec::new();
    let mut seen_names: Vec<String> = Vec::new();

    let mut cursor = if start_dir.is_absolute() {
        start_dir.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(start_dir)
    };

    loop {
        let templates_dir = cursor
            .join(".archidoc")
            .join(TEMPLATES_DIR);

        if templates_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&templates_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    let name = entry.file_name().to_string_lossy().to_string();
                    if seen_names.contains(&name) {
                        continue; // nearest wins
                    }
                    let manifest_path = path.join(MANIFEST_FILENAME);
                    if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                        if let Ok(manifest) = toml::from_str::<TemplateManifest>(&content) {
                            seen_names.push(name.clone());
                            found.push((name, path, manifest));
                        }
                    }
                }
            }
        }

        if !cursor.pop() {
            break;
        }
    }

    found
}

/// Load and parse the manifest for a discovered template directory.
pub fn load_manifest(template_dir: &Path) -> Result<TemplateManifest, ScaffoldError> {
    let manifest_path = template_dir.join(MANIFEST_FILENAME);
    let content = std::fs::read_to_string(&manifest_path).map_err(|e| {
        ScaffoldError::InvalidManifest {
            path: manifest_path.clone(),
            source: e.to_string(),
        }
    })?;
    toml::from_str(&content).map_err(|e| ScaffoldError::InvalidManifest {
        path: manifest_path,
        source: e.to_string(),
    })
}
