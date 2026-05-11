use std::collections::BTreeMap;
use std::fmt;

use archidoc_types::scaffold_ir::ScaffoldTemplate;

#[derive(Debug)]
pub enum VariableError {
    Missing(Vec<String>),
}

impl fmt::Display for VariableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VariableError::Missing(names) => {
                write!(f, "missing required variable(s): {}", names.join(", "))
            }
        }
    }
}

/// Merge CLI-supplied variables with manifest defaults, then validate required fields.
///
/// Priority (highest wins): CLI `--var` flags > manifest variable `default`.
pub fn collect(
    template: &ScaffoldTemplate,
    cli_vars: &[(String, String)],
) -> Result<BTreeMap<String, String>, VariableError> {
    let mut vars: BTreeMap<String, String> = BTreeMap::new();

    // Seed defaults from manifest
    for var in &template.variables {
        if let Some(ref default) = var.default {
            vars.insert(var.name.clone(), default.clone());
        }
    }

    // CLI vars override defaults
    for (key, value) in cli_vars {
        vars.insert(key.clone(), value.clone());
    }

    // Validate required
    let missing: Vec<String> = template
        .variables
        .iter()
        .filter(|v| v.required && !vars.contains_key(&v.name))
        .map(|v| v.name.clone())
        .collect();

    if !missing.is_empty() {
        return Err(VariableError::Missing(missing));
    }

    Ok(vars)
}

#[cfg(test)]
mod tests {
    use super::*;
    use archidoc_types::scaffold_ir::{ScaffoldTemplate, ScaffoldVariable};

    fn template(vars: Vec<ScaffoldVariable>) -> ScaffoldTemplate {
        ScaffoldTemplate {
            name: "test".to_string(),
            description: "test".to_string(),
            variables: vars,
            post_hooks: vec![],
        }
    }

    fn required(name: &str) -> ScaffoldVariable {
        ScaffoldVariable {
            name: name.to_string(),
            description: name.to_string(),
            required: true,
            default: None,
        }
    }

    fn optional(name: &str, default: &str) -> ScaffoldVariable {
        ScaffoldVariable {
            name: name.to_string(),
            description: name.to_string(),
            required: false,
            default: Some(default.to_string()),
        }
    }

    #[test]
    fn cli_vars_override_defaults() {
        let t = template(vec![optional("env", "dev")]);
        let cli = vec![("env".to_string(), "prod".to_string())];
        let result = collect(&t, &cli).unwrap();
        assert_eq!(result["env"], "prod");
    }

    #[test]
    fn default_used_when_no_cli_var() {
        let t = template(vec![optional("env", "dev")]);
        let result = collect(&t, &[]).unwrap();
        assert_eq!(result["env"], "dev");
    }

    #[test]
    fn missing_required_errors() {
        let t = template(vec![required("module_name")]);
        let err = collect(&t, &[]).unwrap_err();
        assert!(matches!(err, VariableError::Missing(ref names) if names.contains(&"module_name".to_string())));
    }

    #[test]
    fn required_satisfied_by_cli() {
        let t = template(vec![required("module_name")]);
        let cli = vec![("module_name".to_string(), "auth".to_string())];
        let result = collect(&t, &cli).unwrap();
        assert_eq!(result["module_name"], "auth");
    }

    #[test]
    fn empty_template_and_no_cli_vars_ok() {
        let t = template(vec![]);
        let result = collect(&t, &[]).unwrap();
        assert!(result.is_empty());
    }
}
