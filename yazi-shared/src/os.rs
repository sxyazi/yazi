#[cfg(unix)]
pub static USERS_CACHE: crate::RoCell<uzers::UsersCache> = crate::RoCell::new();

#[cfg(unix)]
pub fn hostname() -> Result<String, std::io::Error> {
	use std::io::{Error, ErrorKind};

	use libc::{gethostname, strlen};

	let mut s = [0; 256];
	let len = unsafe {
		if gethostname(s.as_mut_ptr() as *mut _, 255) == -1 {
			return Err(std::io::Error::last_os_error());
		}

		strlen(s.as_ptr() as *const _)
	};

	std::str::from_utf8(&s[..len])
		.map_err(|_| Error::new(ErrorKind::Other, "invalid hostname"))
		.map(|s| s.to_owned())
}
