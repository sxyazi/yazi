use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::mpsc;
use yazi_dds::Pump;
use yazi_fs::ok_or_not_found;
use yazi_proxy::TasksProxy;
use yazi_vfs::provider;

use crate::{Ongoing, TaskOp, TaskOps, file::{FileOutCopy, FileOutCut, FileOutDelete, FileOutDownload, FileOutTrash, FileOutUpload}, hook::{HookIn, HookInDelete, HookInDownload, HookInOutCopy, HookInOutCut, HookInTrash, HookInUpload}};

pub(crate) struct Hook {
	ops:     TaskOps,
	ongoing: Arc<Mutex<Ongoing>>,
	tx:      async_priority_channel::Sender<HookIn, u8>,
}

impl Hook {
	pub(crate) fn new(
		ops: &mpsc::UnboundedSender<TaskOp>,
		ongoing: &Arc<Mutex<Ongoing>>,
		tx: async_priority_channel::Sender<HookIn, u8>,
	) -> Self {
		Self { ops: ops.into(), ongoing: ongoing.clone(), tx }
	}

	// --- File
	pub(crate) async fn cut(&self, task: HookInOutCut) {
		if !self.ongoing.lock().intact(task.id) {
			return self.ops.out(task.id, FileOutCut::Clean(Ok(())));
		}

		let result = ok_or_not_found(provider::remove_dir_clean(&task.from).await);
		TasksProxy::update_succeed([&task.to, &task.from]);
		Pump::push_move(task.from, task.to);

		self.ops.out(task.id, FileOutCut::Clean(result));
	}

	pub(crate) async fn copy(&self, task: HookInOutCopy) {
		if self.ongoing.lock().intact(task.id) {
			TasksProxy::update_succeed([&task.to]);
			Pump::push_duplicate(task.from, task.to);
		}

		self.ops.out(task.id, FileOutCopy::Clean);
	}

	pub(crate) async fn delete(&self, task: HookInDelete) {
		if !self.ongoing.lock().intact(task.id) {
			return self.ops.out(task.id, FileOutDelete::Clean(Ok(())));
		}

		let result = ok_or_not_found(provider::remove_dir_all(&task.target).await);
		TasksProxy::update_succeed([&task.target]);
		Pump::push_delete(task.target);

		self.ops.out(task.id, FileOutDelete::Clean(result));
	}

	pub(crate) async fn trash(&self, task: HookInTrash) {
		let intact = self.ongoing.lock().intact(task.id);
		if intact {
			TasksProxy::update_succeed([&task.target]);
			Pump::push_trash(task.target);
		}
		self.ops.out(task.id, FileOutTrash::Clean);
	}

	pub(crate) async fn download(&self, task: HookInDownload) {
		let intact = self.ongoing.lock().intact(task.id);
		if intact {
			TasksProxy::update_succeed([&task.target]);
			Pump::push_download(task.target);
		}
		self.ops.out(task.id, FileOutDownload::Clean);
	}

	pub(crate) async fn upload(&self, task: HookInUpload) {
		let intact = self.ongoing.lock().intact(task.id);
		if intact {
			TasksProxy::update_succeed([task.target]);
		}
		self.ops.out(task.id, FileOutUpload::Clean);
	}
}

impl Hook {
	#[inline]
	pub(crate) fn submit(&self, r#in: impl Into<HookIn>, priority: u8) {
		_ = self.tx.try_send(r#in.into(), priority);
	}
}
