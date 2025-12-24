use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::mpsc;
use yazi_dds::Pump;
use yazi_fs::ok_or_not_found;
use yazi_proxy::TasksProxy;
use yazi_vfs::provider;

use crate::{Ongoing, TaskOp, TaskOps, file::{FileOutCopy, FileOutCut, FileOutDelete, FileOutDownload, FileOutTrash}, hook::{HookInOutCopy, HookInOutCut, HookInOutDelete, HookInOutDownload, HookInOutTrash}};

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

	pub(crate) async fn delete(&self, task: HookInOutDelete) {
		if !self.ongoing.lock().intact(task.id) {
			return self.ops.out(task.id, FileOutDelete::Clean(Ok(())));
		}

		let result = ok_or_not_found(provider::remove_dir_all(&task.target).await);
		TasksProxy::update_succeed([&task.target]);
		Pump::push_delete(task.target);

		self.ops.out(task.id, FileOutDelete::Clean(result));
	}

	pub(crate) async fn trash(&self, task: HookInOutTrash) {
		let intact = self.ongoing.lock().intact(task.id);
		if intact {
			TasksProxy::update_succeed([&task.target]);
			Pump::push_trash(task.target);
		}
		self.ops.out(task.id, FileOutTrash::Clean);
	}

	pub(crate) async fn download(&self, task: HookInOutDownload) {
		self.ops.out(task.id, FileOutDownload::Clean);
	}
}
