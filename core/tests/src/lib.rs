//! # Extract Docs Tests <<component>>
//!
//! BDD test infrastructure — DSL, protocol drivers, fakes.
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `dsl/` | Facade | Domain-specific language layer | active |
//! | `drivers/` | Strategy | Protocol driver traits and implementations | active |
//! | `fakes/` | -- | Test doubles for source tree creation | active |
//! | `params.rs` | -- | String parameter parser | stable |

pub mod dsl;
pub mod drivers;
pub mod fakes;
pub mod params;

pub use dsl::ArchitectureDsl;
