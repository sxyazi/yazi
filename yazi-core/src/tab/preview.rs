use yazi_fs::{FsHash64, file::File};
use yazi_shared::{id::Id, scope::Scope, url::UrlBuf};

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

	pub fn same_url(&self, url: &UrlBuf) -> bool { matches!(&self.lock, Some(l) if l.url == *url) }

	pub fn same_folder(&self, url: &UrlBuf) -> bool { self.folder_lock.as_ref() == Some(url) }

	pub fn same_file(&self, file: &File, mime: &str) -> bool {
		self.same_url(&file.url)
			&& matches!(&self.lock, Some(l) if l.sig == PreviewSig::new(file, mime).hash_id())
	}

	pub(super) fn same_lock(&self, sig: Id) -> bool {
		self.lock.as_ref().is_some_and(|l| l.sig == sig && l.skip == self.skip)
	}
}
