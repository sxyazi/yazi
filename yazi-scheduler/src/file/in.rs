use std::{borrow::Cow, mem, path::PathBuf};

use tokio::sync::mpsc;
use yazi_fs::cha::Cha;
use yazi_shared::{Id, url::{UrlBuf, UrlLike}};

use crate::{TaskIn, file::{FileProgCopy, FileProgCut, FileProgDelete, FileProgDownload, FileProgHardlink, FileProgLink, FileProgTrash, FileProgUpload}};

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

impl TaskIn for FileIn {
	type Prog = ();

	fn id(&self) -> Id {
		match self {
			Self::Copy(r#in) => r#in.id(),
			Self::CopyDo(r#in) => r#in.id(),
			Self::Cut(r#in) => r#in.id(),
			Self::CutDo(r#in) => r#in.id(),
			Self::Link(r#in) => r#in.id(),
			Self::LinkDo(r#in) => r#in.id(),
			Self::Hardlink(r#in) => r#in.id(),
			Self::HardlinkDo(r#in) => r#in.id(),
			Self::Delete(r#in) => r#in.id(),
			Self::DeleteDo(r#in) => r#in.id(),
			Self::Trash(r#in) => r#in.id(),
			Self::TrashDo(r#in) => r#in.id(),
			Self::Download(r#in) => r#in.id(),
			Self::DownloadDo(r#in) => r#in.id(),
			Self::Upload(r#in) => r#in.id(),
			Self::UploadDo(r#in) => r#in.id(),
		}
	}

	fn set_id(&mut self, id: Id) -> &mut Self {
		match self {
			Self::Copy(r#in) => _ = r#in.set_id(id),
			Self::CopyDo(r#in) => _ = r#in.set_id(id),
			Self::Cut(r#in) => _ = r#in.set_id(id),
			Self::CutDo(r#in) => _ = r#in.set_id(id),
			Self::Link(r#in) => _ = r#in.set_id(id),
			Self::LinkDo(r#in) => _ = r#in.set_id(id),
			Self::Hardlink(r#in) => _ = r#in.set_id(id),
			Self::HardlinkDo(r#in) => _ = r#in.set_id(id),
			Self::Delete(r#in) => _ = r#in.set_id(id),
			Self::DeleteDo(r#in) => _ = r#in.set_id(id),
			Self::Trash(r#in) => _ = r#in.set_id(id),
			Self::TrashDo(r#in) => _ = r#in.set_id(id),
			Self::Download(r#in) => _ = r#in.set_id(id),
			Self::DownloadDo(r#in) => _ = r#in.set_id(id),
			Self::Upload(r#in) => _ = r#in.set_id(id),
			Self::UploadDo(r#in) => _ = r#in.set_id(id),
		}
		self
	}

	fn title(&self) -> Cow<'_, str> {
		match self {
			Self::Copy(r#in) => r#in.title(),
			Self::CopyDo(r#in) => r#in.title(),
			Self::Cut(r#in) => r#in.title(),
			Self::CutDo(r#in) => r#in.title(),
			Self::Link(r#in) => r#in.title(),
			Self::LinkDo(r#in) => r#in.title(),
			Self::Hardlink(r#in) => r#in.title(),
			Self::HardlinkDo(r#in) => r#in.title(),
			Self::Delete(r#in) => r#in.title(),
			Self::DeleteDo(r#in) => r#in.title(),
			Self::Trash(r#in) => r#in.title(),
			Self::TrashDo(r#in) => r#in.title(),
			Self::Download(r#in) => r#in.title(),
			Self::DownloadDo(r#in) => r#in.title(),
			Self::Upload(r#in) => r#in.title(),
			Self::UploadDo(r#in) => r#in.title(),
		}
	}
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
}

impl TaskIn for FileInCopy {
	type Prog = FileProgCopy;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> {
		format!("Copy {} to {}", self.from.display(), self.to.display()).into()
	}
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
	pub(crate) drop:   Option<mpsc::Sender<()>>,
}

impl TaskIn for FileInCut {
	type Prog = FileProgCut;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> {
		format!("Cut {} to {}", self.from.display(), self.to.display()).into()
	}
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

impl TaskIn for FileInLink {
	type Prog = FileProgLink;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> {
		format!("Link {} to {}", self.from.display(), self.to.display()).into()
	}
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

impl TaskIn for FileInHardlink {
	type Prog = FileProgHardlink;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> {
		format!("Hardlink {} to {}", self.from.display(), self.to.display()).into()
	}
}

// --- Delete
#[derive(Clone, Debug)]
pub(crate) struct FileInDelete {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
	pub(crate) cha:    Option<Cha>,
}

impl TaskIn for FileInDelete {
	type Prog = FileProgDelete;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Delete {}", self.target.display()).into() }
}

// --- Trash
#[derive(Clone, Debug)]
pub(crate) struct FileInTrash {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
}

impl TaskIn for FileInTrash {
	type Prog = FileProgTrash;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Trash {}", self.target.display()).into() }
}

// --- Download
#[derive(Clone, Debug)]
pub(crate) struct FileInDownload {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
	pub(crate) cha:    Option<Cha>,
	pub(crate) retry:  u8,
}

impl TaskIn for FileInDownload {
	type Prog = FileProgDownload;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Download {}", self.target.display()).into() }
}

// --- Upload
#[derive(Clone, Debug)]
pub(crate) struct FileInUpload {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
	pub(crate) cha:    Option<Cha>,
	pub(crate) cache:  Option<PathBuf>,
}

impl TaskIn for FileInUpload {
	type Prog = FileProgUpload;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Upload {}", self.target.display()).into() }
}
