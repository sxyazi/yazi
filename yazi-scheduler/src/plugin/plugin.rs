use anyhow::Result;
use tokio::sync::mpsc;
use yazi_plugin::isolate;

use super::PluginInEntry;
use crate::{HIGH, TaskIn, TaskOp, TaskOps, plugin::PluginOutEntry};

pub(crate) struct Plugin {
	ops:     TaskOps,
	r#macro: async_priority_channel::Sender<TaskIn, u8>,
}

impl Plugin {
	pub(crate) fn new(
		ops: &mpsc::UnboundedSender<TaskOp>,
		r#macro: &async_priority_channel::Sender<TaskIn, u8>,
	) -> Self {
		Self { ops: ops.into(), r#macro: r#macro.clone() }
	}

	pub(crate) async fn entry(&self, task: PluginInEntry) -> Result<(), PluginOutEntry> {
		Ok(self.queue(task, HIGH))
	}

	pub(crate) async fn entry_do(&self, task: PluginInEntry) -> Result<(), PluginOutEntry> {
		isolate::entry(task.opt).await?;
		Ok(self.ops.out(task.id, PluginOutEntry::Succ))
	}
}

impl Plugin {
	#[inline]
	fn queue(&self, r#in: impl Into<TaskIn>, priority: u8) {
		_ = self.r#macro.try_send(r#in.into(), priority);
	}
}
