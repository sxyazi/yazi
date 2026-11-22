use std::{borrow::Cow, path::PathBuf};

use anyhow::Context;
use mlua::{FromLua, Lua, Table, Value};
use tokio::sync::mpsc;
use yazi_fs::{file::File, stat::Stat};
use yazi_shared::{id::Id, path::{PathCow, PathLike}, scope::Scope, url::{UrlBuf, UrlLike}};

use crate::{TaskIn, custom::CustomIn, file::{FileProgCopy, FileProgDelete, FileProgDownload, FileProgHardlink, FileProgLink, FileProgMove, FileProgTrash, FileProgUpload}};

#[derive(Debug)]
pub(crate) enum FileIn {
	Copy(FileInCopy),
	CopyDo(FileInCopy),
	Move(FileInMove),
	MoveDo(FileInMove),
	Link(FileInLink),
	LinkDo(FileInLink),
	Hardlink(FileInHardlink),
	HardlinkDo(FileInHardlink),
	Delete(FileInDelete),
	Trash(FileInTrash),
	TrashDo(FileInTrash),
	Download(FileInDownload),
	DownloadDo(FileInDownload),
	Upload(FileInUpload),
	UploadDo(FileInUpload),
	Custom(CustomIn),
}

impl TaskIn for FileIn {
	type Prog = ();

	fn id(&self) -> Id {
		match self {
			Self::Copy(r#in) | Self::CopyDo(r#in) => r#in.id(),
			Self::Move(r#in) | Self::MoveDo(r#in) => r#in.id(),
			Self::Link(r#in) | Self::LinkDo(r#in) => r#in.id(),
			Self::Hardlink(r#in) | Self::HardlinkDo(r#in) => r#in.id(),
			Self::Delete(r#in) => r#in.id(),
			Self::Trash(r#in) | Self::TrashDo(r#in) => r#in.id(),
			Self::Download(r#in) | Self::DownloadDo(r#in) => r#in.id(),
			Self::Upload(r#in) | Self::UploadDo(r#in) => r#in.id(),
			Self::Custom(r#in) => r#in.id(),
		}
	}

	fn set_id(&mut self, id: Id) -> &mut Self {
		match self {
			Self::Copy(r#in) | Self::CopyDo(r#in) => _ = r#in.set_id(id),
			Self::Move(r#in) | Self::MoveDo(r#in) => _ = r#in.set_id(id),
			Self::Link(r#in) | Self::LinkDo(r#in) => _ = r#in.set_id(id),
			Self::Hardlink(r#in) | Self::HardlinkDo(r#in) => _ = r#in.set_id(id),
			Self::Delete(r#in) => _ = r#in.set_id(id),
			Self::Trash(r#in) | Self::TrashDo(r#in) => _ = r#in.set_id(id),
			Self::Download(r#in) | Self::DownloadDo(r#in) => _ = r#in.set_id(id),
			Self::Upload(r#in) | Self::UploadDo(r#in) => _ = r#in.set_id(id),
			Self::Custom(r#in) => _ = r#in.set_id(id),
		}
		self
	}

	fn scope(&self) -> &Scope {
		match self {
			Self::Copy(r#in) | Self::CopyDo(r#in) => r#in.scope(),
			Self::Move(r#in) | Self::MoveDo(r#in) => r#in.scope(),
			Self::Link(r#in) | Self::LinkDo(r#in) => r#in.scope(),
			Self::Hardlink(r#in) | Self::HardlinkDo(r#in) => r#in.scope(),
			Self::Delete(r#in) => r#in.scope(),
			Self::Trash(r#in) | Self::TrashDo(r#in) => r#in.scope(),
			Self::Download(r#in) | Self::DownloadDo(r#in) => r#in.scope(),
			Self::Upload(r#in) | Self::UploadDo(r#in) => r#in.scope(),
			Self::Custom(r#in) => r#in.scope(),
		}
	}

	fn title(&self) -> Cow<'_, str> {
		match self {
			Self::Copy(r#in) | Self::CopyDo(r#in) => r#in.title(),
			Self::Move(r#in) | Self::MoveDo(r#in) => r#in.title(),
			Self::Link(r#in) | Self::LinkDo(r#in) => r#in.title(),
			Self::Hardlink(r#in) | Self::HardlinkDo(r#in) => r#in.title(),
			Self::Delete(r#in) => r#in.title(),
			Self::Trash(r#in) | Self::TrashDo(r#in) => r#in.title(),
			Self::Download(r#in) | Self::DownloadDo(r#in) => r#in.title(),
			Self::Upload(r#in) | Self::UploadDo(r#in) => r#in.title(),
			Self::Custom(r#in) => r#in.title(),
		}
	}
}

impl_from_in! {
	Copy(FileInCopy),
	Move(FileInMove),
	Link(FileInLink),
	Hardlink(FileInHardlink),
	Delete(FileInDelete),
	Trash(FileInTrash),
	Download(FileInDownload),
	Upload(FileInUpload),
	Custom(CustomIn),
}

impl FileIn {
	pub(crate) fn into_doable(self) -> Self {
		match self {
			Self::Copy(r#in) => Self::CopyDo(r#in),
			Self::CopyDo(_) => self,
			Self::Move(r#in) => Self::MoveDo(r#in),
			Self::MoveDo(_) => self,
			Self::Link(r#in) => Self::LinkDo(r#in),
			Self::LinkDo(_) => self,
			Self::Hardlink(r#in) => Self::HardlinkDo(r#in),
			Self::HardlinkDo(_) => self,
			Self::Delete(_) => self,
			Self::Trash(r#in) => Self::TrashDo(r#in),
			Self::TrashDo(_) => self,
			Self::Download(r#in) => Self::DownloadDo(r#in),
			Self::DownloadDo(_) => self,
			Self::Upload(r#in) => Self::UploadDo(r#in),
			Self::UploadDo(_) => self,
			Self::Custom(_) => self,
		}
	}
}

// --- Copy
#[derive(Clone, Debug)]
pub struct FileInCopy {
	pub(crate) id:     Id,
	pub(crate) from:   UrlBuf,
	pub(crate) to:     UrlBuf,
	pub(crate) follow: bool,
	pub(crate) force:  bool,
	pub(crate) stat:   Option<Stat>,
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
	pub fn new(from: UrlBuf, to: UrlBuf, follow: bool, force: bool) -> Self {
		Self {
			id: Id::ZERO,
			follow: follow || !from.auth().same_service(to.auth()),
			from,
			to,
			force,
			stat: None,
		}
	}

	pub(super) fn into_link(self) -> FileInLink {
		FileInLink {
			id:       self.id,
			from:     self.from,
			to:       self.to,
			relative: false,
			follow:   true,
			force:    true,
			delete:   false,
			stat:     self.stat,
		}
	}
}

impl FromLua for FileInCopy {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;
		Ok(Self::new(t.raw_get("from")?, t.raw_get("to")?, t.raw_get("follow")?, t.raw_get("force")?))
	}
}
// --- Move
#[derive(Clone, Debug)]
pub struct FileInMove {
	pub(crate) id:     Id,
	pub(crate) from:   UrlBuf,
	pub(crate) to:     UrlBuf,
	pub(crate) follow: bool,
	pub(crate) force:  bool,
	pub(crate) stat:   Option<Stat>,
	pub(crate) drop:   Option<mpsc::Sender<()>>,
}

impl TaskIn for FileInMove {
	type Prog = FileProgMove;

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> {
		format!("Move {} to {}", self.from.display(), self.to.display()).into()
	}
}

impl FileInMove {
	pub fn new(from: UrlBuf, to: UrlBuf, force: bool) -> Self {
		Self {
			follow: !from.auth().same_service(to.auth()),
			id: Id::ZERO,
			from,
			to,
			force,
			stat: None,
			drop: None,
		}
	}

	pub(super) fn into_link(self) -> FileInLink {
		FileInLink {
			id:       self.id,
			from:     self.from,
			to:       self.to,
			relative: false,
			follow:   true,
			force:    true,
			delete:   true,
			stat:     self.stat,
		}
	}

	pub(super) fn with_drop(mut self, drop: &mpsc::Sender<()>) -> Self {
		self.drop = Some(drop.clone());
		self
	}
}

impl FromLua for FileInMove {
	fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
		let t = Table::from_lua(value, lua)?;
		Ok(Self::new(t.raw_get("from")?, t.raw_get("to")?, t.raw_get("force")?))
	}
}

// --- Link
#[derive(Clone, Debug)]
pub struct FileInLink {
	pub(crate) id:       Id,
	pub(crate) from:     UrlBuf,
	pub(crate) to:       UrlBuf,
	pub(crate) relative: bool,
	pub(crate) follow:   bool,
	pub(crate) force:    bool,
	pub(crate) delete:   bool,
	pub(crate) stat:     Option<Stat>,
}

impl FileInLink {
	pub fn new(from: UrlBuf, to: UrlBuf, relative: bool, follow: bool, force: bool) -> Self {
		Self { id: Id::ZERO, from, to, relative, follow, force, delete: false, stat: None }
	}

	pub(super) fn target<'a>(&'a self, file: &'a File) -> anyhow::Result<PathCow<'a>> {
		let path = file.extra.backing().map_or(self.from.loc(), Into::into);
		if !file.is_link() || !self.follow {
			return Ok(path.into());
		}

		let target = file.extra.link_to().context("Cannot determine symbolic link target")?;
		if target.is_absolute() {
			return Ok(target.into());
		}

		let parent = path.parent().context("Symbolic link path has no parent")?;
		Ok(parent.try_join(target)?.into())
	}
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
	pub(crate) follow: bool,
	pub(crate) force:  bool,
	pub(crate) top:    bool,
	pub(crate) stat:   Option<Stat>,
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
	pub(crate) stat:   Option<Stat>,
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
	pub(crate) stat:   Option<Stat>,
	pub(crate) cache:  Option<PathBuf>,
	pub(crate) top:    bool,
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
