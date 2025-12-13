use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::mpsc;
use yazi_dds::Pump;
use yazi_proxy::TasksProxy;
use yazi_vfs::provider;

use crate::{Ongoing, TaskOp, TaskOps, file::{FileOutCut, FileOutDelete, FileOutDownload, FileOutTrash}, hook::{HookInOutBg, HookInOutBlock, HookInOutCut, HookInOutDelete, HookInOutDownload, HookInOutFetch, HookInOutOrphan, HookInOutTrash}, prework::PreworkOutFetch, process::{ProcessOutBg, ProcessOutBlock, ProcessOutOrphan}};

pub(crate) struct Hook {
	ops:     TaskOps,
	ongoing: Arc<Mutex<Ongoing>>,
}

impl Hook {
	pub(crate) fn new(ops: &mpsc::UnboundedSender<TaskOp>, ongoing: &Arc<Mutex<Ongoing>>) -> Self {
		Self { ops: ops.into(), ongoing: ongoing.clone() }
	}

	// --- File
	pub(crate) async fn cut(&self, task: HookInOutCut) {
		let intact = self.ongoing.lock().intact(task.id);
		if intact {
			provider::remove_dir_clean(&task.from).await.ok();
			Pump::push_move(&task.from, &task.to);
		}
		self.ops.out(task.id, FileOutCut::Clean);
	}

	pub(crate) async fn delete(&self, task: HookInOutDelete) {
		let intact = self.ongoing.lock().intact(task.id);
		if intact {
			provider::remove_dir_all(&task.target).await.ok();
			TasksProxy::update_succeed(&task.target);
			Pump::push_delete(&task.target);
		}
		self.ops.out(task.id, FileOutDelete::Clean);
	}

	pub(crate) async fn trash(&self, task: HookInOutTrash) {
		let intact = self.ongoing.lock().intact(task.id);
		if intact {
			TasksProxy::update_succeed(&task.target);
			Pump::push_trash(&task.target);
		}
		self.ops.out(task.id, FileOutTrash::Clean);
	}

	pub(crate) async fn download(&self, task: HookInOutDownload) {
		let intact = self.ongoing.lock().intact(task.id);
		task.done.send(intact).ok();
		self.ops.out(task.id, FileOutDownload::Clean);
	}

	// --- Process
	pub(crate) async fn block(&self, task: HookInOutBlock) {
		if let Some(tx) = task.done {
			tx.send(()).ok();
		}
		self.ops.out(task.id, ProcessOutBlock::Clean);
	}

	pub(crate) async fn orphan(&self, task: HookInOutOrphan) {
		if let Some(tx) = task.done {
			tx.send(()).ok();
		}
		self.ops.out(task.id, ProcessOutOrphan::Clean);
	}

	pub(crate) async fn bg(&self, task: HookInOutBg) {
		let intact = self.ongoing.lock().intact(task.id);
		if !intact {
			task.cancel.send(()).await.ok();
			task.cancel.closed().await;
		}
		if let Some(tx) = task.done {
			tx.send(()).ok();
		}
		self.ops.out(task.id, ProcessOutBg::Clean);
	}

	// --- Prework
	pub(crate) async fn fetch(&self, task: HookInOutFetch) {
		let intact = self.ongoing.lock().intact(task.id);
		task.done.send(intact).ok();
		self.ops.out(task.id, PreworkOutFetch::Clean);
	}
}
