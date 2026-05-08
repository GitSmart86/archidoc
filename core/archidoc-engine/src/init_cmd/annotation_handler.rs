use std::collections::BTreeMap;
use std::path::Path;

use super::handler::{HandlerArgs, InitHandler, InitOutput};
use crate::custom::CustomTemplates;
use crate::suggest;

/// Handler for `c4-annotation` — generates @c4 annotation block for a directory.
///
/// Replaces the old `archidoc suggest <dir>` command.
/// Output is printed to stdout (path is a placeholder).
pub struct AnnotationHandler;

impl InitHandler for AnnotationHandler {
    fn name(&self) -> &str {
        "c4-annotation"
    }

    fn description(&self) -> &str {
        "Generate @c4 annotation template for a directory (was: suggest)"
    }

    fn generate(
        &self,
        target_dir: &Path,
        _vars: &BTreeMap<String, String>,
        _extra_args: &HandlerArgs,
    ) -> Result<Vec<InitOutput>, String> {
        if !target_dir.exists() {
            return Err(format!("path does not exist: {}", target_dir.display()));
        }
        if !target_dir.is_dir() {
            return Err(format!("path is not a directory: {}", target_dir.display()));
        }

        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        let custom = CustomTemplates::load(&cwd);
        let annotation =
            suggest::suggest_annotation(target_dir, custom.suggest_rust.as_deref());

        // This handler outputs to stdout, not a file — use a sentinel path
        Ok(vec![InitOutput {
            path: std::path::PathBuf::from("-"), // stdout sentinel
            contents: annotation,
        }])
    }
}
