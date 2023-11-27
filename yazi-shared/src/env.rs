#[inline]
pub fn env_exists(name: &str) -> bool { std::env::var_os(name).is_some_and(|s| !s.is_empty()) }
