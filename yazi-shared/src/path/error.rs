use thiserror::Error;

use crate::strand::StrandError;

// --- EndsWithError
#[derive(Debug, Error)]
#[error("calling ends_with on paths with different encodings")]
pub enum EndsWithError {
	FromStrand(#[from] StrandError),
}

// --- JoinError
#[derive(Debug, Error)]
#[error("calling join on paths with different encodings")]
pub enum JoinError {
	FromStrand(#[from] StrandError),
	FromPathDyn(#[from] PathDynError),
}

impl From<StartsWithError> for JoinError {
	fn from(err: StartsWithError) -> Self {
		match err {
			StartsWithError::FromStrand(e) => Self::FromStrand(e),
		}
	}
}

impl From<JoinError> for std::io::Error {
	fn from(err: JoinError) -> Self { Self::other(err) }
}

// --- PathDynError
#[derive(Debug, Error)]
pub enum PathDynError {
	#[error("conversion to OS path failed")]
	AsOs,
	#[error("conversion to Unix path failed")]
	AsUnix,
	#[error("conversion to UTF-8 path failed")]
	AsUtf8,
}

impl From<StrandError> for PathDynError {
	fn from(err: StrandError) -> Self {
		match err {
			StrandError::AsOs => Self::AsOs,
			StrandError::AsUtf8 => Self::AsUtf8,
		}
	}
}

impl From<PathDynError> for std::io::Error {
	fn from(err: PathDynError) -> Self { Self::other(err) }
}

// --- SetNameError
#[derive(Debug, Error)]
#[error("calling set_name on paths with different encodings")]
pub enum SetNameError {
	FromStrand(#[from] StrandError),
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
pub enum StartsWithError {
	FromStrand(#[from] StrandError),
}

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

impl From<StrandError> for StripPrefixError {
	fn from(err: StrandError) -> Self {
		match err {
			StrandError::AsOs | StrandError::AsUtf8 => Self::WrongEncoding,
		}
	}
}

impl From<std::path::StripPrefixError> for StripPrefixError {
	fn from(_: std::path::StripPrefixError) -> Self { Self::NotPrefix }
}

impl From<typed_path::StripPrefixError> for StripPrefixError {
	fn from(_: typed_path::StripPrefixError) -> Self { Self::NotPrefix }
}

// --- StripSuffixError
#[derive(Debug, Error)]
pub enum StripSuffixError {
	#[error("calling strip_suffix on URLs with different schemes")]
	Exotic,
	#[error("the base is not a suffix of the path")]
	NotSuffix,
	#[error("calling strip_suffix on paths with different encodings")]
	WrongEncoding,
}

impl From<StrandError> for StripSuffixError {
	fn from(err: StrandError) -> Self {
		match err {
			StrandError::AsOs | StrandError::AsUtf8 => Self::WrongEncoding,
		}
	}
}
