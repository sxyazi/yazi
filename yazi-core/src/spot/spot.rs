use yazi_fs::file::File;
use yazi_shared::{scope::Scope, url::AsUrl};

use crate::spot::SpotLock;

pub struct Spot {
	pub lock: Option<SpotLock>,
	pub skip: usize,

	pub(super) scope: Scope,
}

impl Spot {
	pub fn new(parent: &Scope) -> Self { Self { lock: None, skip: 0, scope: parent.child() } }

	pub fn visible(&self) -> bool { self.lock.is_some() }

	pub fn same_url(&self, url: impl AsUrl) -> bool {
		self.lock.as_ref().is_some_and(|l| l.url == url.as_url())
	}

	fn same_file(&self, file: &File, mime: &str) -> bool {
		self.same_url(file)
			&& self
				.lock
				.as_ref()
				.is_some_and(|l| file.stat.hits(l.stat) && file.lstat().hits(l.lstat) && mime == l.mime)
	}

	pub(super) fn same_lock(&self, file: &File, mime: &str) -> bool {
		self.same_file(file, mime) && self.lock.as_ref().is_some_and(|l| self.skip == l.skip)
	}
}
