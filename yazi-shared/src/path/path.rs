use std::{borrow::Cow, ffi::OsStr};

use anyhow::Result;

use crate::{BytesExt, Utf8BytePredictor, path::{AsPathView, EndsWithError, JoinError, PathBufDyn, PathBufLike, PathDyn, PathKind, RsplitOnceError, StartsWithError, StripPrefixError}, strand::{AsStrandView, StrandLike}};

pub trait PathLike<'p>
where
	Self: Copy + AsPathView<'p, Self::View<'p>> + AsStrandView<'p, Self::Strand<'p>>,
{
	type Strand<'a>: StrandLike<'a>;
	type Owned: PathBufLike + Into<Self::Owned>;
	type View<'a>;
	type Components<'a>: Clone
		+ DoubleEndedIterator
		+ AsPathView<'a, Self::View<'a>>
		+ AsStrandView<'a, Self::Strand<'a>>;
	type Display<'a>: std::fmt::Display;

	fn as_dyn(self) -> PathDyn<'p>;

	fn components(self) -> Self::Components<'p>;

	fn default() -> Self;

	fn display(self) -> Self::Display<'p>;

	fn encoded_bytes(self) -> &'p [u8];

	fn ext(self) -> Option<Self::Strand<'p>>;

	fn has_root(self) -> bool;

	fn is_absolute(self) -> bool;

	fn is_empty(self) -> bool { self.encoded_bytes().is_empty() }

	#[cfg(unix)]
	fn is_hidden(self) -> bool {
		self.name().is_some_and(|n| n.encoded_bytes().first() == Some(&b'.'))
	}

	fn kind(self) -> PathKind;

	fn len(self) -> usize { self.encoded_bytes().len() }

	fn name(self) -> Option<Self::Strand<'p>>;

	fn owned(self) -> Self::Owned;

	fn parent(self) -> Option<Self>;

	fn rsplit_pred<T>(self, pred: T) -> Option<(Self, Self)>
	where
		T: Utf8BytePredictor;

	fn stem(self) -> Option<Self::Strand<'p>>;

	fn to_str(self) -> Result<&'p str, std::str::Utf8Error> { str::from_utf8(self.encoded_bytes()) }

	fn to_string_lossy(self) -> Cow<'p, str> { String::from_utf8_lossy(self.encoded_bytes()) }

	fn to_buf_dyn(self) -> PathBufDyn { self.as_dyn().owned() }

	fn try_ends_with<'a, T>(self, child: T) -> Result<bool, EndsWithError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>;

	fn try_join<'a, T>(self, path: T) -> Result<Self::Owned, JoinError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>;

	fn try_rsplit_seq<'a, T>(self, pat: T) -> Result<(Self, Self), RsplitOnceError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>;

	fn try_starts_with<'a, T>(self, base: T) -> Result<bool, StartsWithError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>;

	fn try_strip_prefix<'a, T>(self, base: T) -> Result<Self, StripPrefixError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>;
}

impl<'p> PathLike<'p> for &'p std::path::Path {
	type Components<'a> = std::path::Components<'a>;
	type Display<'a> = std::path::Display<'a>;
	type Owned = std::path::PathBuf;
	type Strand<'a> = &'a std::ffi::OsStr;
	type View<'a> = &'a std::path::Path;

	fn as_dyn(self) -> PathDyn<'p> { PathDyn::Os(self) }

	fn components(self) -> Self::Components<'p> { self.components() }

	fn default() -> Self { std::path::Path::new("") }

	fn display(self) -> Self::Display<'p> { self.display() }

	fn encoded_bytes(self) -> &'p [u8] { self.as_os_str().as_encoded_bytes() }

	fn ext(self) -> Option<Self::Strand<'p>> { self.extension() }

	fn has_root(self) -> bool { self.has_root() }

	fn is_absolute(self) -> bool { self.is_absolute() }

	fn kind(self) -> PathKind { PathKind::Os }

	fn name(self) -> Option<Self::Strand<'p>> { self.file_name() }

	fn owned(self) -> Self::Owned { self.to_path_buf() }

	fn parent(self) -> Option<Self> { self.parent() }

	fn stem(self) -> Option<Self::Strand<'p>> { self.file_stem() }

	fn try_ends_with<'a, T>(self, child: T) -> Result<bool, EndsWithError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		Ok(self.ends_with(child.as_strand_view()))
	}

	fn try_join<'a, T>(self, path: T) -> Result<Self::Owned, JoinError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		Ok(self.join(path.as_strand_view()))
	}

	fn rsplit_pred<'a, T>(self, pred: T) -> Option<(Self, Self)>
	where
		T: Utf8BytePredictor,
	{
		let b = self.encoded_bytes();
		let (left, right) = b.rsplit_pred_once(pred)?;

		Some(unsafe {
			(
				OsStr::from_encoded_bytes_unchecked(left).as_ref(),
				OsStr::from_encoded_bytes_unchecked(right).as_ref(),
			)
		})
	}

	fn try_rsplit_seq<'a, T>(self, pat: T) -> Result<(Self, Self), RsplitOnceError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		let b = self.encoded_bytes();
		let p = pat.as_strand_view().encoded_bytes();

		let (left, right) = b.rsplit_seq_once(p).ok_or(RsplitOnceError::NotFound)?;
		Ok(unsafe {
			(
				OsStr::from_encoded_bytes_unchecked(left).as_ref(),
				OsStr::from_encoded_bytes_unchecked(right).as_ref(),
			)
		})
	}

	fn try_starts_with<'a, T>(self, base: T) -> Result<bool, StartsWithError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		Ok(self.starts_with(base.as_strand_view()))
	}

	fn try_strip_prefix<'a, T>(self, base: T) -> Result<Self, StripPrefixError>
	where
		T: AsStrandView<'a, Self::Strand<'a>>,
	{
		Ok(self.strip_prefix(base.as_strand_view())?)
	}
}

impl<'a, P> From<P> for PathBufDyn
where
	P: PathLike<'a>,
{
	fn from(value: P) -> Self { value.to_buf_dyn() }
}
