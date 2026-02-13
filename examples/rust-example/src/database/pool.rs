// Connection pool management
use std::sync::OnceLock;

static POOL: OnceLock<String> = OnceLock::new();

pub fn get_pool() -> &'static str {
    POOL.get_or_init(|| "postgres://localhost/app".to_string())
}
