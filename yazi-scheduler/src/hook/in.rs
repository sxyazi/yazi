use std::borrow::Cow;

use yazi_shared::{Id, url::{UrlBuf, UrlLike}};

use crate::{Task, TaskIn, TaskProg};

#[derive(Debug)]
pub(crate) enum HookIn {
	Copy(HookInOutCopy),
	Cut(HookInOutCut),
	Delete(HookInDelete),
	Trash(HookInTrash),
	Link(HookInOutLink),
	Hardlink(HookInOutHardlink),
	Download(HookInDownload),
	Upload(HookInUpload),
	Preload(HookInPreload),
}

impl_from_in!(
	Copy(HookInOutCopy),
	Cut(HookInOutCut),
	Delete(HookInDelete),
	Trash(HookInTrash),
	Link(HookInOutLink),
	Hardlink(HookInOutHardlink),
	Download(HookInDownload),
	Upload(HookInUpload),
	Preload(HookInPreload),
);

impl TaskIn for HookIn {
	type Prog = ();

	fn id(&self) -> Id {
		match self {
			Self::Copy(r#in) => r#in.id(),
			Self::Cut(r#in) => r#in.id(),
			Self::Delete(r#in) => r#in.id(),
			Self::Trash(r#in) => r#in.id(),
			Self::Link(r#in) => r#in.id(),
			Self::Hardlink(r#in) => r#in.id(),
			Self::Download(r#in) => r#in.id(),
			Self::Upload(r#in) => r#in.id(),
			Self::Preload(r#in) => r#in.id(),
		}
	}

	fn set_id(&mut self, id: Id) -> &mut Self {
		match self {
			Self::Copy(r#in) => r#in.id = id,
			Self::Cut(r#in) => r#in.id = id,
			Self::Delete(r#in) => r#in.id = id,
			Self::Trash(r#in) => r#in.id = id,
			Self::Link(r#in) => r#in.id = id,
			Self::Hardlink(r#in) => r#in.id = id,
			Self::Download(r#in) => r#in.id = id,
			Self::Upload(r#in) => r#in.id = id,
			Self::Preload(r#in) => r#in.id = id,
		}
		self
	}

	fn title(&self) -> Cow<'_, str> {
		match self {
			Self::Copy(r#in) => r#in.title(),
			Self::Cut(r#in) => r#in.title(),
			Self::Delete(r#in) => r#in.title(),
			Self::Trash(r#in) => r#in.title(),
			Self::Link(r#in) => r#in.title(),
			Self::Hardlink(r#in) => r#in.title(),
			Self::Download(r#in) => r#in.title(),
			Self::Upload(r#in) => r#in.title(),
			Self::Preload(r#in) => r#in.title(),
		}
	}
}

// --- Copy
#[derive(Debug)]
pub(crate) struct HookInOutCopy {
	pub(crate) id:   Id,
	pub(crate) from: UrlBuf,
	pub(crate) to:   UrlBuf,
}

impl TaskIn for HookInOutCopy {
	type Prog = ();

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> {
		format!("Hook: copy {} to {}", self.from.display(), self.to.display()).into()
	}
}

impl HookInOutCopy {
	pub(crate) fn new<U>(from: U, to: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, from: from.into(), to: to.into() }
	}

	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileCopy(_) = &task.prog {
			task.with_hook(self);
		}
	}
}

// --- Cut
#[derive(Debug)]
pub(crate) struct HookInOutCut {
	pub(crate) id:   Id,
	pub(crate) from: UrlBuf,
	pub(crate) to:   UrlBuf,
}

impl TaskIn for HookInOutCut {
	type Prog = ();

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> {
		format!("Hook: cut {} to {}", self.from.display(), self.to.display()).into()
	}
}

impl HookInOutCut {
	pub(crate) fn new<U>(from: U, to: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, from: from.into(), to: to.into() }
	}

	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileCut(_) = &task.prog {
			task.with_hook(self);
		}
	}
}

// --- Delete
#[derive(Debug)]
pub(crate) struct HookInDelete {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
}

impl TaskIn for HookInDelete {
	type Prog = ();

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Hook: delete {}", self.target.display()).into() }
}

impl HookInDelete {
	pub(crate) fn new<U>(target: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, target: target.into() }
	}
}

// --- Trash
#[derive(Debug)]
pub(crate) struct HookInTrash {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
}

impl TaskIn for HookInTrash {
	type Prog = ();

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Hook: trash {}", self.target.display()).into() }
}

impl HookInTrash {
	pub(crate) fn new<U>(target: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, target: target.into() }
	}
}

// --- Link
#[derive(Debug)]
pub(crate) struct HookInOutLink {
	pub(crate) id:   Id,
	#[allow(dead_code)]
	pub(crate) from: UrlBuf,
	pub(crate) to:   UrlBuf,
}

impl TaskIn for HookInOutLink {
	type Prog = ();

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> {
		format!("Hook: link {} to {}", self.from.display(), self.to.display()).into()
	}
}

impl HookInOutLink {
	pub(crate) fn new<U>(from: U, to: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, from: from.into(), to: to.into() }
	}

	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileLink(_) = &task.prog {
			task.with_hook(self);
		}
	}
}

// --- Hardlink
#[derive(Debug)]
pub(crate) struct HookInOutHardlink {
	pub(crate) id:   Id,
	#[allow(dead_code)]
	pub(crate) from: UrlBuf,
	pub(crate) to:   UrlBuf,
}

impl TaskIn for HookInOutHardlink {
	type Prog = ();

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> {
		format!("Hook: hardlink {} to {}", self.from.display(), self.to.display()).into()
	}
}

impl HookInOutHardlink {
	pub(crate) fn new<U>(from: U, to: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, from: from.into(), to: to.into() }
	}

	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileHardlink(_) = &task.prog {
			task.with_hook(self);
		}
	}
}

// --- Download
#[derive(Debug)]
pub(crate) struct HookInDownload {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
}

impl TaskIn for HookInDownload {
	type Prog = ();

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Hook: download {}", self.target.display()).into() }
}

impl HookInDownload {
	pub(crate) fn new<U>(target: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, target: target.into() }
	}
}

// --- Upload
#[derive(Debug)]
pub(crate) struct HookInUpload {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
}

impl TaskIn for HookInUpload {
	type Prog = ();

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Hook: upload {}", self.target.display()).into() }
}

impl HookInUpload {
	pub(crate) fn new<U>(target: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, target: target.into() }
	}
}

// --- Preload
#[derive(Debug)]
pub(crate) struct HookInPreload {
	pub(crate) id:   Id,
	pub(crate) idx:  u8,
	pub(crate) hash: u64,
}

impl TaskIn for HookInPreload {
	type Prog = ();

	fn id(&self) -> Id { self.id }

	fn set_id(&mut self, id: Id) -> &mut Self {
		self.id = id;
		self
	}

	fn title(&self) -> Cow<'_, str> { format!("Hook: run {}-th preloader", self.idx).into() }
}

impl HookInPreload {
	pub(crate) fn new(idx: u8, hash: u64) -> Self { Self { id: Id::ZERO, idx, hash } }
}
