#![allow(rustdoc::invalid_html_tags)]
//! @c4 container
//! # Extract Docs Engine
//!
//! Language-agnostic generator engine — reads ModuleDoc[], produces documentation and diagrams.
//! Also provides the init handler registry and folder scaffold engine.
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `architecture.rs` | -- | Single ARCHITECTURE.md generator | stable |
//! | `ai_context.rs` | -- | Token-optimized AI context generator | stable |
//! | `mermaid.rs` | -- | Mermaid C4 diagram generation | stable |
//! | `drawio.rs` | -- | draw.io CSV generation | stable |
//! | `plantuml.rs` | -- | PlantUML C4 diagram generation | stable |
//! | `ir.rs` | -- | JSON IR serialization and validation | stable |
//! | `check.rs` | -- | Documentation drift detection | stable |
//! | `health.rs` | -- | Health report aggregation | stable |
//! | `validate.rs` | -- | Ghost and orphan detection | stable |
//! | `init.rs` | -- | Root-level project template generator (used by root-annotation handler) | stable |
//! | `suggest.rs` | -- | Annotation scaffolding templates (used by c4-annotation handler) | stable |
//! | `scaffold.rs` | -- | _index.md stub generation (used by _index.md handler) | stable |
//! | `tree.rs` | -- | Token-optimized directory tree generation (used by tree handler) | stable |
//! | `merge.rs` | -- | Polyglot IR merging | stable |
//! | `custom.rs` | -- | Template loading and token substitution | stable |
//! | `init_cmd/` | Registry | Init handler registry — dispatches `archidoc init <handler>` | active |
//! | `folder_scaffold/` | -- | Folder template engine — `archidoc scaffold <name>` | active |

pub mod ai_context;
pub mod architecture;
pub mod check;
pub mod custom;
pub mod drawio;
pub mod folder_scaffold;
pub mod health;
pub mod init;
pub mod init_cmd;
pub mod ir;
pub mod merge;
pub mod mermaid;
pub mod plantuml;
pub mod scaffold;
pub mod suggest;
pub mod tree;
pub mod validate;
