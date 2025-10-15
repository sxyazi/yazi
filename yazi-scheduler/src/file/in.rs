use std::path::PathBuf;

use yazi_fs::cha::Cha;
use yazi_shared::{Id, url::UrlBuf};

// --- Paste
#[derive(Clone, Debug)]
pub(crate) struct FileInPaste {
	pub(crate) id:     Id,
	pub(crate) from:   UrlBuf,
	pub(crate) to:     UrlBuf,
	pub(crate) cha:    Option<Cha>,
	pub(crate) cut:    bool,
	pub(crate) follow: bool,
	pub(crate) retry:  u8,
}

impl FileInPaste {
	pub(super) fn spawn(&self, from: UrlBuf, to: UrlBuf, cha: Cha) -> Self {
		Self {
			id: self.id,
			from,
			to,
			cha: Some(cha),
			cut: self.cut,
			follow: self.follow,
			retry: self.retry,
		}
	}

	pub(super) fn into_link(self) -> FileInLink {
		FileInLink {
			id:       self.id,
			from:     self.from,
			to:       self.to,
			cha:      self.cha,
			resolve:  true,
			relative: false,
			delete:   self.cut,
		}
	}
}

// --- Link
#[derive(Clone, Debug)]
pub(crate) struct FileInLink {
	pub(crate) id:       Id,
	pub(crate) from:     UrlBuf,
	pub(crate) to:       UrlBuf,
	pub(crate) cha:      Option<Cha>,
	pub(crate) resolve:  bool,
	pub(crate) relative: bool,
	pub(crate) delete:   bool,
}

// --- Hardlink
#[derive(Clone, Debug)]
pub(crate) struct FileInHardlink {
	pub(crate) id:     Id,
	pub(crate) from:   UrlBuf,
	pub(crate) to:     UrlBuf,
	pub(crate) cha:    Option<Cha>,
	pub(crate) follow: bool,
}

impl FileInHardlink {
	pub(super) fn spawn(&self, from: UrlBuf, to: UrlBuf, cha: Cha) -> Self {
		Self { id: self.id, from, to, cha: Some(cha), follow: self.follow }
	}
}

// --- Delete
#[derive(Clone, Debug)]
pub(crate) struct FileInDelete {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
	pub(crate) length: u64,
}

// --- Trash
#[derive(Clone, Debug)]
pub(crate) struct FileInTrash {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
}

// --- Download
#[derive(Clone, Debug)]
pub(crate) struct FileInDownload {
	pub(crate) id:    Id,
	pub(crate) url:   UrlBuf,
	pub(crate) cha:   Option<Cha>,
	pub(crate) retry: u8,
}

impl FileInDownload {
	pub(super) fn spawn(&self, url: UrlBuf, cha: Cha) -> Self {
		Self { id: self.id, url, cha: Some(cha), retry: self.retry }
	}
}

// --- Upload
#[derive(Clone, Debug)]
pub(crate) struct FileInUpload {
	pub(crate) id:  Id,
	pub(crate) url: UrlBuf,
}

// --- UploadDo
#[derive(Clone, Debug)]
pub(crate) struct FileInUploadDo {
	pub(crate) id:    Id,
	pub(crate) url:   UrlBuf,
	pub(crate) cha:   Cha,
	pub(crate) cache: PathBuf,
}
