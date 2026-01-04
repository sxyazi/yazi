use std::{borrow::Cow, ffi::OsStr, path::{Path, PathBuf}};

use yazi_shared::{path::{AsPath, PathDyn}, url::{AsUrl, Url, UrlBuf, UrlCow}};

use crate::{FsHash128, FsScheme, path::PercentEncoding};

pub trait FsUrl<'a> {
	fn cache(&self) -> Option<PathBuf>;

	fn cache_lock(&self) -> Option<PathBuf>;

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
		fn with_loc(loc: PathDyn, mut root: PathBuf) -> PathBuf {
			let mut it = loc.components();
			if it.next() == Some(yazi_shared::path::Component::RootDir) {
				root.push(it.as_path().percent_encode());
			} else {
				root.push(".%2F");
				root.push(loc.percent_encode());
			}
			root
		}

		self.scheme().cache().map(|root| with_loc(self.loc(), root))
	}

	fn cache_lock(&self) -> Option<PathBuf> {
		self.scheme().cache().map(|mut root| {
			root.push("%lock");
			root.push(format!("{:x}", self.hash_u128()));
			root
		})
	}

	fn unified_path(self) -> Cow<'a, Path> {
		match self {
			Self::Regular(loc) | Self::Search { loc, .. } => loc.as_inner().into(),
			Self::Archive { .. } | Self::Sftp { .. } => {
				self.cache().expect("non-local URL should have a cache path").into()
			}
		}
	}
}

impl FsUrl<'_> for UrlBuf {
	fn cache(&self) -> Option<PathBuf> { self.as_url().cache() }

	fn cache_lock(&self) -> Option<PathBuf> { self.as_url().cache_lock() }

	fn unified_path(self) -> Cow<'static, Path> {
		match self {
			Self::Regular(loc) | Self::Search { loc, .. } => loc.into_inner().into(),
			Self::Archive { .. } | Self::Sftp { .. } => {
				self.cache().expect("non-local URL should have a cache path").into()
			}
		}
	}
}

impl<'a> FsUrl<'a> for UrlCow<'a> {
	fn cache(&self) -> Option<PathBuf> { self.as_url().cache() }

	fn cache_lock(&self) -> Option<PathBuf> { self.as_url().cache_lock() }

	fn unified_path(self) -> Cow<'a, Path> {
		match self {
			Self::Regular(loc) | Self::Search { loc, .. } => loc.into_inner().into(),
			Self::RegularRef(loc) | Self::SearchRef { loc, .. } => loc.as_inner().into(),
			Self::Archive { .. } | Self::ArchiveRef { .. } | Self::Sftp { .. } | Self::SftpRef { .. } => {
				self.cache().expect("non-local URL should have a cache path").into()
			}
		}
	}
}
