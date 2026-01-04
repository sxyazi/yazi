use std::path::Path;

use hashbrown::HashSet;
use percent_encoding::percent_decode_str;
use yazi_fs::{Xdg, path::PercentEncoding};
use yazi_shared::{path::{Component, PathBufDyn, PathDyn, PathLike}, pool::InternStr, scheme::SchemeKind, url::{AsUrl, UrlBuf, UrlLike}};

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
		self.0.iter().filter_map(|u| u.as_local())
	}

	#[inline]
	pub(crate) fn remove(&mut self, url: impl AsUrl) { self.0.remove(&url.as_url()); }

	pub(super) fn find_by_cache(&self, cache: PathDyn) -> Option<UrlBuf> {
		let mut it = cache.try_strip_prefix(Xdg::cache_dir()).ok()?.components();

		// Parse domain
		let domain = it.next()?.as_normal()?.to_str().ok()?;
		let domain = percent_decode_str(domain.strip_prefix("sftp-")?).decode_utf8().ok()?.intern();

		// Parse path
		let (path, abs) =
			if let Ok(p) = it.path().try_strip_prefix(".%2F") { (p, false) } else { (it.path(), true) };
		let path = path.percent_decode(SchemeKind::Sftp).ok()?;
		let path = PathBufDyn::from_components(
			SchemeKind::Sftp,
			Some(Component::RootDir).filter(|_| abs).into_iter().chain(path.components()),
		)
		.ok()?;

		let url = UrlBuf::Sftp { loc: path.into_unix().ok()?.into(), domain };
		if self.contains(&url) { Some(url) } else { None }
	}
}
