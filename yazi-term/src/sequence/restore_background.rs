use std::fmt::{self, Display};

/// Restore background color to default
pub struct RestoreBackground;

impl Display for RestoreBackground {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "\x1b]111\x1b\\") }
}
