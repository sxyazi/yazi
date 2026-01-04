use std::borrow::Cow;

use anyhow::Result;

use crate::{Utf8BytePredictor, path::{AsPath, Components, Display, EndsWithError, JoinError, PathBufDyn, PathCow, PathDyn, PathDynError, PathKind, RsplitOnceError, StartsWithError, StripPrefixError, StripSuffixError}, strand::{AsStrand, Strand}};

pub trait PathLike: AsPath {
	fn as_os(&self) -> Result<&std::path::Path, PathDynError> { self.as_path().as_os() }

	fn as_unix(&self) -> Result<&typed_path::UnixPath, PathDynError> { self.as_path().as_unix() }

	fn components(&self) -> Components<'_> { self.as_path().components() }

	fn display(&self) -> Display<'_> { self.as_path().display() }

	fn encoded_bytes(&self) -> &[u8] { self.as_path().encoded_bytes() }

	fn ext(&self) -> Option<Strand<'_>> { self.as_path().ext() }

	fn has_root(&self) -> bool { self.as_path().has_root() }

	fn is_absolute(&self) -> bool { self.as_path().is_absolute() }

	fn is_empty(&self) -> bool { self.as_path().is_empty() }

	#[cfg(unix)]
	fn is_hidden(&self) -> bool { self.as_path().is_hidden() }

	fn kind(&self) -> PathKind { self.as_path().kind() }

	fn len(&self) -> usize { self.as_path().len() }

	fn name(&self) -> Option<Strand<'_>> { self.as_path().name() }

	fn parent(&self) -> Option<PathDyn<'_>> { self.as_path().parent() }

	fn rsplit_pred<T>(&self, pred: T) -> Option<(PathDyn<'_>, PathDyn<'_>)>
	where
		T: Utf8BytePredictor,
	{
		self.as_path().rsplit_pred(pred)
	}

	fn stem(&self) -> Option<Strand<'_>> { self.as_path().stem() }

	fn to_os_owned(&self) -> Result<std::path::PathBuf, PathDynError> { self.as_path().to_os_owned() }

	fn to_owned(&self) -> PathBufDyn { self.as_path().to_owned() }

	fn to_str(&self) -> Result<&str, std::str::Utf8Error> { self.as_path().to_str() }

	fn to_string_lossy(&self) -> Cow<'_, str> { self.as_path().to_string_lossy() }

	fn try_ends_with<T>(&self, child: T) -> Result<bool, EndsWithError>
	where
		T: AsStrand,
	{
		self.as_path().try_ends_with(child)
	}

	fn try_join<T>(&self, path: T) -> Result<PathBufDyn, JoinError>
	where
		T: AsStrand,
	{
		self.as_path().try_join(path)
	}

	fn try_rsplit_seq<T>(&self, pat: T) -> Result<(PathDyn<'_>, PathDyn<'_>), RsplitOnceError>
	where
		T: AsStrand,
	{
		self.as_path().try_rsplit_seq(pat)
	}

	fn try_starts_with<T>(&self, base: T) -> Result<bool, StartsWithError>
	where
		T: AsStrand,
	{
		self.as_path().try_starts_with(base)
	}

	fn try_strip_prefix<T>(&self, base: T) -> Result<PathDyn<'_>, StripPrefixError>
	where
		T: AsStrand,
	{
		self.as_path().try_strip_prefix(base)
	}

	fn try_strip_suffix<T>(&self, suffix: T) -> Result<PathDyn<'_>, StripSuffixError>
	where
		T: AsStrand,
	{
		self.as_path().try_strip_suffix(suffix)
	}
}

impl<P> From<&P> for PathBufDyn
where
	P: PathLike,
{
	fn from(value: &P) -> Self { value.to_owned() }
}

impl PathLike for PathBufDyn {}
impl PathLike for PathCow<'_> {}
