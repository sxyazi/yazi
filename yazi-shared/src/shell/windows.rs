use std::borrow::Cow;

pub fn escape_os_bytes(b: &[u8]) -> Cow<'_, [u8]> {
	let quote = needs_quotes(b);
	let mut buf = Vec::with_capacity(b.len() + quote as usize * 2);

	if quote {
		buf.push(b'"');
	}

	// Loop through the string, escaping `\` only if followed by `"`.
	// And escaping `"` by doubling them.
	let mut backslashes: usize = 0;
	for &c in b {
		if c == b'\\' {
			backslashes += 1;
		} else {
			if c == b'"' {
				buf.extend((0..backslashes).map(|_| b'\\'));
				buf.push(b'"');
			} else if c == b'%' {
				buf.extend_from_slice(b"%%cd:~,");
			} else if c == b'\r' || c == b'\n' {
				buf.extend_from_slice(b"%=%");
				backslashes = 0;
				continue;
			}
			backslashes = 0;
		}
		buf.push(c);
	}

	if quote {
		buf.extend((0..backslashes).map(|_| b'\\'));
		buf.push(b'"');
	}

	buf.into()
}

#[cfg(windows)]
pub fn escape_os_str(s: &std::ffi::OsStr) -> Cow<'_, std::ffi::OsStr> {
	use crate::wtf8::FromWtf8Vec;

	match escape_os_bytes(s.as_encoded_bytes()) {
		Cow::Borrowed(_) => Cow::Borrowed(s),
		Cow::Owned(v) => std::ffi::OsString::from_wtf8_vec(v).expect("valid WTF-8").into(),
	}
}

fn needs_quotes(arg: &[u8]) -> bool {
	static UNQUOTED: &[u8] = br"#$*+-./:?@\_";

	if arg.is_empty() || arg.last() == Some(&b'\\') {
		return true;
	}

	for c in arg {
		if c.is_ascii_control() {
			return true;
		} else if c.is_ascii() && !(c.is_ascii_alphanumeric() || UNQUOTED.contains(c)) {
			return true;
		}
	}
	false
}

#[cfg(windows)]
pub fn split(s: &str) -> std::io::Result<Vec<String>> {
	use std::os::windows::ffi::OsStrExt;

	let s: Vec<_> = std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect();
	split_wide(&s)
}

#[cfg(windows)]
fn split_wide(s: &[u16]) -> std::io::Result<Vec<String>> {
	use std::mem::MaybeUninit;

	use windows_sys::{Win32::{Foundation::LocalFree, UI::Shell::CommandLineToArgvW}, core::PCWSTR};

	unsafe extern "C" {
		fn wcslen(s: PCWSTR) -> usize;
	}

	let mut argc = MaybeUninit::<i32>::uninit();
	let argv_p = unsafe { CommandLineToArgvW(s.as_ptr(), argc.as_mut_ptr()) };
	if argv_p.is_null() {
		return Err(std::io::Error::last_os_error());
	}

	let argv = unsafe { std::slice::from_raw_parts(argv_p, argc.assume_init() as usize) };
	let mut res = vec![];
	for &arg in argv {
		let len = unsafe { wcslen(arg) };
		res.push(String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(arg, len) }));
	}

	unsafe { LocalFree(argv_p as _) };
	Ok(res)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_escape_os_bytes() {
		let cases: &[(&[u8], &[u8])] = &[
			// Empty string
			(b"", br#""""#),
			(br#""""#, br#""""""""#),
			// No escaping needed
			(b"--aaa=bbb-ccc", br#""--aaa=bbb-ccc""#),
			// Paths with spaces
			(br#"\path\to\my documents\"#, br#""\path\to\my documents\\""#),
			// Strings with quotes
			(br#"--features="default""#, br#""--features=""default""""#),
			// Nested quotes
			(br#""--features=\"default\"""#, br#""""--features=\\""default\\""""""#),
			// Complex command
			(b"linker=gcc -L/foo -Wl,bar", br#""linker=gcc -L/foo -Wl,bar""#),
			// Variable expansion
			(b"%APPDATA%.txt", br#""%%cd:~,%APPDATA%%cd:~,%.txt""#),
			// Unicode characters
			("이것은 테스트".as_bytes(), r#""이것은 테스트""#.as_bytes()),
		];

		for &(input, expected) in cases {
			let escaped = escape_os_bytes(input);
			assert_eq!(escaped, expected, "Failed to escape: {:?}", String::from_utf8_lossy(input));
		}
	}

	#[cfg(windows)]
	#[test]
	fn test_escape_os_str() {
		use std::{ffi::OsString, os::windows::ffi::OsStringExt};

		#[rustfmt::skip]
		let cases: &[(OsString, OsString)] = &[
			// Surrogate pairs and special characters
			(
				OsString::from_wide(&[0x1055, 0x006e, 0x0069, 0x0063, 0x006f, 0x0064, 0x0065]),
				OsString::from_wide(&[0x1055, 0x006e, 0x0069, 0x0063, 0x006f, 0x0064, 0x0065]),
			),
			// Surrogate pair with quotes
			(
				OsString::from_wide(&[0xd801, 0x006e, 0x0069, 0x0063, 0x006f, 0x0064, 0x0065]),
				OsString::from_wide(&[0xd801, 0x006e, 0x0069, 0x0063, 0x006f, 0x0064, 0x0065]),
			),
		];

		for (input, expected) in cases {
			let escaped = escape_os_str(&input);
			assert_eq!(&*escaped, expected, "Failed to escape: {:?}", input.to_string_lossy());
		}
	}
}
