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
pub(crate) struct HookInOutDelete {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
}

impl HookInOutDelete {
	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileDelete(_) = &task.prog {
			task.hook = Some(self.into());
		}
	}
}

// --- Trash
#[derive(Debug)]
pub(crate) struct HookInOutTrash {
	pub(crate) id:     Id,
	pub(crate) target: UrlBuf,
}

impl HookInOutTrash {
	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileTrash(_) = &task.prog {
			task.hook = Some(self.into());
		}
	}
}

// --- Download
#[derive(Debug)]
pub(crate) struct HookInOutDownload {
	pub(crate) id: Id,
}

impl HookInOutDownload {
	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileDownload(_) = &task.prog {
			task.hook = Some(self.into());
		}
	}
}
