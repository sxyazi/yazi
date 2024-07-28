use std::{borrow::Cow, iter::repeat};

pub fn escape_str(s: &str) -> Cow<str> {
	let bytes = s.as_bytes();
	if !bytes.is_empty() && !bytes.iter().any(|&c| matches!(c, b' ' | b'"' | b'\n' | b'\t')) {
		return Cow::Borrowed(s);
	}

	let mut escaped = String::with_capacity(bytes.len() + 2);
	escaped.push('"');

	let mut chars = bytes.iter().copied().peekable();
	loop {
		let mut slashes = 0;
		while chars.next_if_eq(&b'\\').is_some() {
			slashes += 1;
		}
		match chars.next() {
			Some(b'"') => {
				escaped.reserve(slashes * 2 + 2);
				escaped.extend(repeat('\\').take(slashes * 2 + 1));
				escaped.push('"');
			}
			Some(c) => {
				escaped.reserve(slashes + 1);
				escaped.extend(repeat('\\').take(slashes));
				escaped.push(c as _);
			}
			None => {
				escaped.reserve(slashes * 2);
				escaped.extend(repeat('\\').take(slashes * 2));
				break;
			}
		}
	}

	escaped.push('"');
	escaped.into()
}

#[cfg(windows)]
pub fn escape_os_str(s: &std::ffi::OsStr) -> Cow<std::ffi::OsStr> {
	use std::os::windows::ffi::{OsStrExt, OsStringExt};

	let wide = s.encode_wide();
	if !s.is_empty() && !wide.clone().into_iter().any(disallowed) {
		return Cow::Borrowed(s);
	}

	let mut escaped: Vec<u16> = Vec::with_capacity(s.len() + 2);
	escaped.push(b'"' as _);

	let mut chars = wide.into_iter().peekable();
	loop {
		let mut slashes = 0;
		while chars.next_if_eq(&(b'\\' as _)).is_some() {
			slashes += 1;
		}
		match chars.next() {
			Some(c) if c == b'"' as _ => {
				escaped.reserve(slashes * 2 + 2);
				escaped.extend(repeat(b'\\' as u16).take(slashes * 2 + 1));
				escaped.push(b'"' as _);
			}
			Some(c) => {
				escaped.reserve(slashes + 1);
				escaped.extend(repeat(b'\\' as u16).take(slashes));
				escaped.push(c);
			}
			None => {
				escaped.reserve(slashes * 2);
				escaped.extend(repeat(b'\\' as u16).take(slashes * 2));
				break;
			}
		}
	}

	escaped.push(b'"' as _);
	std::ffi::OsString::from_wide(&escaped).into()
}

#[cfg(windows)]
pub fn split(s: &str) -> std::io::Result<Vec<String>> {
	use std::os::windows::ffi::OsStrExt;

	let s: Vec<_> = std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect();
	split_slice(&s)
}

#[cfg(windows)]
fn split_slice(s: &[u16]) -> std::io::Result<Vec<String>> {
	use std::mem::MaybeUninit;

	use windows_sys::Win32::{Foundation::LocalFree, UI::Shell::CommandLineToArgvW};

	let mut argc = MaybeUninit::<i32>::uninit();
	let argv_p = unsafe { CommandLineToArgvW(s.as_ptr(), argc.as_mut_ptr()) };
	if argv_p.is_null() {
		return Err(std::io::Error::last_os_error());
	}

	let argv = unsafe { std::slice::from_raw_parts(argv_p, argc.assume_init() as usize) };
	let mut res = vec![];
	for &arg in argv {
		let len = unsafe { libc::wcslen(arg) };
		res.push(String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(arg, len) }));
	}

	unsafe { LocalFree(argv_p as _) };
	Ok(res)
}

#[cfg(windows)]
fn disallowed(b: u16) -> bool {
	match char::from_u32(b as u32) {
		Some(c) => matches!(c, ' ' | '"' | '\n' | '\t'),
		None => true,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_escape_str() {
		assert_eq!(escape_str(""), r#""""#);
		assert_eq!(escape_str(r#""""#), r#""\"\"""#);

		assert_eq!(escape_str("--aaa=bbb-ccc"), "--aaa=bbb-ccc");
		assert_eq!(escape_str(r#"\path\to\my documents\"#), r#""\path\to\my documents\\""#);

		assert_eq!(escape_str(r#"--features="default""#), r#""--features=\"default\"""#);
		assert_eq!(escape_str(r#""--features=\"default\"""#), r#""\"--features=\\\"default\\\"\"""#);
		assert_eq!(escape_str("linker=gcc -L/foo -Wl,bar"), r#""linker=gcc -L/foo -Wl,bar""#);
	}

	#[cfg(windows)]
	#[test]
	fn test_escape_os_str() {
		use std::{ffi::OsString, os::windows::ffi::OsStringExt};

		fn from_str(input: &str, expected: &str) {
			let observed = OsString::from(input);
			let expected = OsString::from(expected);
			assert_eq!(escape_os_str(observed.as_os_str()), expected.as_os_str());
		}

		fn from_bytes(input: &[u16], expected: &[u16]) {
			let observed = OsString::from_wide(input);
			let expected = OsString::from_wide(expected);
			assert_eq!(escape_os_str(observed.as_os_str()), expected.as_os_str());
		}

		from_str("", r#""""#);
		from_str(r#""""#, r#""\"\"""#);

		from_str("--aaa=bbb-ccc", "--aaa=bbb-ccc");
		from_str(r#"\path\to\my documents\"#, r#""\path\to\my documents\\""#);

		from_str(r#"--features="default""#, r#""--features=\"default\"""#);
		from_str(r#""--features=\"default\"""#, r#""\"--features=\\\"default\\\"\"""#);
		from_str("linker=gcc -L/foo -Wl,bar", r#""linker=gcc -L/foo -Wl,bar""#);

		from_bytes(&[0x1055, 0x006e, 0x0069, 0x0063, 0x006f, 0x0064, 0x0065], &[
			0x1055, 0x006e, 0x0069, 0x0063, 0x006f, 0x0064, 0x0065,
		]);
		from_bytes(&[0xd801, 0x006e, 0x0069, 0x0063, 0x006f, 0x0064, 0x0065], &[
			b'"' as u16,
			0xd801,
			0x006e,
			0x0069,
			0x0063,
			0x006f,
			0x0064,
			0x0065,
			b'"' as u16,
		]);
	}
}
