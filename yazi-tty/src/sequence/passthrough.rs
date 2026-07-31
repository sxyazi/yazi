use std::fmt::{self, Display};

pub struct TmuxPassthrough<T>(pub T, pub bool);

impl<T: Display> Display for TmuxPassthrough<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if !self.1 {
			return self.0.fmt(f);
		}

		write!(
			f,
			"\x1bPtmux;\x1b\x1b{}\x1b\\",
			self.0.to_string().trim_start_matches('\x1b').replace('\x1b', "\x1b\x1b")
		)
	}
}
