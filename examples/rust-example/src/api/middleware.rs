// Auth and rate limiting middleware
pub trait AuthStrategy {
    fn authenticate(&self, token: &str) -> bool;
}
