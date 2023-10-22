use std::{error::Error, fmt::{self, Display}};

#[derive(Debug)]
pub enum InputError {
	Typed(String),
	Canceled(String),
}

impl Display for InputError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Typed(text) => write!(f, "Typed error: {text}"),
			Self::Canceled(text) => write!(f, "Canceled error: {text}"),
		}
	}
}

impl Error for InputError {}
