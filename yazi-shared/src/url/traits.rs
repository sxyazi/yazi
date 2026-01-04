use std::path::{Path, PathBuf};

use crate::{loc::Loc, url::{Url, UrlBuf, UrlCow}};

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
	fn as_url(&self) -> Url<'_> { (**self).as_url() }
}

impl AsUrl for super::Components<'_> {
	fn as_url(&self) -> Url<'_> { self.url() }
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
