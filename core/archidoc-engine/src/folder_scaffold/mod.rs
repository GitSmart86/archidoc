//! Scaffold folder templates — copies named template trees with `{{variable}}` substitution.
//!
//! Templates live in `.archidoc/templates/scaffold-folder-templates/<name>/`.
//! Walk-up discovery finds the nearest matching template from the current directory upward.

pub mod discover;
pub mod execute;
pub mod plan;
pub mod types;
pub mod variables;

pub use discover::{discover_template, list_templates, load_manifest};
pub use execute::execute_plan;
pub use plan::build_plan;
pub use types::{
    ActionOutcome, HookResult, PostHook, ScaffoldAction, ScaffoldError, ScaffoldPlan,
    ScaffoldResult, TemplateManifest, TemplateVariable,
};
pub use variables::collect_variables;
