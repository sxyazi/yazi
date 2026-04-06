use anyhow::Result;
use tokio::sync::mpsc;
use yazi_runner::RUNNER;

use crate::{TaskOp, TaskOps, plugin::{PluginIn, PluginInEntry, PluginOutEntry}};

pub(crate) struct Plugin {
	ops: TaskOps,
	tx:  async_priority_channel::Sender<PluginIn, u8>,
}

impl Plugin {
	pub(crate) fn new(
		ops: &mpsc::UnboundedSender<TaskOp>,
		tx: async_priority_channel::Sender<PluginIn, u8>,
	) -> Self {
		Self { ops: ops.into(), tx }
	}

	pub(crate) async fn entry(&self, task: PluginInEntry) -> Result<(), PluginOutEntry> {
		let id = task.id;

		RUNNER.entry(task.into_job()).await?;
		Ok(self.ops.out(id, PluginOutEntry::Succ))
	}
}

impl Plugin {
	#[inline]
	pub(crate) fn submit(&self, r#in: impl Into<PluginIn>, priority: u8) {
		_ = self.tx.try_send(r#in.into(), priority);
	}
}
