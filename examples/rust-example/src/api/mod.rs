//! @c4 container
//! # Api
//!
//! REST API gateway — handles authentication and request routing.
//!
//! @c4 uses database "Persists user data" "sqlx"
//! @c4 uses events "Publishes domain events" "channel"
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `routes.rs` | -- | HTTP route handlers | active |
//! | `middleware.rs` | Strategy | Auth and rate limiting | stable |
//! | `errors.rs` | -- | Error types and conversions | stable |
