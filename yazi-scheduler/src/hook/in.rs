use tokio::sync::{mpsc, oneshot};
use yazi_shared::{Id, url::UrlBuf};

use crate::{Task, TaskProg, file::FileInCut};

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
	pub(crate) id:   Id,
	pub(crate) done: oneshot::Sender<bool>,
}

impl HookInOutDownload {
	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::FileDownload(_) = &task.prog {
			task.hook = Some(self.into());
		}
	}
}

// --- Fetch
#[derive(Debug)]
pub(crate) struct HookInOutFetch {
	pub(crate) id:   Id,
	pub(crate) done: oneshot::Sender<bool>,
}

impl HookInOutFetch {
	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::PreworkFetch(_) = &task.prog {
			task.hook = Some(self.into());
		}
	}
}

// --- Block
#[derive(Debug)]
pub(crate) struct HookInOutBlock {
	pub(crate) id:   Id,
	pub(crate) done: Option<oneshot::Sender<()>>,
}

impl HookInOutBlock {
	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::ProcessBlock(_) = &task.prog {
			task.hook = Some(self.into());
		}
	}
}

// --- Orphan
#[derive(Debug)]
pub(crate) struct HookInOutOrphan {
	pub(crate) id:   Id,
	pub(crate) done: Option<oneshot::Sender<()>>,
}

impl HookInOutOrphan {
	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::ProcessOrphan(_) = &task.prog {
			task.hook = Some(self.into());
		}
	}
}

// --- Bg
#[derive(Debug)]
pub(crate) struct HookInOutBg {
	pub(crate) id:     Id,
	pub(crate) cancel: mpsc::Sender<()>,
	pub(crate) done:   Option<oneshot::Sender<()>>,
}

impl HookInOutBg {
	pub(crate) fn reduce(self, task: &mut Task) {
		if let TaskProg::ProcessBg(_) = &task.prog {
			task.hook = Some(self.into());
		}
	}
}
