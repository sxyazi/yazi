use std::{borrow::Cow, ffi::OsStr, path::{Path, PathBuf}};

use anyhow::Result;

use crate::{loc::Loc, path::{AsPathRef, EndsWithError, JoinError, PathDyn, StartsWithError, StripPrefixError}, scheme::{SchemeKind, SchemeRef}, strand::{AsStrand, Strand}, url::{Components, Display, Url, UrlBuf, UrlCow}};

// --- AsUrl
pub trait AsUrl {
	fn as_url(&self) -> Url<'_>;
}

impl AsUrl for Path {
	#[inline]
	fn as_url(&self) -> Url<'_> { Url::Regular(Loc::bare(self)) }
}

impl AsUrl for &Path {
	#[inline]
	fn as_url(&self) -> Url<'_> { (*self).as_url() }
}

impl AsUrl for PathBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> { self.as_path().as_url() }
}

impl AsUrl for &PathBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> { (*self).as_path().as_url() }
}

impl AsUrl for PathDyn<'_> {
	#[inline]
	fn as_url(&self) -> Url<'_> {
		match *self {
			Self::Os(p) => p.as_url(),
		}
	}
}

impl AsUrl for Url<'_> {
	#[inline]
	fn as_url(&self) -> Url<'_> { *self }
}

impl AsUrl for UrlBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> {
		match self {
			Self::Regular(loc) => Url::Regular(loc.as_loc()),
			Self::Search { loc, domain } => Url::Search { loc: loc.as_loc(), domain },
			Self::Archive { loc, domain } => Url::Archive { loc: loc.as_loc(), domain },
			Self::Sftp { loc, domain } => Url::Sftp { loc: loc.as_loc(), domain },
		}
	}
}

impl AsUrl for &UrlBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> { (**self).as_url() }
}

impl AsUrl for &mut UrlBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> { (**self).as_url() }
}

impl AsUrl for UrlCow<'_> {
	fn as_url(&self) -> Url<'_> {
		match self {
			Self::Regular(loc) => Url::Regular(loc.as_loc()),
			Self::Search { loc, domain } => Url::Search { loc: loc.as_loc(), domain },
			Self::Archive { loc, domain } => Url::Archive { loc: loc.as_loc(), domain },
			Self::Sftp { loc, domain } => Url::Sftp { loc: loc.as_loc(), domain },

			Self::RegularRef(loc) => Url::Regular(*loc),
			Self::SearchRef { loc, domain } => Url::Search { loc: *loc, domain },
			Self::ArchiveRef { loc, domain } => Url::Archive { loc: *loc, domain },
			Self::SftpRef { loc, domain } => Url::Sftp { loc: *loc, domain },
		}
	}
}

impl AsUrl for &UrlCow<'_> {
	#[inline]
	fn as_url(&self) -> Url<'_> { (**self).as_url() }
}

impl<'a, T> From<&'a T> for Url<'a>
where
	T: AsUrl + ?Sized,
{
	fn from(value: &'a T) -> Self { value.as_url() }
}

impl<'a, T> From<&'a mut T> for Url<'a>
where
	T: AsUrl + ?Sized,
{
	fn from(value: &'a mut T) -> Self { value.as_url() }
}

// UrlLike
pub trait UrlLike
where
	Self: AsUrl,
{
	fn as_local(&self) -> Option<&Path> { self.as_url().as_local() }

	fn base(&self) -> Url<'_> { self.as_url().base() }

	fn components(&self) -> Components<'_> { self.as_url().into() }

	fn covariant(&self, other: impl AsUrl) -> bool { self.as_url().covariant(other) }

	fn display(&self) -> Display<'_> { self.as_url().into() }

	fn ext(&self) -> Option<Strand<'_>> { self.as_url().ext() }

	fn has_base(&self) -> bool { self.as_url().has_base() }

	fn has_root(&self) -> bool { self.as_url().has_root() }

	fn has_trail(&self) -> bool { self.as_url().has_trail() }

	fn is_absolute(&self) -> bool { self.as_url().is_absolute() }

	fn kind(&self) -> SchemeKind { self.as_url().kind() }

	fn loc(&self) -> PathDyn<'_> { self.as_url().loc() }

	fn name(&self) -> Option<Strand<'_>> { self.as_url().name() }

	fn os_str(&self) -> Cow<'_, OsStr> { self.components().os_str() }

	fn pair(&self) -> Option<(Url<'_>, PathDyn<'_>)> { self.as_url().pair() }

	fn parent(&self) -> Option<Url<'_>> { self.as_url().parent() }

	fn scheme(&self) -> SchemeRef<'_> { self.as_url().scheme() }

	fn stem(&self) -> Option<Strand<'_>> { self.as_url().stem() }

	fn trail(&self) -> Url<'_> { self.as_url().trail() }

	fn try_ends_with(&self, child: impl AsUrl) -> Result<bool, EndsWithError> {
		self.as_url().try_ends_with(child)
	}

	fn try_join(&self, path: impl AsStrand) -> Result<UrlBuf, JoinError> {
		self.as_url().try_join(path)
	}

	fn try_replace<'a>(&self, take: usize, path: impl AsPathRef<'a>) -> Result<UrlCow<'a>> {
		self.as_url().try_replace(take, path)
	}

	fn try_starts_with(&self, base: impl AsUrl) -> Result<bool, StartsWithError> {
		self.as_url().try_starts_with(base)
	}

	fn try_strip_prefix(&self, base: impl AsUrl) -> Result<PathDyn<'_>, StripPrefixError> {
		self.as_url().try_strip_prefix(base)
	}

	fn uri(&self) -> PathDyn<'_> { self.as_url().uri() }

	fn urn(&self) -> PathDyn<'_> { self.as_url().urn() }
}

impl UrlLike for UrlBuf {}
impl UrlLike for UrlCow<'_> {}
