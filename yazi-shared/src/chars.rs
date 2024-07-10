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
	if s.ends_with('\n') {
		s.pop();
	}
	if s.ends_with('\r') {
		s.pop();
	}
	s
}
