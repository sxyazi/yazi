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
