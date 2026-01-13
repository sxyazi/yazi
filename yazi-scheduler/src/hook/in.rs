use yazi_shared::{Id, url::UrlBuf};

use crate::{Task, TaskProg, file::{FileInCopy, FileInCut}};

// --- Copy
#[derive(Debug)]
pub(crate) struct HookInOutCopy {
	pub(crate) id:   Id,
	pub(crate) from: UrlBuf,
	pub(crate) to:   UrlBuf,
}

impl From<&FileInCopy> for HookInOutCopy {
	fn from(value: &FileInCopy) -> Self {
		Self { id: value.id, from: value.from.clone(), to: value.to.clone() }
	}
}

impl HookInOutCopy {
	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileCopy(_) = &task.prog {
			task.hook = Some(self.into());
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

impl From<&FileInCut> for HookInOutCut {
	fn from(value: &FileInCut) -> Self {
		Self { id: value.id, from: value.from.clone(), to: value.to.clone() }
	}
}

impl HookInOutCut {
	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileCut(_) = &task.prog {
			task.hook = Some(self.into());
		}
	}
}

// --- Delete
#[derive(Debug)]
pub(crate) struct HookInDelete {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
}

// --- Trash
#[derive(Debug)]
pub(crate) struct HookInTrash {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
}

// --- Download
#[derive(Debug)]
pub(crate) struct HookInDownload {
	pub(crate) id: Id,
}

// --- Upload
#[derive(Debug)]
pub(crate) struct HookInUpload {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
}
