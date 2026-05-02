use std::{ops::{Deref, DerefMut}, path::Path};

use hashbrown::HashSet;
use percent_encoding::percent_decode_str;
use yazi_fs::{Xdg, path::PercentEncoding};
use yazi_shared::{path::{Component, PathBufDyn, PathDyn, PathLike}, pool::InternStr, scheme::SchemeKind, url::{AsUrl, UrlBuf}};

use crate::Watchee;

#[derive(Debug, Default)]
pub struct Watched(HashSet<Watchee<'static>>);

impl Deref for Watched {
	type Target = HashSet<Watchee<'static>>;

	fn deref(&self) -> &Self::Target { &self.0 }
}

impl DerefMut for Watched {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

impl Watched {
	pub(super) fn contains_url(&self, url: impl AsUrl) -> bool {
		let url = url.as_url();
		if url.as_local().is_some() {
			self.0.contains(&Watchee::Local(url.into(), false))
				|| self.0.contains(&Watchee::Local(url.into(), true))
		} else {
			self.0.contains(&Watchee::Remote(url.into()))
		}
	}

	pub(super) fn contains_path(&self, path: &Path) -> bool {
		self.0.iter().any(|watchee| watchee.as_url().as_local() == Some(path))
	}

	pub(super) fn paths(&self) -> impl Iterator<Item = &Path> {
		self.0.iter().filter_map(|watchee| watchee.as_url().as_local())
	}

	pub(super) fn find_by_cache(&self, cache: PathDyn) -> Option<UrlBuf> {
		let mut it = cache.try_strip_prefix(Xdg::temp_dir()).ok()?.components();

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
		if self.contains_url(&url) { Some(url) } else { None }
	}
}
