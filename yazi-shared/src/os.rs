#[cfg(unix)]
pub static USERS_CACHE: crate::RoCell<uzers::UsersCache> = crate::RoCell::new();

#[cfg(unix)]
pub fn hostname() -> Option<&'static str> {
	static CACHE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

	CACHE.get_or_init(|| hostname_impl().ok()).as_deref()
}

#[cfg(unix)]
fn hostname_impl() -> Result<String, std::io::Error> {
	use libc::{gethostname, strlen};

	let mut s = [0; 256];
	let len = unsafe {
		if gethostname(s.as_mut_ptr() as *mut _, 255) == -1 {
			return Err(std::io::Error::last_os_error());
		}

		strlen(s.as_ptr() as *const _)
	};

	std::str::from_utf8(&s[..len])
		.map_err(|_| std::io::Error::other("invalid hostname"))
		.map(|s| s.to_owned())
}
