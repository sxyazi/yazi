use std::{error::Error, fmt::{self, Display}};

#[derive(Debug)]
pub enum PagedError {
	Exceed(usize),
	Unexpected(String),
}

impl Display for PagedError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Exceed(lines) => write!(f, "Exceed: {lines}"),
			Self::Unexpected(msg) => write!(f, "Unexpected error: {msg}"),
		}
	}
}

impl Error for PagedError {}

impl From<String> for PagedError {
	fn from(error: String) -> Self { Self::Unexpected(error) }
}
impl From<&str> for PagedError {
	fn from(error: &str) -> Self { Self::from(error.to_owned()) }
}
impl From<anyhow::Error> for PagedError {
	fn from(error: anyhow::Error) -> Self { Self::from(error.to_string()) }
}
impl From<std::io::Error> for PagedError {
	fn from(error: std::io::Error) -> Self { Self::from(error.to_string()) }
}
impl From<tokio::task::JoinError> for PagedError {
	fn from(error: tokio::task::JoinError) -> Self { Self::from(error.to_string()) }
}
