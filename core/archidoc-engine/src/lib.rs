#![allow(rustdoc::invalid_html_tags)]
//! @c4 container
//! # Extract Docs Engine
//!
//! Language-agnostic generator engine — reads ModuleDoc[], produces documentation and diagrams.
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `architecture.rs` | -- | Single ARCHITECTURE.md generator | stable |
//! | `ai_context.rs` | -- | Token-optimized AI context generator | active |
//! | `mermaid.rs` | -- | Mermaid C4 diagram generation | stable |
//! | `drawio.rs` | -- | draw.io CSV generation | stable |
//! | `plantuml.rs` | -- | PlantUML C4 diagram generation | stable |
//! | `ir.rs` | -- | JSON IR serialization and validation | stable |
//! | `check.rs` | -- | Documentation drift detection | stable |
//! | `health.rs` | -- | Health report aggregation | stable |
//! | `validate.rs` | -- | Ghost and orphan detection | stable |
//! | `init.rs` | -- | Root-level project template generator | active |
//! | `suggest.rs` | -- | Annotation scaffolding templates | active |
//! | `merge.rs` | -- | Polyglot IR merging | active |

pub mod ai_context;
pub mod architecture;
pub mod check;
pub mod drawio;
pub mod health;
pub mod init;
pub mod ir;
pub mod merge;
pub mod mermaid;
pub mod plantuml;
pub mod suggest;
pub mod validate;
