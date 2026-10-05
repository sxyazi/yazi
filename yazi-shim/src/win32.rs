use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt, path::Path};

use windows_sys::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};

/// Maps a Win32 non-zero-success return value to `io::Result<T>`,
/// 0 becomes `Err(last_os_error())`.
pub fn nz_ok<T: PartialEq + From<u8>>(r: T) -> io::Result<T> {
	if r == 0.into() { Err(io::Error::last_os_error()) } else { Ok(r) }
}

/// Maps a Win32 BOOL return value to `io::Result<()>`,
/// 0 becomes `Err(last_os_error())`.
pub fn bool_ok(r: i32) -> io::Result<()> { nz_ok(r).map(|_| ()) }

/// Maps `INVALID_HANDLE_VALUE` to `Err(last_os_error())`; null is not checked.
pub fn handle_ok(handle: HANDLE) -> io::Result<HANDLE> {
	if handle == INVALID_HANDLE_VALUE { Err(io::Error::last_os_error()) } else { Ok(handle) }
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
