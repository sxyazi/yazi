#[inline]
pub fn env_exists(name: &str) -> bool { std::env::var_os(name).is_some_and(|s| !s.is_empty()) }

#[cfg(unix)]
#[inline]
pub fn in_ssh_connection() -> bool {
	env_exists("SSH_CLIENT") || env_exists("SSH_TTY") || env_exists("SSH_CONNECTION")
}
