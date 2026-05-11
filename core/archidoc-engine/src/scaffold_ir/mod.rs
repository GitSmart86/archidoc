//! ScaffoldIR — JSON-based scaffold template system.
//!
//! A ScaffoldIR template is a single `.json` file that describes a directory
//! structure to instantiate. Templates compose via `$ref` node references.
//!
//! | File | Purpose |
//! |------|---------|
//! | `resolver.rs` | `$ref` resolution pre-pass |
//! | `executor.rs` | Variable substitution + directory instantiation |
//! | `builtins.rs` | Embedded built-in templates |
//! | `discover.rs` | Walk-up `.archidoc/scaffolds/<name>.json` finder |
//! | `variables.rs` | Variable collection and validation |
//! | `compile.rs` | Compile a folder template directory into a ScaffoldIR JSON |

pub mod builtins;
pub mod compile;
pub mod discover;
pub mod executor;
pub mod resolver;
pub mod variables;

pub use builtins::{is_builtin, load_builtin, BUILTIN_NAMES};
pub use discover::{find, list};
pub use executor::{dry_run, execute, ActionOutcome, ExecuteError, ExecuteResult, HookResult, PlannedAction};
pub use resolver::{resolve, ResolveError};
pub use variables::{collect, VariableError};
