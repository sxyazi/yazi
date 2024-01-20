#[cfg(unix)]
pub fn hostname() -> Result<String, std::io::Error> {
	use libc::{gethostname, strlen};

	let mut s = Vec::<u8>::with_capacity(256);
	unsafe {
		if gethostname(s.as_mut_ptr() as *mut _, 255) == -1 {
			return Err(std::io::Error::last_os_error());
		}

		s.set_len(strlen(s.as_ptr() as *const _));
		Ok(String::from_utf8_unchecked(s))
	}
}
