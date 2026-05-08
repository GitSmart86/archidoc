use std::collections::BTreeMap;

use super::types::{ScaffoldError, TemplateManifest};

/// Collect and validate variables for a scaffold operation.
///
/// Priority (highest wins):
/// 1. CLI `--var` flags
/// 2. Variable defaults from manifest
///
/// Returns error if any required variables are missing after merging.
pub fn collect_variables(
    manifest: &TemplateManifest,
    cli_vars: &[(String, String)],
) -> Result<BTreeMap<String, String>, ScaffoldError> {
    let mut vars = BTreeMap::new();

    // Layer 1: defaults from manifest
    for var_def in &manifest.variables {
        if let Some(default) = &var_def.default {
            vars.insert(var_def.name.clone(), default.clone());
        }
    }

    // Layer 2: CLI vars (override defaults)
    for (key, value) in cli_vars {
        vars.insert(key.clone(), value.clone());
    }

    // Validate: all required variables must have values
    let missing: Vec<String> = manifest
        .variables
        .iter()
        .filter(|v| v.required && !vars.contains_key(&v.name))
        .map(|v| v.name.clone())
        .collect();

    if !missing.is_empty() {
        return Err(ScaffoldError::MissingVariables(missing));
    }

    Ok(vars)
}
