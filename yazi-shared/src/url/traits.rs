use std::{borrow::Cow, ffi::OsStr, path::{Path, PathBuf}};

use crate::{scheme::SchemeRef, url::{Components, Display, Uri, Url, UrlBuf, UrlCow, Urn}};

// --- AsUrl
pub trait AsUrl {
	fn as_url(&self) -> Url<'_>;
}

impl AsUrl for Path {
	#[inline]
	fn as_url(&self) -> Url<'_> { Url { loc: self.into(), scheme: SchemeRef::Regular } }
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

impl AsUrl for Url<'_> {
	#[inline]
	fn as_url(&self) -> Url<'_> { *self }
}

impl AsUrl for UrlBuf {
	#[inline]
	fn as_url(&self) -> Url<'_> { Url { loc: self.loc.as_loc(), scheme: self.scheme.as_ref() } }
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
	#[inline]
	fn as_url(&self) -> Url<'_> {
		match self {
			UrlCow::Borrowed { loc, scheme } => Url { loc: *loc, scheme: scheme.as_ref() },
			UrlCow::Owned { loc, scheme } => Url { loc: loc.as_loc(), scheme: scheme.as_ref() },
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
	Self: AsUrl + Sized,
{
	fn as_path(&self) -> Option<&Path> { self.as_url().as_path() }

	fn base(&self) -> Option<Url<'_>> { self.as_url().base() }

	fn components(&self) -> Components<'_> { self.as_url().into() }

	fn covariant(&self, other: impl AsUrl) -> bool { self.as_url().covariant(other) }

	fn display(&self) -> Display<'_> { self.as_url().into() }

	fn ends_with(&self, child: impl AsUrl) -> bool { self.as_url().ends_with(child) }

	fn ext(&self) -> Option<&OsStr> { self.as_url().ext() }

	fn has_root(&self) -> bool { self.as_url().has_root() }

	fn has_trail(&self) -> bool { self.as_url().has_trail() }

	fn is_absolute(&self) -> bool { self.as_url().is_absolute() }

	fn join(&self, path: impl AsRef<Path>) -> UrlBuf { self.as_url().join(path) }

	fn name(&self) -> Option<&OsStr> { self.as_url().name() }

	fn os_str(&self) -> Cow<'_, OsStr> { self.components().os_str() }

	fn pair(&self) -> Option<(Url<'_>, &Urn)> { self.as_url().pair() }

	fn parent(&self) -> Option<Url<'_>> { self.as_url().parent() }

	fn starts_with(&self, base: impl AsUrl) -> bool { self.as_url().starts_with(base) }

	fn stem(&self) -> Option<&OsStr> { self.as_url().stem() }

	fn strip_prefix(&self, base: impl AsUrl) -> Option<&Urn> { self.as_url().strip_prefix(base) }

	fn uri(&self) -> &Uri { self.as_url().uri() }

	fn urn(&self) -> &Urn { self.as_url().urn() }
}

impl UrlLike for Url<'_> {}
impl UrlLike for UrlBuf {}
impl UrlLike for UrlCow<'_> {}
