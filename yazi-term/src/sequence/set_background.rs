use std::fmt::{self, Display};

/// Set background color to a RGB value or named color
pub struct SetBackground<'a>(pub &'a str);

impl Display for SetBackground<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.0.is_empty() { Ok(()) } else { write!(f, "\x1b]11;{}\x1b\\", self.0) }
	}
}
