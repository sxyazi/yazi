use std::{mem, path::PathBuf};

use tokio::sync::mpsc;
use yazi_fs::cha::Cha;
use yazi_shared::{CompletionToken, Id, url::UrlBuf};

// --- Copy
#[derive(Clone, Debug)]
pub(crate) struct FileInCopy {
	pub(crate) id:     Id,
	pub(crate) from:   UrlBuf,
	pub(crate) to:     UrlBuf,
	pub(crate) force:  bool,
	pub(crate) cha:    Option<Cha>,
	pub(crate) follow: bool,
	pub(crate) retry:  u8,
	pub(crate) done:   CompletionToken,
}

impl FileInCopy {
	pub(super) fn into_link(self) -> FileInLink {
		FileInLink {
			id:       self.id,
			from:     self.from,
			to:       self.to,
			force:    true,
			cha:      self.cha,
			resolve:  true,
			relative: false,
			delete:   false,
		}
	}
}

// --- Cut
#[derive(Clone, Debug)]
pub(crate) struct FileInCut {
	pub(crate) id:     Id,
	pub(crate) from:   UrlBuf,
	pub(crate) to:     UrlBuf,
	pub(crate) force:  bool,
	pub(crate) cha:    Option<Cha>,
	pub(crate) follow: bool,
	pub(crate) retry:  u8,
	pub(crate) done:   CompletionToken,
	pub(crate) drop:   Option<mpsc::Sender<()>>,
}

impl Drop for FileInCut {
	fn drop(&mut self) { _ = self.drop.take(); }
}

impl FileInCut {
	pub(super) fn into_link(mut self) -> FileInLink {
		FileInLink {
			id:       self.id,
			from:     mem::take(&mut self.from),
			to:       mem::take(&mut self.to),
			force:    true,
			cha:      self.cha,
			resolve:  true,
			relative: false,
			delete:   true,
		}
	}

	pub(super) fn with_drop(mut self, drop: &mpsc::Sender<()>) -> Self {
		self.drop = Some(drop.clone());
		self
	}
}

// --- Link
#[derive(Clone, Debug)]
pub(crate) struct FileInLink {
	pub(crate) id:       Id,
	pub(crate) from:     UrlBuf,
	pub(crate) to:       UrlBuf,
	pub(crate) force:    bool,
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
	pub(crate) force:  bool,
	pub(crate) cha:    Option<Cha>,
	pub(crate) follow: bool,
}

// --- Delete
#[derive(Clone, Debug)]
pub(crate) struct FileInDelete {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
	pub(crate) cha:    Option<Cha>,
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
	pub(crate) done:  CompletionToken,
}

// --- Upload
#[derive(Clone, Debug)]
pub(crate) struct FileInUpload {
	pub(crate) id:    Id,
	pub(crate) url:   UrlBuf,
	pub(crate) cha:   Option<Cha>,
	pub(crate) cache: Option<PathBuf>,
	pub(crate) done:  CompletionToken,
}
