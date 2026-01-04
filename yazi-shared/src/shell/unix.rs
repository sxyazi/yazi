use std::{borrow::Cow, mem};

use crate::shell::SplitError;

pub fn escape_os_bytes(b: &[u8]) -> Cow<'_, [u8]> {
	if !b.is_empty() && b.iter().copied().all(allowed) {
		return Cow::Borrowed(b);
	}

	let mut escaped = Vec::with_capacity(b.len() + 2);
	escaped.push(b'\'');

	for &c in b {
		match c {
			b'\'' | b'!' => {
				escaped.reserve(4);
				escaped.push(b'\'');
				escaped.push(b'\\');
				escaped.push(c);
				escaped.push(b'\'');
			}
			_ => escaped.push(c),
		}
	}

	escaped.push(b'\'');
	escaped.into()
}

#[cfg(unix)]
pub fn escape_os_str(s: &std::ffi::OsStr) -> Cow<'_, std::ffi::OsStr> {
	use std::os::unix::ffi::{OsStrExt, OsStringExt};

	match escape_os_bytes(s.as_bytes()) {
		Cow::Borrowed(_) => Cow::Borrowed(s),
		Cow::Owned(v) => std::ffi::OsString::from_vec(v).into(),
	}
}

fn allowed(b: u8) -> bool {
	matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'=' | b'/' | b',' | b'.' | b'+')
}

pub fn split(s: &str, eoo: bool) -> Result<(Vec<String>, Option<String>), SplitError> {
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
				None => return Err(SplitError::MissingSingleQuote),
				Some('\'') => Unquoted,
				Some(c) => {
					word.push(c);
					SingleQuoted
				}
			},
			DoubleQuoted => match c {
				None => return Err(SplitError::MissingDoubleQuote),
				Some('\"') => Unquoted,
				Some('\\') => DoubleQuotedBackslash,
				Some(c) => {
					word.push(c);
					DoubleQuoted
				}
			},
			DoubleQuotedBackslash => match c {
				None => return Err(SplitError::MissingQuoteAfterSlash),
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
	fn test_escape_os_bytes() {
		let cases: &[(&[u8], &[u8])] = &[
			(b"", br#"''"#),
			(b" ", br#"' '"#),
			(b"*", br#"'*'"#),
			(b"--aaa=bbb-ccc", b"--aaa=bbb-ccc"),
			(br#"--features="default""#, br#"'--features="default"'"#),
			(b"linker=gcc -L/foo -Wl,bar", br#"'linker=gcc -L/foo -Wl,bar'"#),
			(
				b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_=/,.+",
				b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_=/,.+",
			),
			(br#"'!\$`\\\n "#, br#"''\'''\!'\$`\\\n '"#),
			(&[0x66, 0x6f, 0x80, 0x6f], &[b'\'', 0x66, 0x6f, 0x80, 0x6f, b'\'']),
		];

		for &(input, expected) in cases {
			let escaped = escape_os_bytes(input);
			assert_eq!(escaped, expected, "Failed to escape: {:?}", String::from_utf8_lossy(input));
		}
	}
}
