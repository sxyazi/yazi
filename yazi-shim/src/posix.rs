use std::io;

#[inline]
pub fn nonneg_ok(value: i32) -> io::Result<()> {
	if value < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
}
