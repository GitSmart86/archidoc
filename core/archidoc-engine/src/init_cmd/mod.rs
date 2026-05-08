//! Init command — generates files from environment context.
//!
//! Each handler reads the target directory, applies a template, and produces output.
//! Handlers are registered by name and dispatched via `archidoc init <handler> [target-dir]`.

pub mod annotation_handler;
pub mod handler;
pub mod index_handler;
pub mod root_annotation_handler;
pub mod tree_handler;

pub use handler::{HandlerArgs, InitHandler, InitOutput};

use annotation_handler::AnnotationHandler;
use index_handler::IndexHandler;
use root_annotation_handler::RootAnnotationHandler;
use tree_handler::TreeHandler;

/// Return all registered init handlers.
pub fn all_handlers() -> Vec<Box<dyn InitHandler>> {
    vec![
        Box::new(IndexHandler),
        Box::new(AnnotationHandler),
        Box::new(RootAnnotationHandler),
        Box::new(TreeHandler),
    ]
}

/// Find a handler by name.
pub fn find_handler(name: &str) -> Option<Box<dyn InitHandler>> {
    all_handlers().into_iter().find(|h| h.name() == name)
}
