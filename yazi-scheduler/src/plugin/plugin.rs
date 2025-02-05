use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use yazi_plugin::isolate;

use super::{PluginOp, PluginOpEntry};
use crate::{HIGH, TaskOp, TaskProg};

pub struct Plugin {
	macro_: async_priority_channel::Sender<TaskOp, u8>,
	prog:   mpsc::UnboundedSender<TaskProg>,
}

impl Plugin {
	pub fn new(
		macro_: async_priority_channel::Sender<TaskOp, u8>,
		prog: mpsc::UnboundedSender<TaskProg>,
	) -> Self {
		Self { macro_, prog }
	}

	pub async fn work(&self, op: PluginOp) -> Result<()> {
		match op {
			PluginOp::Entry(task) => {
				isolate::entry(task.opt).await?;
			}
		}
		Ok(())
	}

	pub async fn micro(&self, task: PluginOpEntry) -> Result<()> {
		self.prog.send(TaskProg::New(task.id, 0))?;

		if let Err(e) = isolate::entry(task.opt).await {
			self.fail(task.id, format!("Failed to run the plugin:\n{e}"))?;
			return Ok(());
		}

		self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
		self.succ(task.id)
	}

	pub fn macro_(&self, task: PluginOpEntry) -> Result<()> {
		let id = task.id;

		self.prog.send(TaskProg::New(id, 0))?;
		self.queue(PluginOp::Entry(task), HIGH)?;
		self.succ(id)
	}
}

impl Plugin {
	#[inline]
	fn succ(&self, id: usize) -> Result<()> { Ok(self.prog.send(TaskProg::Succ(id))?) }

	#[inline]
	fn fail(&self, id: usize, reason: String) -> Result<()> {
		Ok(self.prog.send(TaskProg::Fail(id, reason))?)
	}

	#[inline]
	fn queue(&self, op: impl Into<TaskOp>, priority: u8) -> Result<()> {
		self.macro_.try_send(op.into(), priority).map_err(|_| anyhow!("Failed to send task"))
	}
}
