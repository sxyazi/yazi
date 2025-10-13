use std::{borrow::Cow, ffi::OsStr, path::{Path, PathBuf}};

use yazi_shared::{loc::Loc, scheme::SchemeRef, url::{AsUrl, Url, UrlBuf, UrlCow}};

use crate::{FsHash128, Xdg, path::PercentEncoding};

pub trait FsUrl<'a> {
	fn cache(&self) -> Option<PathBuf>;

	fn cache_lock(&self) -> Option<PathBuf>;

	fn cache_root(&self) -> Option<PathBuf>;

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
		fn with_loc(loc: Loc, mut path: PathBuf) -> PathBuf {
			let mut it = loc.components();
			if it.next() == Some(std::path::Component::RootDir) {
				path.push(it.as_path().percent_encode());
			} else {
				path.push(".%2F");
				path.push(loc.percent_encode());
			}
			path
		}

		self.cache_root().map(|root| with_loc(self.loc, root))
	}

	fn cache_lock(&self) -> Option<PathBuf> {
		self.cache_root().map(|mut root| {
			root.push("%lock");
			root.push(format!("{:x}", self.hash_u128()));
			root
		})
	}

	fn cache_root(&self) -> Option<PathBuf> {
		match self.scheme {
			SchemeRef::Regular | SchemeRef::Search(_) => None,
			SchemeRef::Archive(name) => {
				Some(Xdg::cache_dir().join(format!("archive-{}", yazi_shared::url::Encode::domain(name))))
			}
			SchemeRef::Sftp(name) => {
				Some(Xdg::cache_dir().join(format!("sftp-{}", yazi_shared::url::Encode::domain(name))))
			}
		}
	}

	fn unified_path(self) -> Cow<'a, Path> {
		self.cache().map(Cow::Owned).unwrap_or_else(|| Cow::Borrowed(self.loc.as_path()))
	}
}

impl<'a> FsUrl<'a> for UrlBuf {
	fn cache(&self) -> Option<PathBuf> { self.as_url().cache() }

	fn cache_lock(&self) -> Option<PathBuf> { self.as_url().cache_lock() }

	fn cache_root(&self) -> Option<PathBuf> { self.as_url().cache_root() }

	fn unified_path(self) -> Cow<'a, Path> {
		self.cache().unwrap_or_else(|| self.loc.into_path()).into()
	}
}

impl<'a> FsUrl<'a> for UrlCow<'a> {
	fn cache(&self) -> Option<PathBuf> { self.as_url().cache() }

	fn cache_lock(&self) -> Option<PathBuf> { self.as_url().cache_lock() }

	fn cache_root(&self) -> Option<PathBuf> { self.as_url().cache_root() }

	fn unified_path(self) -> Cow<'a, Path> {
		match (self.cache(), self) {
			(None, UrlCow::Borrowed { loc, .. }) => loc.as_path().into(),
			(None, UrlCow::Owned { loc, .. }) => loc.into_path().into(),
			(Some(cache), _) => cache.into(),
		}
	}
}
