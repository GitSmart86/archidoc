use std::collections::BTreeMap;
use std::path::Path;

use super::handler::{HandlerArgs, InitHandler, InitOutput};
use crate::scaffold;

/// Handler for `_index.md` — generates directory listing tables for all dirs missing one.
///
/// Replaces the old `archidoc scaffold` (stubs) and `archidoc audit` commands.
/// With `--dry-run`, lists missing dirs without creating files (was: `audit`).
pub struct IndexHandler;

impl InitHandler for IndexHandler {
    fn name(&self) -> &str {
        "_index.md"
    }

    fn description(&self) -> &str {
        "Generate _index.md directory listing for all dirs missing one (was: scaffold/audit)"
    }

    fn generate(
        &self,
        target_dir: &Path,
        _vars: &BTreeMap<String, String>,
        _extra_args: &HandlerArgs,
    ) -> Result<Vec<InitOutput>, String> {
        let missing = scaffold::find_missing(target_dir);
        let mut outputs = Vec::new();

        for dir in missing {
            let index_path = dir.join("_index.md");
            if index_path.exists() {
                continue;
            }
            let content = scaffold::scaffold_stub(&dir, target_dir);
            outputs.push(InitOutput {
                path: index_path,
                contents: content,
            });
        }

        Ok(outputs)
    }

    fn supports_dry_run(&self) -> bool {
        true
    }

    fn dry_run(
        &self,
        target_dir: &Path,
        _vars: &BTreeMap<String, String>,
        _extra_args: &HandlerArgs,
    ) -> Result<Vec<String>, String> {
        let missing = scaffold::find_missing(target_dir);
        Ok(missing
            .iter()
            .map(|d| {
                let display = d.strip_prefix(target_dir).unwrap_or(d);
                display.display().to_string()
            })
            .collect())
    }
}
