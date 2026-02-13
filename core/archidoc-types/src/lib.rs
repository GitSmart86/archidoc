#![allow(rustdoc::invalid_html_tags)]
//! # Extract Docs Types <<component>>
//!
//! Shared type definitions for the archidoc toolchain.
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `module_doc.rs` | -- | Core data structures | planned |
//! | `annotation.rs` | -- | Annotation spec enums | planned |

pub mod annotation;
pub mod module_doc;
pub mod report;

pub use annotation::{HealthStatus, PatternStatus};
pub use module_doc::{C4Level, FileEntry, ModuleDoc, Relationship};
pub use report::{
    DriftReport, DriftedFile, ElementHealth, GhostEntry, HealthReport, OrphanEntry,
    ValidationReport,
};
