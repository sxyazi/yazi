use std::{ffi::OsStr, io, os::windows::ffi::OsStrExt, path::Path, ptr};

use windows_sys::Win32::{Foundation::{HANDLE, INVALID_HANDLE_VALUE}, Storage::FileSystem::GetFullPathNameW};

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

	/// Long Win32 paths need extended prefixes: follow std's conversion rules, keeping short drive/UNC and already-prefixed paths unchanged.
	fn to_wide_path(&self) -> io::Result<Vec<u16>>;
}

impl ToWide for OsStr {
	#[inline]
	fn to_wide(&self) -> Vec<u16> { self.encode_wide().chain([0]).collect() }

	fn to_wide_path(&self) -> io::Result<Vec<u16>> {
		const SEP: u16 = b'\\' as _;
		const ALT: u16 = b'/' as _;
		const COLON: u16 = b':' as _;
		const QUERY: u16 = b'?' as _;
		const DOT: u16 = b'.' as _;
		const VERBATIM: &[u16] = &[SEP, SEP, QUERY, SEP];
		const NT: &[u16] = &[SEP, QUERY, QUERY, SEP];
		const UNC: &[u16] = &[SEP, SEP, QUERY, SEP, b'U' as _, b'N' as _, b'C' as _, SEP];

		if self.as_encoded_bytes().contains(&0) {
			return Err(io::ErrorKind::InvalidInput.into());
		}

		let mut path = self.to_wide();
		if path.starts_with(VERBATIM) || path.starts_with(NT) || path == [0] {
			return Ok(path);
		} else if path.len() < 248 {
			match &*path {
				[drive, COLON, 0] | [drive, COLON, SEP | ALT, ..] if *drive != SEP && *drive != ALT => {
					return Ok(path);
				}
				[SEP | ALT, SEP | ALT, ..] => return Ok(path),
				_ => {}
			}
		}

		let mut stack = [0; 512];
		let mut heap = Vec::new();
		let mut buf = &mut stack[..];

		loop {
			let n = unsafe {
				GetFullPathNameW(path.as_ptr(), buf.len() as _, buf.as_mut_ptr(), ptr::null_mut())
			};
			if n == 0 {
				return Err(io::Error::last_os_error());
			} else if n as usize >= buf.len() {
				heap.resize(n as usize, 0);
				buf = &mut heap;
				continue;
			}

			let mut absolute = &buf[..n as usize];
			let prefix = match absolute {
				[_, COLON, SEP, ..] => VERBATIM,
				[SEP, SEP, DOT, SEP, ..] => {
					absolute = &absolute[4..];
					VERBATIM
				}
				[SEP, SEP, QUERY, SEP, ..] | [SEP, QUERY, QUERY, SEP, ..] => &[],
				[SEP, SEP, ..] => {
					absolute = &absolute[2..];
					UNC
				}
				_ => &[],
			};

			path.clear();
			path.reserve_exact(prefix.len() + absolute.len() + 1);
			path.extend_from_slice(prefix);
			path.extend_from_slice(absolute);
			path.push(0);

			return Ok(path);
		}
	}
}

impl ToWide for str {
	#[inline]
	fn to_wide(&self) -> Vec<u16> { OsStr::new(self).to_wide() }

	fn to_wide_path(&self) -> io::Result<Vec<u16>> { OsStr::new(self).to_wide_path() }
}

impl ToWide for Path {
	#[inline]
	fn to_wide(&self) -> Vec<u16> { self.as_os_str().to_wide() }

	fn to_wide_path(&self) -> io::Result<Vec<u16>> { self.as_os_str().to_wide_path() }
}
