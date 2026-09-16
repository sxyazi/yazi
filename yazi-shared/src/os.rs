#[cfg(unix)]
pub fn hostname() -> Option<&'static str> {
	static CACHE: std::sync::LazyLock<Option<String>> = std::sync::LazyLock::new(|| {
		rustix::system::uname().nodename().to_str().ok().map(str::to_owned)
	});

	CACHE.as_deref()
}

#[cfg(unix)]
pub fn session_leader() -> bool {
	rustix::process::getsid(None).is_ok_and(|sid| sid == rustix::process::getpid())
}
