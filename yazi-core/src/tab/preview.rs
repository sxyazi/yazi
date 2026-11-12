use yazi_fs::{FsHash64, file::File};
use yazi_shared::{id::Id, scope::Scope, url::{AsUrl, UrlBuf}};

use super::{PreviewLock, PreviewSig};

pub struct Preview {
	pub lock:        Option<PreviewLock>,
	pub skip:        usize,
	pub folder_lock: Option<UrlBuf>,

	pub scope: Scope,
}

impl Preview {
	pub fn new(parent: &Scope) -> Self {
		Self { lock: None, skip: 0, folder_lock: None, scope: parent.child() }
	}

	pub fn same_url(&self, url: impl AsUrl) -> bool {
		self.lock.as_ref().is_some_and(|l| l.url == url.as_url())
	}

	pub fn same_folder(&self, url: impl AsUrl) -> bool {
		self.folder_lock.as_ref().is_some_and(|u| *u == url.as_url())
	}

	pub fn same_file(&self, file: &File, mime: &str) -> bool {
		self.same_url(file)
			&& matches!(&self.lock, Some(l) if l.sig == PreviewSig::new(file, mime).hash_id())
	}

	pub(super) fn same_lock(&self, sig: Id) -> bool {
		self.lock.as_ref().is_some_and(|l| l.sig == sig && l.skip == self.skip)
	}
}
