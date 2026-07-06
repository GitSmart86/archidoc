#![allow(rustdoc::invalid_html_tags)]
//! @c4 container
//! # Extract Docs Engine
//!
//! Language-agnostic generator engine — reads ArchitectureIR, produces documentation and diagrams.
//! Also provides the scaffold template engine.
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `architecture.rs` | -- | Single ARCHITECTURE.md generator | stable |
//! | `ai_context.rs` | -- | Token-optimized AI context generator | stable |
//! | `mermaid.rs` | -- | Mermaid C4 diagram generation | stable |
//! | `drawio.rs` | -- | draw.io CSV generation | stable |
//! | `plantuml.rs` | -- | PlantUML C4 diagram generation | stable |
//! | `ir.rs` | -- | JSON IR serialization (v2.0 unified tree) | stable |
//! | `ir_builder.rs` | -- | IR build pipeline: scan → overlay annotations → resolve parents | active |
//! | `design_validate.rs` | -- | Design vs actual IR diff engine — conformance validation | active |
//! | `check.rs` | -- | Documentation drift detection | stable |
//! | `health.rs` | -- | Health report aggregation | stable |
//! | `validate.rs` | -- | Ghost and orphan detection | stable |
//! | `tree.rs` | -- | Token-optimized directory tree generation | stable |
//! | `merge.rs` | -- | Polyglot IR merging | stable |
//! | `custom.rs` | -- | Template loading and token substitution | stable |
//! | `annotate.rs` | -- | Directory annotation generation for rs/ts/js/md | stable |
//! | `diagnostics.rs` | -- | Unified diagnostic engine — errors and warnings for --validate | stable |
//! | `narrative_context.rs` | -- | Discovery and parsing of `.archidoc/narrative-context.md` source file | active |
//! | `scaffold_ir/` | -- | JSON scaffold template engine — `archidoc scaffold <name>` | active |

pub mod ai_context;
pub mod annotate;
pub mod diagnostics;
pub mod architecture;
pub mod check;
pub mod context;
pub mod coverage;
pub mod custom;
pub mod design_validate;
pub mod drawio;
pub mod filemap;
pub mod health;
pub mod ir;
pub mod ir_builder;
pub mod ir_query;
pub mod merge;
pub mod mermaid;
pub mod narrative_context;
pub mod plantuml;
pub mod scaffold_ir;
pub mod tree;
pub mod validate;
