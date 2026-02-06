use std::{mem, path::PathBuf};

use tokio::sync::mpsc;
use yazi_fs::cha::Cha;
use yazi_shared::{CompletionToken, Id, url::UrlBuf};

#[derive(Debug)]
pub(crate) enum FileIn {
	Copy(FileInCopy),
	CopyDo(FileInCopy),
	Cut(FileInCut),
	CutDo(FileInCut),
	Link(FileInLink),
	LinkDo(FileInLink),
	Hardlink(FileInHardlink),
	HardlinkDo(FileInHardlink),
	Delete(FileInDelete),
	DeleteDo(FileInDelete),
	Trash(FileInTrash),
	TrashDo(FileInTrash),
	Download(FileInDownload),
	DownloadDo(FileInDownload),
	Upload(FileInUpload),
	UploadDo(FileInUpload),
}

impl_from_in! {
	Copy(FileInCopy),
	Cut(FileInCut),
	Link(FileInLink),
	Hardlink(FileInHardlink),
	Delete(FileInDelete),
	Trash(FileInTrash),
	Download(FileInDownload),
	Upload(FileInUpload),
}

impl FileIn {
	pub(crate) fn id(&self) -> Id {
		match self {
			Self::Copy(r#in) => r#in.id,
			Self::CopyDo(r#in) => r#in.id,
			Self::Cut(r#in) => r#in.id,
			Self::CutDo(r#in) => r#in.id,
			Self::Link(r#in) => r#in.id,
			Self::LinkDo(r#in) => r#in.id,
			Self::Hardlink(r#in) => r#in.id,
			Self::HardlinkDo(r#in) => r#in.id,
			Self::Delete(r#in) => r#in.id,
			Self::DeleteDo(r#in) => r#in.id,
			Self::Trash(r#in) => r#in.id,
			Self::TrashDo(r#in) => r#in.id,
			Self::Download(r#in) => r#in.id,
			Self::DownloadDo(r#in) => r#in.id,
			Self::Upload(r#in) => r#in.id,
			Self::UploadDo(r#in) => r#in.id,
		}
	}

	pub(crate) fn into_doable(self) -> Self {
		match self {
			Self::Copy(r#in) => Self::CopyDo(r#in),
			Self::CopyDo(_) => self,
			Self::Cut(r#in) => Self::CutDo(r#in),
			Self::CutDo(_) => self,
			Self::Link(r#in) => Self::LinkDo(r#in),
			Self::LinkDo(_) => self,
			Self::Hardlink(r#in) => Self::HardlinkDo(r#in),
			Self::HardlinkDo(_) => self,
			Self::Delete(r#in) => Self::DeleteDo(r#in),
			Self::DeleteDo(_) => self,
			Self::Trash(r#in) => Self::TrashDo(r#in),
			Self::TrashDo(_) => self,
			Self::Download(r#in) => Self::DownloadDo(r#in),
			Self::DownloadDo(_) => self,
			Self::Upload(r#in) => Self::UploadDo(r#in),
			Self::UploadDo(_) => self,
		}
	}
}

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
