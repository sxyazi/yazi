use std::io;

/// Maps a Win32 BOOL return value to `io::Result<()>`; 0 becomes
/// `Err(last_os_error())`.
pub fn bool_ok(r: i32) -> io::Result<()> {
	if r == 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
}

/// Maps a Win32 non-zero-success return value to `io::Result<u32>`; 0 becomes
/// `Err(last_os_error())`.
pub fn nz_ok(r: u32) -> io::Result<u32> {
	if r == 0 { Err(io::Error::last_os_error()) } else { Ok(r) }
}
