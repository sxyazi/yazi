use std::fmt::{self, Display};

use bitflags::bitflags;

/// Pop keyboard enhancement flags
pub struct PopKeyboardFlags;

impl Display for PopKeyboardFlags {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str("\x1b[<1u") }
}

// Push keyboard enhancement flags
bitflags! {
	#[derive(Clone, Copy, Debug, Eq, PartialEq)]
	pub struct PushKeyboardFlags: u8 {
		const DISAMBIGUATE_ESCAPE_CODES       = 1;
		const REPORT_EVENT_TYPES              = 2;
		const REPORT_ALTERNATE_KEYS           = 4;
		const REPORT_ALL_KEYS_AS_ESCAPE_CODES = 8;
		const REPORT_ASSOCIATED_TEXT          = 16;
	}
}

impl Display for PushKeyboardFlags {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\x1b[>{}u", self.bits()) }
}
