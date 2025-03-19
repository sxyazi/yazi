use std::{error::Error, fmt::{self, Display}};

use crate::Id;

#[derive(Debug)]
pub enum InputError {
	Typed(String),
	Completed(String, Id),
	Canceled(String),
}

impl Display for InputError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Typed(text) => write!(f, "Typed error: {text}"),
			Self::Completed(text, _) => write!(f, "Completed error: {text}"),
			Self::Canceled(text) => write!(f, "Canceled error: {text}"),
		}
	}
}

impl Error for InputError {}
