use std::path::Path;

use hashbrown::HashSet;
use yazi_shared::url::{AsUrl, UrlBuf, UrlLike};

#[derive(Default)]
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
}
