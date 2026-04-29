use std::path::Path;
use std::fs;

use crate::custom::CustomTemplates;

/// Generate an annotation template for the given directory.
///
/// If `template` is `Some`, uses it as the template with `{{token}}` substitution.
/// If `None`, falls back to the hardcoded default output.
///
/// Tokens available in custom templates:
/// - `{{module_name}}` — title-cased directory name
/// - `{{c4_level}}` — "container" or "component"
/// - `{{file_rows}}` — pre-rendered table rows (one per source file, with `//! ` prefix)
pub fn suggest_annotation(dir: &Path, template: Option<&str>) -> String {
    let c4_level = infer_c4_level(dir);
    let source_files = scan_source_files(dir);
    let module_name = derive_module_name(dir);

    let file_rows = source_files
        .iter()
        .map(|f| format!("//! | `{}` | -- | [TODO] | active |", f))
        .collect::<Vec<_>>()
        .join("\n");

    if let Some(tmpl) = template {
        return CustomTemplates::substitute(
            tmpl,
            &[
                ("module_name", &module_name),
                ("c4_level", c4_level),
                ("file_rows", &file_rows),
            ],
        );
    }

    // Hardcoded default (fallback when no .archidoc/custom/mod.rs exists)
    let mut output = String::new();
    output.push_str(&format!("//! @c4 {}\n", c4_level));
    output.push_str("//!\n");
    output.push_str(&format!("//! # {}\n", module_name));
    output.push_str("//!\n");
    output.push_str("//! [TODO: describe this module's responsibility]\n");

    if !source_files.is_empty() {
        output.push_str("//!\n");
        output.push_str("//! | File | Pattern | Purpose | Health |\n");
        output.push_str("//! |------|---------|---------|--------|\n");
        for file in &source_files {
            output.push_str(&format!("//! | `{}` | -- | [TODO] | active |\n", file));
        }
    }

    output
}

/// Infer C4 level from directory depth relative to src/.
/// Depth 1 (e.g., src/api/) = "container", depth 2+ (e.g., src/api/auth/) = "component".
/// If no src/ ancestor found, defaults to "container".
pub fn infer_c4_level(dir: &Path) -> &'static str {
    let components: Vec<_> = dir.components().collect();

    // Find the position of "src" in the path
    let src_position = components.iter().position(|c| {
        c.as_os_str().to_string_lossy() == "src"
    });

    match src_position {
        Some(pos) => {
            let depth_after_src = components.len() - pos - 1;
            if depth_after_src == 1 {
                "container"
            } else {
                "component"
            }
        }
        None => "container",
    }
}

/// Scan directory for source files, excluding entry files.
/// Returns sorted list of filenames (not full paths).
pub fn scan_source_files(dir: &Path) -> Vec<String> {
    let entry_files = [
        "mod.rs",
        "lib.rs",
        "main.rs",
        "index.ts",
        "index.js",
        "__init__.py",
    ];

    let source_extensions = [".rs", ".ts", ".js", ".py"];

    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut files = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name() else {
            continue;
        };

        let file_name_str = file_name.to_string_lossy();

        // Skip entry files
        if entry_files.contains(&file_name_str.as_ref()) {
            continue;
        }

        // Check for source extensions
        let has_source_ext = source_extensions.iter().any(|ext| file_name_str.ends_with(ext));
        if has_source_ext {
            files.push(file_name_str.to_string());
        }
    }

    files.sort();
    files
}

/// Derive module name from directory name, converting to title case.
fn derive_module_name(dir: &Path) -> String {
    let dir_name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Module");

    // Replace underscores with spaces and capitalize first letter
    let with_spaces = dir_name.replace('_', " ");
    let mut chars = with_spaces.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn container_inferred_at_depth_one() {
        let tmp = TempDir::new().unwrap();
        let api_dir = tmp.path().join("src").join("api");
        fs::create_dir_all(&api_dir).unwrap();

        assert_eq!(infer_c4_level(&api_dir), "container");
    }

    #[test]
    fn component_inferred_at_depth_two() {
        let tmp = TempDir::new().unwrap();
        let auth_dir = tmp.path().join("src").join("api").join("auth");
        fs::create_dir_all(&auth_dir).unwrap();

        assert_eq!(infer_c4_level(&auth_dir), "component");
    }

    #[test]
    fn entry_files_excluded() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("test_module");
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("mod.rs"), "").unwrap();
        fs::write(dir.join("lib.rs"), "").unwrap();
        fs::write(dir.join("routes.rs"), "").unwrap();
        fs::write(dir.join("utils.rs"), "").unwrap();

        let files = scan_source_files(&dir);
        assert_eq!(files, vec!["routes.rs", "utils.rs"]);
    }

    #[test]
    fn source_files_discovered() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("test_module");
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("handler.rs"), "").unwrap();
        fs::write(dir.join("service.ts"), "").unwrap();
        fs::write(dir.join("readme.txt"), "").unwrap();
        fs::write(dir.join("data.json"), "").unwrap();

        let files = scan_source_files(&dir);
        assert_eq!(files, vec!["handler.rs", "service.ts"]);
    }

    #[test]
    fn todo_placeholders_present() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("src").join("api");
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("routes.rs"), "").unwrap();

        let annotation = suggest_annotation(&dir, None);
        assert!(annotation.contains("[TODO]"));
        assert!(annotation.contains("[TODO: describe this module's responsibility]"));
    }

    #[test]
    fn annotation_uses_at_c4_syntax() {
        let tmp = TempDir::new().unwrap();
        let container_dir = tmp.path().join("src").join("api");
        fs::create_dir_all(&container_dir).unwrap();

        let annotation = suggest_annotation(&container_dir, None);
        assert!(annotation.contains("@c4 container"));

        let component_dir = tmp.path().join("src").join("api").join("auth");
        fs::create_dir_all(&component_dir).unwrap();

        let annotation = suggest_annotation(&component_dir, None);
        assert!(annotation.contains("@c4 component"));
    }

    #[test]
    fn custom_template_tokens_substituted() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("src").join("api");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("routes.rs"), "").unwrap();

        let tmpl = "//! @c4 {{c4_level}}\n//! # {{module_name}}\n{{file_rows}}";
        let annotation = suggest_annotation(&dir, Some(tmpl));
        assert!(annotation.contains("@c4 container"));
        assert!(annotation.contains("# Api"));
        assert!(annotation.contains("routes.rs"));
    }

    #[test]
    fn missing_template_falls_back_to_default() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("src").join("api");
        fs::create_dir_all(&dir).unwrap();

        let annotation = suggest_annotation(&dir, None);
        assert!(annotation.contains("//! @c4"));
        assert!(annotation.contains("[TODO: describe this module's responsibility]"));
    }
}
