use thiserror::Error;

use crate::strand::StrandError;

// --- EndsWithError
#[derive(Debug, Error)]
#[error("calling ends_with on paths with different encodings")]
pub struct EndsWithError;

// --- JoinError
#[derive(Debug, Error)]
#[error("calling join on paths with different encodings")]
pub enum JoinError {
	FromWtf8,
	FromPathBufDyn(#[from] PathBufDynError),
}

impl From<JoinError> for std::io::Error {
	fn from(err: JoinError) -> Self { Self::other(err) }
}

// --- PathDynError
#[derive(Debug, Error)]
pub enum PathDynError {
	#[error("conversion to OsStr failed")]
	AsOs,
}

impl From<PathDynError> for std::io::Error {
	fn from(err: PathDynError) -> Self { Self::other(err) }
}

// --- PathBufDynError
#[derive(Debug, Error)]
pub enum PathBufDynError {
	#[error("conversion to OsString failed")]
	IntoOs,
}

// --- SetNameError
#[derive(Debug, Error)]
#[error("calling set_name on paths with different encodings")]
pub enum SetNameError {
	FromWtf8,
	FromStrandDyn(#[from] StrandError),
}

impl From<SetNameError> for std::io::Error {
	fn from(err: SetNameError) -> Self { Self::other(err) }
}

// --- RsplitOnce
#[derive(Error, Debug)]
#[error("calling rsplit_once on paths with different encodings")]
pub enum RsplitOnceError {
	#[error("conversion to OsStr failed")]
	AsOs,
	#[error("conversion to UTF-8 str failed")]
	AsUtf8,
	#[error("the pattern was not found")]
	NotFound,
}

impl From<StrandError> for RsplitOnceError {
	fn from(err: StrandError) -> Self {
		match err {
			StrandError::AsOs => Self::AsOs,
			StrandError::AsUtf8 => Self::AsUtf8,
		}
	}
}

// --- StartsWithError
#[derive(Error, Debug)]
#[error("calling starts_with on paths with different encodings")]
pub struct StartsWithError;

// --- StripPrefixError
#[derive(Debug, Error)]
pub enum StripPrefixError {
	#[error("calling strip_prefix on URLs with different schemes")]
	Exotic,
	#[error("the base is not a prefix of the path")]
	NotPrefix,
	#[error("calling strip_prefix on paths with different encodings")]
	WrongEncoding,
}

impl From<std::path::StripPrefixError> for StripPrefixError {
	fn from(_: std::path::StripPrefixError) -> Self { Self::NotPrefix }
}
