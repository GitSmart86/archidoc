#![allow(rustdoc::invalid_html_tags)]
//! # Extract Docs Engine <<container>>
//!
//! Language-agnostic generator engine — reads ModuleDoc[], produces documentation and diagrams.
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `markdown.rs` | -- | Per-module .md and index generation | planned |
//! | `mermaid.rs` | -- | Mermaid C4 diagram generation | planned |
//! | `drawio.rs` | -- | draw.io CSV generation | planned |
//! | `ir.rs` | -- | JSON IR serialization and validation | planned |
//! | `check.rs` | -- | Documentation drift detection | planned |
//! | `health.rs` | -- | Health report aggregation | planned |
//! | `validate.rs` | -- | Ghost and orphan detection | planned |

pub mod check;
pub mod drawio;
pub mod health;
pub mod ir;
pub mod markdown;
pub mod mermaid;
pub mod validate;
