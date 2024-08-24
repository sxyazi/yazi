use std::{error::Error, fmt::{self, Display}};

#[derive(Debug)]
pub enum PeekError {
	Exceed(usize),
	Unexpected(String),
}

impl Display for PeekError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Exceed(lines) => write!(f, "Exceed maximum lines {lines}"),
			Self::Unexpected(msg) => write!(f, "{msg}"),
		}
	}
}

impl Error for PeekError {}

impl From<String> for PeekError {
	fn from(error: String) -> Self { Self::Unexpected(error) }
}
impl From<&str> for PeekError {
	fn from(error: &str) -> Self { Self::from(error.to_owned()) }
}
impl From<anyhow::Error> for PeekError {
	fn from(error: anyhow::Error) -> Self { Self::from(error.to_string()) }
}
impl From<std::io::Error> for PeekError {
	fn from(error: std::io::Error) -> Self { Self::from(error.to_string()) }
}
impl From<tokio::task::JoinError> for PeekError {
	fn from(error: tokio::task::JoinError) -> Self { Self::from(error.to_string()) }
}
