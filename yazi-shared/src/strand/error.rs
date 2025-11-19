use thiserror::Error;

use crate::path::PathDynError;

// --- StrandDynError
#[derive(Debug, Error)]
pub enum StrandError {
	#[error("conversion to OsStr failed")]
	AsOs,
	#[error("conversion to UTF-8 str failed")]
	AsUtf8,
}

impl From<PathDynError> for StrandError {
	fn from(err: PathDynError) -> Self {
		match err {
			PathDynError::AsOs => Self::AsOs,
		}
	}
}

impl From<StrandError> for std::io::Error {
	fn from(err: StrandError) -> Self { Self::other(err) }
}
