use core::str;
use std::borrow::Cow;

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

pub fn replace_cow<'a, T>(s: T, from: &str, to: &str) -> Cow<'a, str>
where
	T: Into<Cow<'a, str>>,
{
	let cow = s.into();
	match replace_cow_impl(&cow, cow.match_indices(from), to) {
		Cow::Borrowed(_) => cow,
		Cow::Owned(new) => Cow::Owned(new),
	}
}

pub fn replacen_cow<'a, T>(s: T, from: &str, to: &str, n: usize) -> Cow<'a, str>
where
	T: Into<Cow<'a, str>>,
{
	let cow = s.into();
	match replace_cow_impl(&cow, cow.match_indices(from).take(n), to) {
		Cow::Borrowed(_) => cow,
		Cow::Owned(now) => Cow::Owned(now),
	}
}

fn replace_cow_impl<'a, T>(src: &'a str, mut indices: T, to: &str) -> Cow<'a, str>
where
	T: Iterator<Item = (usize, &'a str)>,
{
	let Some((first_idx, first_sub)) = indices.next() else {
		return Cow::Borrowed(src);
	};

	let mut result = String::with_capacity(src.len());
	result += unsafe { src.get_unchecked(..first_idx) };
	result += to;

	let mut last = first_idx + first_sub.len();
	for (idx, sub) in indices {
		result += unsafe { src.get_unchecked(last..idx) };
		result += to;
		last = idx + sub.len();
	}

	Cow::Owned(result + unsafe { src.get_unchecked(last..) })
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

pub fn replace_to_printable(b: &[u8], lf: bool, tab_size: u8, replacement: bool) -> Cow<'_, [u8]> {
	// Fast path to skip over printable chars at the beginning of the string
	let printable_len = b.iter().take_while(|&&c| !c.is_ascii_control()).count();
	if printable_len >= b.len() {
		return Cow::Borrowed(b);
	}

	let (printable, rest) = b.split_at(printable_len);

	let mut out = Vec::new();
	out.reserve_exact(b.len() | 15);
	out.extend_from_slice(printable);

	for &c in rest {
		push_printable_char(&mut out, c, lf, tab_size, replacement);
	}
	Cow::Owned(out)
}

#[inline]
pub fn push_printable_char(buf: &mut Vec<u8>, c: u8, lf: bool, tab_size: u8, replacement: bool) {
	match c {
		b'\n' if lf => buf.push(b'\n'),
		b'\t' => {
			buf.extend((0..tab_size).map(|_| b' '));
		}
		b'\0'..=b'\x1F' => {
			if replacement {
				buf.extend_from_slice(&[0xef, 0xbf, 0xbd]);
			} else {
				buf.push(b'^');
				buf.push(c + b'@');
			}
		}
		0x7f => {
			if replacement {
				buf.extend_from_slice(&[0xef, 0xbf, 0xbd]);
			} else {
				buf.push(b'^');
				buf.push(b'?');
			}
		}
		_ => buf.push(c),
	}
}
