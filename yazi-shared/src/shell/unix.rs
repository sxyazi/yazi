use std::{borrow::Cow, mem};

pub fn escape_str(s: &str) -> Cow<str> {
	match escape_slice(s.as_bytes()) {
		Cow::Borrowed(_) => Cow::Borrowed(s),
		Cow::Owned(v) => String::from_utf8(v).expect("Invalid bytes returned by escape_slice()").into(),
	}
}

#[cfg(unix)]
pub fn escape_os_str(s: &std::ffi::OsStr) -> Cow<std::ffi::OsStr> {
	use std::os::unix::ffi::{OsStrExt, OsStringExt};

	match escape_slice(s.as_bytes()) {
		Cow::Borrowed(_) => Cow::Borrowed(s),
		Cow::Owned(v) => std::ffi::OsString::from_vec(v).into(),
	}
}

fn escape_slice(s: &[u8]) -> Cow<[u8]> {
	if !s.is_empty() && s.iter().copied().all(allowed) {
		return Cow::Borrowed(s);
	}

	let mut escaped = Vec::with_capacity(s.len() + 2);
	escaped.push(b'\'');

	for &b in s {
		match b {
			b'\'' | b'!' => {
				escaped.reserve(4);
				escaped.push(b'\'');
				escaped.push(b'\\');
				escaped.push(b);
				escaped.push(b'\'');
			}
			_ => escaped.push(b),
		}
	}

	escaped.push(b'\'');
	escaped.into()
}

fn allowed(b: u8) -> bool {
	matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'=' | b'/' | b',' | b'.' | b'+')
}

pub fn split(s: &str, eoo: bool) -> Result<(Vec<String>, Option<String>), ()> {
	enum State {
		/// Within a delimiter.
		Delimiter,
		/// After backslash, but before starting word.
		Backslash,
		/// Within an unquoted word.
		Unquoted,
		/// After backslash in an unquoted word.
		UnquotedBackslash,
		/// Within a single quoted word.
		SingleQuoted,
		/// Within a double quoted word.
		DoubleQuoted,
		/// After backslash inside a double quoted word.
		DoubleQuotedBackslash,
		/// Inside a comment.
		Comment,
	}
	use State::*;

	let mut words = Vec::new();
	let mut word = String::new();
	let mut chars = s.chars();
	let mut state = Delimiter;

	macro_rules! flush {
		() => {
			if word == "--" && eoo {
				return Ok((words, Some(chars.collect())));
			}
			words.push(mem::take(&mut word));
		};
	}

	loop {
		let c = chars.next();
		state = match state {
			Delimiter => match c {
				None => break,
				Some('\'') => SingleQuoted,
				Some('\"') => DoubleQuoted,
				Some('\\') => Backslash,
				Some('\t') | Some(' ') | Some('\n') => Delimiter,
				Some('#') => Comment,
				Some(c) => {
					word.push(c);
					Unquoted
				}
			},
			Backslash => match c {
				None => {
					word.push('\\');
					flush!();
					break;
				}
				Some('\n') => Delimiter,
				Some(c) => {
					word.push(c);
					Unquoted
				}
			},
			Unquoted => match c {
				None => {
					flush!();
					break;
				}
				Some('\'') => SingleQuoted,
				Some('\"') => DoubleQuoted,
				Some('\\') => UnquotedBackslash,
				Some('\t') | Some(' ') | Some('\n') => {
					flush!();
					Delimiter
				}
				Some(c) => {
					word.push(c);
					Unquoted
				}
			},
			UnquotedBackslash => match c {
				None => {
					word.push('\\');
					flush!();
					break;
				}
				Some('\n') => Unquoted,
				Some(c) => {
					word.push(c);
					Unquoted
				}
			},
			SingleQuoted => match c {
				None => return Err(()),
				Some('\'') => Unquoted,
				Some(c) => {
					word.push(c);
					SingleQuoted
				}
			},
			DoubleQuoted => match c {
				None => return Err(()),
				Some('\"') => Unquoted,
				Some('\\') => DoubleQuotedBackslash,
				Some(c) => {
					word.push(c);
					DoubleQuoted
				}
			},
			DoubleQuotedBackslash => match c {
				None => return Err(()),
				Some('\n') => DoubleQuoted,
				Some(c @ '$') | Some(c @ '`') | Some(c @ '"') | Some(c @ '\\') => {
					word.push(c);
					DoubleQuoted
				}
				Some(c) => {
					word.push('\\');
					word.push(c);
					DoubleQuoted
				}
			},
			Comment => match c {
				None => break,
				Some('\n') => Delimiter,
				Some(_) => Comment,
			},
		}
	}

	Ok((words, None))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_escape_str() {
		assert_eq!(escape_str(""), r#"''"#);
		assert_eq!(escape_str(" "), r#"' '"#);
		assert_eq!(escape_str("*"), r#"'*'"#);

		assert_eq!(escape_str("--aaa=bbb-ccc"), "--aaa=bbb-ccc");
		assert_eq!(escape_str(r#"--features="default""#), r#"'--features="default"'"#);
		assert_eq!(escape_str("linker=gcc -L/foo -Wl,bar"), r#"'linker=gcc -L/foo -Wl,bar'"#);

		assert_eq!(
			escape_str("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_=/,.+"),
			"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_=/,.+",
		);
		assert_eq!(escape_str(r#"'!\$`\\\n "#), r#"''\'''\!'\$`\\\n '"#);
	}

	#[cfg(unix)]
	#[test]
	fn test_escape_os_str() {
		use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

		fn from_str(input: &str, expected: &str) { from_bytes(input.as_bytes(), expected.as_bytes()) }

		fn from_bytes(input: &[u8], expected: &[u8]) {
			assert_eq!(escape_os_str(OsStr::from_bytes(input)), OsStr::from_bytes(expected));
		}

		from_str("", r#"''"#);
		from_str(" ", r#"' '"#);
		from_str("*", r#"'*'"#);

		from_str("--aaa=bbb-ccc", "--aaa=bbb-ccc");
		from_str(r#"--features="default""#, r#"'--features="default"'"#);
		from_str("linker=gcc -L/foo -Wl,bar", r#"'linker=gcc -L/foo -Wl,bar'"#);

		from_str(
			"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_=/,.+",
			"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_=/,.+",
		);
		from_str(r#"'!\$`\\\n "#, r#"''\'''\!'\$`\\\n '"#);

		from_bytes(&[0x66, 0x6f, 0x80, 0x6f], &[b'\'', 0x66, 0x6f, 0x80, 0x6f, b'\'']);
	}
}
