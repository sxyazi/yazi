use std::fmt::{self, Display};

pub struct If<T: Display>(pub bool, pub T);

impl<T: Display> Display for If<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.0 { write!(f, "{}", self.1) } else { Ok(()) }
	}
}
