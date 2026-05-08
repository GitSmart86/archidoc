use std::collections::BTreeMap;
use std::path::Path;

use super::handler::{HandlerArgs, InitHandler, InitOutput};
use crate::init::{CommentStyle, wrap_jsdoc};

/// Handler for `root-annotation` — generates root-level lib.rs/index.ts template.
///
/// Replaces the old `archidoc init [--lang <lang>]` command.
/// Output is printed to stdout.
pub struct RootAnnotationHandler;

impl InitHandler for RootAnnotationHandler {
    fn name(&self) -> &str {
        "root-annotation"
    }

    fn description(&self) -> &str {
        "Generate root-level lib.rs/index.ts annotation template (was: init)"
    }

    fn generate(
        &self,
        target_dir: &Path,
        _vars: &BTreeMap<String, String>,
        extra_args: &HandlerArgs,
    ) -> Result<Vec<InitOutput>, String> {
        let style = if let Some(lang) = &extra_args.lang {
            CommentStyle::from_lang(lang).ok_or_else(|| {
                format!("unsupported language '{}' (try: rust, ts)", lang)
            })?
        } else {
            CommentStyle::detect(target_dir).ok_or_else(|| {
                "could not detect language (no Cargo.toml or package.json). Use --lang rust or --lang ts".to_string()
            })?
        };

        let template = crate::init::generate_template(style);
        let content = match style {
            CommentStyle::TypeScript => wrap_jsdoc(&template),
            _ => template,
        };

        Ok(vec![InitOutput {
            path: std::path::PathBuf::from("-"), // stdout sentinel
            contents: content,
        }])
    }
}
