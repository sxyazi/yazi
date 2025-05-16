use core::str;
use std::{borrow::Cow, ffi::OsStr};

pub const MIME_DIR: &str = "inode/directory";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CharKind {
	Space,
	Punct,
	Other,
}

impl CharKind {
	pub fn new(c: char) -> Self {
		if c.is_whitespace() {
			Self::Space
		} else if c.is_ascii_punctuation() {
			Self::Punct
		} else {
			Self::Other
		}
	}

	pub fn vary(self, other: Self, far: bool) -> bool {
		if far { (self == Self::Space) != (other == Self::Space) } else { self != other }
	}
}

pub fn strip_trailing_newline(mut s: String) -> String {
	while s.ends_with('\n') || s.ends_with('\r') {
		s.pop();
	}
	s
}

pub fn replace_cow<'a>(s: &'a str, from: &str, to: &str) -> Cow<'a, str> {
	replace_cow_impl(s, s.match_indices(from), to)
}

pub fn replacen_cow<'a>(s: &'a str, from: &str, to: &str, n: usize) -> Cow<'a, str> {
	replace_cow_impl(s, s.match_indices(from).take(n), to)
}

fn replace_cow_impl<'s>(
	src: &'s str,
	mut indices: impl Iterator<Item = (usize, &'s str)>,
	to: &str,
) -> Cow<'s, str> {
	let Some((first_idx, first_sub)) = indices.next() else {
		return Cow::Borrowed(src);
	};

	let mut result = Cow::Owned(String::with_capacity(src.len()));
	result += unsafe { src.get_unchecked(..first_idx) };
	result.to_mut().push_str(to);

	let mut last = first_idx + first_sub.len();
	for (idx, sub) in indices {
		result += unsafe { src.get_unchecked(last..idx) };
		result.to_mut().push_str(to);
		last = idx + sub.len();
	}

	result + unsafe { src.get_unchecked(last..) }
}

pub fn replace_vec_cow<'a>(v: &'a [u8], from: &[u8], to: &[u8]) -> Cow<'a, [u8]> {
	let mut it = memchr::memmem::find_iter(v, from);
	let Some(mut last) = it.next() else { return Cow::Borrowed(v) };

	let mut out = Vec::with_capacity(v.len());
	out.extend_from_slice(&v[..last]);
	out.extend_from_slice(to);
	last += from.len();

	for idx in it {
		out.extend_from_slice(&v[last..idx]);
		out.extend_from_slice(to);
		last = idx + from.len();
	}

	out.extend_from_slice(&v[last..]);
	Cow::Owned(out)
}

pub fn replace_to_printable(s: &[String], tab_size: u8) -> String {
	let mut buf = Vec::new();
	buf.try_reserve_exact(s.iter().map(|s| s.len()).sum::<usize>() | 15).unwrap_or_else(|_| panic!());

	for &b in s.iter().flat_map(|s| s.as_bytes()) {
		match b {
			b'\n' => buf.push(b'\n'),
			b'\t' => {
				buf.extend((0..tab_size).map(|_| b' '));
			}
			b'\0'..=b'\x1F' => {
				buf.push(b'^');
				buf.push(b + b'@');
			}
			0x7f => {
				buf.push(b'^');
				buf.push(b'?');
			}
			_ => buf.push(b),
		}
	}
	unsafe { String::from_utf8_unchecked(buf) }
}

pub fn osstr_contains(s: impl AsRef<OsStr>, needle: impl AsRef<OsStr>) -> bool {
	memchr::memmem::find(s.as_ref().as_encoded_bytes(), needle.as_ref().as_encoded_bytes()).is_some()
}

pub fn osstr_starts_with(
	s: impl AsRef<OsStr>,
	prefix: impl AsRef<OsStr>,
	insensitive: bool,
) -> bool {
	let (s, prefix) = (s.as_ref().as_encoded_bytes(), prefix.as_ref().as_encoded_bytes());
	if s.len() < prefix.len() {
		return false;
	}
	if insensitive {
		s[..prefix.len()].eq_ignore_ascii_case(prefix)
	} else {
		s[..prefix.len()] == *prefix
	}
}
