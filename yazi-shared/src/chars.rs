use core::str;

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
}

pub fn strip_trailing_newline(mut s: String) -> String {
	while s.ends_with('\n') || s.ends_with('\r') {
		s.pop();
	}
	s
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
