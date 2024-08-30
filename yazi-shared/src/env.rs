#[inline]
pub fn env_exists(name: &str) -> bool { std::env::var_os(name).is_some_and(|s| !s.is_empty()) }

#[inline]
pub fn in_wsl() -> bool {
	#[cfg(target_os = "linux")]
	{
		std::fs::symlink_metadata("/proc/sys/fs/binfmt_misc/WSLInterop").is_ok()
	}
	#[cfg(not(target_os = "linux"))]
	{
		false
	}
}

#[inline]
pub fn in_ssh_connection() -> bool {
	env_exists("SSH_CLIENT") || env_exists("SSH_TTY") || env_exists("SSH_CONNECTION")
}
