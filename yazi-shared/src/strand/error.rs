use thiserror::Error;

// --- StrandError
#[derive(Debug, Error)]
pub enum StrandError {
	#[error("conversion to OS string failed")]
	AsOs,
	#[error("conversion to UTF-8 string failed")]
	AsUtf8,
}

impl From<StrandError> for std::io::Error {
	fn from(err: StrandError) -> Self { Self::other(err) }
}
