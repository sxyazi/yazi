use yazi_shared::{Id, url::UrlBuf};

use crate::{Task, TaskProg, file::FileInCopy};

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
);

impl HookIn {
	pub(crate) fn id(&self) -> Id {
		match self {
			Self::Copy(r#in) => r#in.id,
			Self::Cut(r#in) => r#in.id,
			Self::Delete(r#in) => r#in.id,
			Self::Trash(r#in) => r#in.id,
			Self::Link(r#in) => r#in.id,
			Self::Hardlink(r#in) => r#in.id,
			Self::Download(r#in) => r#in.id,
			Self::Upload(r#in) => r#in.id,
		}
	}

	pub(crate) fn with_id(self, id: Id) -> Self {
		match self {
			Self::Copy(r#in) => Self::Copy(HookInOutCopy { id, ..r#in }),
			Self::Cut(r#in) => Self::Cut(HookInOutCut { id, ..r#in }),
			Self::Delete(r#in) => Self::Delete(HookInDelete { id, ..r#in }),
			Self::Trash(r#in) => Self::Trash(HookInTrash { id, ..r#in }),
			Self::Link(r#in) => Self::Link(HookInOutLink { id, ..r#in }),
			Self::Hardlink(r#in) => Self::Hardlink(HookInOutHardlink { id, ..r#in }),
			Self::Download(r#in) => Self::Download(HookInDownload { id, ..r#in }),
			Self::Upload(r#in) => Self::Upload(HookInUpload { id, ..r#in }),
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

impl HookInOutCopy {
	pub(crate) fn new<U>(from: U, to: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, from: from.into(), to: to.into() }
	}

	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileCopy(_) = &task.prog {
			task.hook = Some(HookIn::from(self).with_id(task.id));
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

impl HookInOutCut {
	pub(crate) fn new<U>(from: U, to: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, from: from.into(), to: to.into() }
	}

	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileCut(_) = &task.prog {
			task.hook = Some(HookIn::from(self).with_id(task.id));
		}
	}
}

// --- Delete
#[derive(Debug)]
pub(crate) struct HookInDelete {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
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

impl HookInOutLink {
	pub(crate) fn new<U>(from: U, to: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, from: from.into(), to: to.into() }
	}

	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileLink(_) = &task.prog {
			task.hook = Some(HookIn::from(self).with_id(task.id));
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

impl HookInOutHardlink {
	pub(crate) fn new<U>(from: U, to: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, from: from.into(), to: to.into() }
	}

	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileHardlink(_) = &task.prog {
			task.hook = Some(HookIn::from(self).with_id(task.id));
		}
	}
}

// --- Download
#[derive(Debug)]
pub(crate) struct HookInDownload {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
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

impl HookInUpload {
	pub(crate) fn new<U>(target: U) -> Self
	where
		U: Into<UrlBuf>,
	{
		Self { id: Id::ZERO, target: target.into() }
	}
}
