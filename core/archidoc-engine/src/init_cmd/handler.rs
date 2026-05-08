use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Output from an init handler — one or more files to write.
pub struct InitOutput {
    pub path: PathBuf,
    pub contents: String,
}

/// Trait for init handlers that generate files from environment context.
///
/// Each handler is registered by name. The CLI dispatches to the matching handler
/// based on the `<handler>` argument to `archidoc init <handler> [target-dir]`.
pub trait InitHandler {
    /// Handler name (matches the CLI argument).
    fn name(&self) -> &str;

    /// Short description for `--list` output.
    fn description(&self) -> &str;

    /// Generate file contents for the given target directory.
    ///
    /// `target_dir` is the directory to read context from / write into.
    /// `vars` contains any extra key-value pairs from `--var` flags.
    /// `extra_args` contains handler-specific flags (e.g., `--files`, `--human` for tree).
    fn generate(
        &self,
        target_dir: &Path,
        vars: &BTreeMap<String, String>,
        extra_args: &HandlerArgs,
    ) -> Result<Vec<InitOutput>, String>;

    /// Whether this handler supports `--dry-run` (listing what would be created without writing).
    fn supports_dry_run(&self) -> bool {
        false
    }

    /// Dry-run: list what would be generated without writing.
    /// Only called if `supports_dry_run()` returns true.
    fn dry_run(
        &self,
        target_dir: &Path,
        vars: &BTreeMap<String, String>,
        extra_args: &HandlerArgs,
    ) -> Result<Vec<String>, String> {
        let _ = (target_dir, vars, extra_args);
        Ok(vec![])
    }
}

/// Handler-specific arguments passed through from the CLI.
#[derive(Debug, Default)]
pub struct HandlerArgs {
    /// --files flag (tree handler)
    pub files: bool,
    /// --human flag (tree handler)
    pub human: bool,
    /// --both flag (tree handler)
    pub both: bool,
    /// --depth N (tree handler)
    pub depth: Option<usize>,
    /// --lang <name> (root-annotation handler)
    pub lang: Option<String>,
    /// --out <dir> (tree handler output directory override)
    pub out: Option<PathBuf>,
}
