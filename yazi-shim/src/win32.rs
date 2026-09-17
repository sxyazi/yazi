use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt, path::Path};

/// Maps a Win32 BOOL return value to `io::Result<()>`,
/// 0 becomes `Err(last_os_error())`.
pub fn bool_ok(r: i32) -> io::Result<()> {
	if r == 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
}

/// Maps a Win32 non-zero-success return value to `io::Result<u32>`,
/// 0 becomes `Err(last_os_error())`.
pub fn nz_ok(r: u32) -> io::Result<u32> {
	if r == 0 { Err(io::Error::last_os_error()) } else { Ok(r) }
}

// --- ToWide
pub trait ToWide {
	fn to_wide(&self) -> Vec<u16>;
}

impl ToWide for OsStr {
	#[inline]
	fn to_wide(&self) -> Vec<u16> { self.encode_wide().chain([0]).collect() }
}

impl ToWide for str {
	#[inline]
	fn to_wide(&self) -> Vec<u16> { OsStr::new(self).to_wide() }
}

impl ToWide for Path {
	#[inline]
	fn to_wide(&self) -> Vec<u16> { self.as_os_str().to_wide() }
}
