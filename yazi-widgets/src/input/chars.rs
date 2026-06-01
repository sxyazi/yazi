use crate::input::Gait;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum CharKind {
	Space,
	Punct,
	Other,
}

impl CharKind {
	pub(crate) fn new(c: char) -> Self {
		if c.is_whitespace() {
			Self::Space
		} else if Self::is_punct(c) {
			Self::Punct
		} else {
			Self::Other
		}
	}

	pub(crate) fn vary(self, other: Self, gait: Gait) -> bool {
		match gait {
			Gait::Fine => self != other,
			Gait::Lean => self == Self::Other && other != Self::Other,
			Gait::Wide => (self == Self::Space) != (other == Self::Space),
		}
	}

	const fn is_punct(c: char) -> bool {
		match c {
			'_' => false,
			c if c.is_ascii_punctuation() => true,
			'！' | '，' | '；' | '？' => true,
			_ => false,
		}
	}
}
