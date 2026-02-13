//! @c4 container
//! # Database
//!
//! Persistence layer — manages database connections and queries.
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `pool.rs` | Singleton | Connection pool management | stable |
//! | `queries.rs` | Repository | SQL query implementations | active |
