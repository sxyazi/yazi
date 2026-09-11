use bitflags::bitflags;
use serde::Serialize;

bitflags! {
	#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Serialize)]
	pub struct Modifiers: u8 {
		const SHIFT   = 1;
		const ALT     = 2;
		const CONTROL = 4;
		const SUPER   = 8;
		const HYPER   = 16;
		const META    = 32;
	}
}

impl Modifiers {
	pub fn is_literal(self) -> bool { self.difference(Self::SHIFT).is_empty() }

	pub(crate) fn for_char(c: char) -> Self {
		if c.is_uppercase() { Self::SHIFT } else { Self::empty() }
	}
}
