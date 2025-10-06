use std::{borrow::Cow, ffi::OsStr, path::{Path, PathBuf}};

use twox_hash::XxHash3_128;
use yazi_shared::{scheme::SchemeRef, url::{AsUrl, Url, UrlBuf, UrlCow}};

use crate::Xdg;

pub trait FsUrl<'a> {
	fn cache(&self) -> Option<PathBuf>;

	fn unified_path(self) -> Cow<'a, Path>;

	fn unified_path_str(self) -> Cow<'a, OsStr>
	where
		Self: Sized,
	{
		match self.unified_path() {
			Cow::Borrowed(p) => p.as_os_str().into(),
			Cow::Owned(p) => p.into_os_string().into(),
		}
	}
}

impl<'a> FsUrl<'a> for Url<'a> {
	fn cache(&self) -> Option<PathBuf> {
		match self.scheme {
			SchemeRef::Regular | SchemeRef::Search(_) => None,
			SchemeRef::Archive(name) => Some(
				Xdg::cache_dir()
					.join(format!("archive-{}", yazi_shared::url::Encode::domain(name)))
					.join(format!("{:x}", XxHash3_128::oneshot(self.loc.bytes()))),
			),
			SchemeRef::Sftp(name) => Some(
				Xdg::cache_dir()
					.join(format!("sftp-{}", yazi_shared::url::Encode::domain(name)))
					.join(format!("{:x}", XxHash3_128::oneshot(self.loc.bytes()))),
			),
		}
	}

	fn unified_path(self) -> Cow<'a, Path> {
		self.cache().map(Cow::Owned).unwrap_or_else(|| Cow::Borrowed(self.loc.as_path()))
	}
}

impl<'a> FsUrl<'a> for UrlBuf {
	fn cache(&self) -> Option<PathBuf> { self.as_url().cache() }

	fn unified_path(self) -> Cow<'a, Path> {
		self.cache().unwrap_or_else(|| self.loc.into_path()).into()
	}
}

impl<'a> FsUrl<'a> for UrlCow<'a> {
	fn cache(&self) -> Option<PathBuf> { self.as_url().cache() }

	fn unified_path(self) -> Cow<'a, Path> {
		match (self.cache(), self) {
			(None, UrlCow::Borrowed { loc, .. }) => loc.as_path().into(),
			(None, UrlCow::Owned { loc, .. }) => loc.into_path().into(),
			(Some(cache), _) => cache.into(),
		}
	}
}
