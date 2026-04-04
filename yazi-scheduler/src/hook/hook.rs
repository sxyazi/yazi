use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::mpsc;
use yazi_dds::Pump;
use yazi_fs::ok_or_not_found;
use yazi_vfs::provider;

use crate::{Ongoing, TaskOp, TaskOps, TasksProxy, file::{FileOutCopy, FileOutCut, FileOutDelete, FileOutDownload, FileOutHardlink, FileOutLink, FileOutTrash, FileOutUpload}, hook::{HookIn, HookInDelete, HookInDownload, HookInOutCopy, HookInOutCut, HookInOutHardlink, HookInOutLink, HookInPreload, HookInTrash, HookInUpload}, preload::{Preload, PreloadOut}};

pub(crate) struct Hook {
	ops:     TaskOps,
	ongoing: Arc<Mutex<Ongoing>>,
	preload: Arc<Preload>,
	tx:      async_priority_channel::Sender<HookIn, u8>,
}

impl Hook {
	pub(crate) fn new(
		ops: &mpsc::UnboundedSender<TaskOp>,
		ongoing: &Arc<Mutex<Ongoing>>,
		preload: &Arc<Preload>,
		tx: async_priority_channel::Sender<HookIn, u8>,
	) -> Self {
		Self { ops: ops.into(), ongoing: ongoing.clone(), preload: preload.clone(), tx }
	}

	// --- File
	pub(crate) async fn cut(&self, task: HookInOutCut) {
		if !self.ongoing.lock().intact(task.id) {
			return self.ops.out(task.id, FileOutCut::Clean(Ok(())));
		}

		let result = ok_or_not_found(provider::remove_dir_clean(&task.from).await);
		TasksProxy::update_succeed(task.id, [&task.to, &task.from], true);
		Pump::push_move(task.from, task.to);

		self.ops.out(task.id, FileOutCut::Clean(result));
	}

	pub(crate) async fn copy(&self, task: HookInOutCopy) {
		if self.ongoing.lock().intact(task.id) {
			TasksProxy::update_succeed(task.id, [&task.to], true);
			Pump::push_duplicate(task.from, task.to);
		}

		self.ops.out(task.id, FileOutCopy::Clean);
	}

	pub(crate) async fn delete(&self, task: HookInDelete) {
		if !self.ongoing.lock().intact(task.id) {
			return self.ops.out(task.id, FileOutDelete::Clean(Ok(())));
		}

		let result = ok_or_not_found(provider::remove_dir_all(&task.target).await);
		TasksProxy::update_succeed(task.id, [&task.target], false);
		Pump::push_delete(task.target);

		self.ops.out(task.id, FileOutDelete::Clean(result));
	}

	pub(crate) async fn trash(&self, task: HookInTrash) {
		let intact = self.ongoing.lock().intact(task.id);
		if intact {
			TasksProxy::update_succeed(task.id, [&task.target], false);
			Pump::push_trash(task.target);
		}
		self.ops.out(task.id, FileOutTrash::Clean);
	}

	pub(crate) async fn link(&self, task: HookInOutLink) {
		if self.ongoing.lock().intact(task.id) {
			TasksProxy::update_succeed(task.id, [&task.to], true);
		}

		self.ops.out(task.id, FileOutLink::Clean);
	}

	pub(crate) async fn hardlink(&self, task: HookInOutHardlink) {
		if self.ongoing.lock().intact(task.id) {
			TasksProxy::update_succeed(task.id, [&task.to], true);
		}

		self.ops.out(task.id, FileOutHardlink::Clean);
	}

	pub(crate) async fn download(&self, task: HookInDownload) {
		let intact = self.ongoing.lock().intact(task.id);
		if intact {
			TasksProxy::update_succeed(task.id, [&task.target], false);
			Pump::push_download(task.target);
		}
		self.ops.out(task.id, FileOutDownload::Clean);
	}

	pub(crate) async fn upload(&self, task: HookInUpload) {
		let intact = self.ongoing.lock().intact(task.id);
		if intact {
			TasksProxy::update_succeed(task.id, [task.target], false);
		}
		self.ops.out(task.id, FileOutUpload::Clean);
	}

	// --- Preload
	pub(crate) async fn preload(&self, task: HookInPreload) {
		if !self.ongoing.lock().intact(task.id) {
			self.preload.loaded.lock().get_mut(&task.hash).map(|x| *x &= !(1 << task.idx));
		}

		self.ops.out(task.id, PreloadOut::Clean);
	}
}

impl Hook {
	#[inline]
	pub(crate) fn submit(&self, r#in: impl Into<HookIn>, priority: u8) {
		_ = self.tx.try_send(r#in.into(), priority);
	}
}
