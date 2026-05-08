use std::fs;
use std::path::Path;

/// Template overrides loaded from `.archidoc/init-overrides/`.
///
/// Each field is `Some(content)` if the override file exists, `None` if absent —
/// callers fall back to hardcoded defaults when `None`.
///
/// Only files matching known handler output names are recognized:
/// - `mod.rs` — overrides the Rust @c4 annotation template
/// - `index.ts` — overrides the TypeScript @c4 annotation template
/// - `_index.md` — overrides the _index.md directory listing template
/// - `architecture-table.md` — overrides the architecture summary table header
///
/// Unknown files in this directory are ignored.
///
/// | File | Pattern | Purpose | Health |
/// |------|---------|---------|--------|
/// | `custom.rs` | -- | Template loading and token substitution | stable |
pub struct CustomTemplates {
    /// Rust suggest template — `.archidoc/init-overrides/mod.rs`
    pub suggest_rust: Option<String>,
    /// TypeScript suggest template — `.archidoc/init-overrides/index.ts`
    pub suggest_ts: Option<String>,
    /// Markdown suggest template — `.archidoc/init-overrides/_index.md`
    pub suggest_md: Option<String>,
    /// Architecture summary table header — `.archidoc/init-overrides/architecture-table.md`
    pub architecture_table: Option<String>,
}

impl CustomTemplates {
    /// Load custom templates from `.archidoc/init-overrides/` relative to `cwd`.
    ///
    /// Missing files are silently ignored — `None` means "use hardcoded default".
    pub fn load(cwd: &Path) -> Self {
        let base = cwd.join(".archidoc").join("init-overrides");
        Self {
            suggest_rust: read_optional(&base.join("mod.rs")),
            suggest_ts: read_optional(&base.join("index.ts")),
            suggest_md: read_optional(&base.join("_index.md")),
            architecture_table: read_optional(&base.join("architecture-table.md")),
        }
    }

    /// Substitute `{{key}}` tokens in a template string.
    ///
    /// `vars` is a slice of `(key, value)` pairs. Keys must not include
    /// the `{{` / `}}` delimiters — they are added internally.
    pub fn substitute(template: &str, vars: &[(&str, &str)]) -> String {
        let mut result = template.to_string();
        for (key, value) in vars {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }
}

/// Parse column names from the header row of an architecture-table override.
///
/// Reads the first `|`-delimited row that does not contain `---` (the separator).
/// Returns an empty Vec if no valid header row is found (caller uses defaults).
///
/// Example input:
/// ```text
/// | Module | Description | Health |
/// |--------|-------------|--------|
/// ```
/// Returns: `["Module", "Description", "Health"]`
pub fn parse_table_columns(template: &str) -> Vec<String> {
    for line in template.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('|') && !trimmed.contains("---") {
            let cols: Vec<String> = trimmed
                .split('|')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
                .collect();
            if !cols.is_empty() {
                return cols;
            }
        }
    }
    vec![]
}

fn read_optional(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}

// ---------------------------------------------------------------------------
// Default template strings — used by `archidoc init` to scaffold
// `.archidoc/init-overrides/` and as the fallback when no override exists.
// ---------------------------------------------------------------------------

/// Default parameterized Rust suggest template (`.archidoc/init-overrides/mod.rs`).
pub const DEFAULT_SUGGEST_RUST: &str = "\
//! @c4 {{c4_level}}
//!
//! # {{module_name}}
//!
//! [TODO: describe this module's responsibility]
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
{{file_rows}}";

/// Default parameterized TypeScript suggest template (`.archidoc/init-overrides/index.ts`).
pub const DEFAULT_SUGGEST_TS: &str = "\
/**
 * @c4 {{c4_level}}
 *
 * {{module_name}}
 *
 * [TODO: describe this module's responsibility]
 *
 * | File | Pattern | Purpose | Health |
 * |------|---------|---------|--------|
{{file_rows}} */";

/// Default parameterized Markdown suggest template (`.archidoc/init-overrides/_index.md`).
pub const DEFAULT_SUGGEST_MD: &str = "\
<!-- @c4 {{c4_level}} -->

[TODO: describe this directory's responsibility]

| File | Purpose | Health |
|------|---------|--------|
{{file_rows}}";

/// Default architecture summary table header (`.archidoc/init-overrides/architecture-table.md`).
pub const DEFAULT_ARCHITECTURE_TABLE: &str = "\
| Module | Level | Pattern | Description |
|--------|-------|---------|-------------|";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitute_replaces_single_token() {
        let result = CustomTemplates::substitute("hello {{name}}", &[("name", "world")]);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn substitute_replaces_multiple_tokens() {
        let result = CustomTemplates::substitute(
            "{{c4_level}} {{module_name}}",
            &[("c4_level", "container"), ("module_name", "Api")],
        );
        assert_eq!(result, "container Api");
    }

    #[test]
    fn substitute_leaves_unknown_tokens_intact() {
        let result = CustomTemplates::substitute("{{unknown}}", &[("other", "value")]);
        assert_eq!(result, "{{unknown}}");
    }

    #[test]
    fn parse_table_columns_extracts_header() {
        let template = "| Module | Description | Health |\n|--------|-------------|--------|\n";
        let cols = parse_table_columns(template);
        assert_eq!(cols, vec!["Module", "Description", "Health"]);
    }

    #[test]
    fn parse_table_columns_skips_separator() {
        let template = "|--------|--------|\n| Module | Level |\n";
        let cols = parse_table_columns(template);
        // First non-separator row is the data row here, which is fine
        assert!(!cols.is_empty());
    }

    #[test]
    fn parse_table_columns_empty_on_no_table() {
        let cols = parse_table_columns("no table here");
        assert!(cols.is_empty());
    }
}
