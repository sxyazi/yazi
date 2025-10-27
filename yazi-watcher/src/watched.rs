use std::path::Path;

use hashbrown::HashSet;
use percent_encoding::percent_decode_str;
use yazi_fs::{Xdg, path::PercentEncoding};
use yazi_shared::{scheme::SchemeRef, url::{AsUrl, UrlBuf, UrlLike}};

#[derive(Debug, Default)]
pub struct Watched(HashSet<UrlBuf>);

impl Watched {
	#[inline]
	pub(crate) fn contains(&self, url: impl AsUrl) -> bool { self.0.contains(&url.as_url()) }

	#[inline]
	pub(crate) fn diff(&self, new: &HashSet<UrlBuf>) -> (Vec<UrlBuf>, Vec<UrlBuf>) {
		(self.0.difference(new).cloned().collect(), new.difference(&self.0).cloned().collect())
	}

	#[inline]
	pub(crate) fn insert(&mut self, url: impl Into<UrlBuf>) { self.0.insert(url.into()); }

	#[inline]
	pub(crate) fn paths(&self) -> impl Iterator<Item = &Path> {
		self.0.iter().filter_map(|u| u.as_path())
	}

	#[inline]
	pub(crate) fn remove(&mut self, url: impl AsUrl) { self.0.remove(&url.as_url()); }

	pub(super) fn find_by_cache(&self, cache: &Path) -> Option<&UrlBuf> {
		let mut it = cache.strip_prefix(Xdg::cache_dir()).ok()?.components();

		let l1 = it.next()?.as_os_str().to_str()?;
		let (l2, rel) =
			if let Ok(p) = it.as_path().strip_prefix(".%2F") { (p, true) } else { (it.as_path(), false) };

		let domain = percent_decode_str(l1.strip_prefix("sftp-")?).decode_utf8().ok()?;
		let loc = l2.percent_decode();

		self.0.iter().find(|u| {
			if u.scheme != SchemeRef::Sftp(&domain) {
				return false;
			}

			let mut it = u.loc.components();
			if it.next() == Some(std::path::Component::RootDir) {
				!rel && it.as_path().as_os_str().as_encoded_bytes() == loc.as_ref()
			} else {
				rel && u.loc.as_os_str().as_encoded_bytes() == loc.as_ref()
			}
		})
	}
}
