use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use yazi_plugin::isolate;
use yazi_shared::Id;

use super::{PluginIn, PluginInEntry};
use crate::{HIGH, TaskOp, TaskProg};

pub struct Plugin {
	r#macro: async_priority_channel::Sender<TaskOp, u8>,
	prog:    mpsc::UnboundedSender<TaskProg>,
}

impl Plugin {
	pub fn new(
		r#macro: async_priority_channel::Sender<TaskOp, u8>,
		prog: mpsc::UnboundedSender<TaskProg>,
	) -> Self {
		Self { r#macro, prog }
	}

	pub async fn work(&self, r#in: PluginIn) -> Result<()> {
		match r#in {
			PluginIn::Entry(task) => {
				isolate::entry(task.opt).await?;
			}
		}
		Ok(())
	}

	pub async fn micro(&self, task: PluginInEntry) -> Result<()> {
		self.prog.send(TaskProg::New(task.id, 0))?;

		if let Err(e) = isolate::entry(task.opt).await {
			self.fail(task.id, format!("Failed to run the plugin:\n{e}"))?;
			return Ok(());
		}

		self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
		self.succ(task.id)
	}

	pub fn r#macro(&self, task: PluginInEntry) -> Result<()> {
		let id = task.id;

		self.prog.send(TaskProg::New(id, 0))?;
		self.queue(PluginIn::Entry(task), HIGH)?;
		self.succ(id)
	}
}

impl Plugin {
	#[inline]
	fn succ(&self, id: Id) -> Result<()> { Ok(self.prog.send(TaskProg::Succ(id))?) }

	#[inline]
	fn fail(&self, id: Id, reason: String) -> Result<()> {
		Ok(self.prog.send(TaskProg::Fail(id, reason))?)
	}

	#[inline]
	fn queue(&self, r#in: impl Into<TaskOp>, priority: u8) -> Result<()> {
		self.r#macro.try_send(r#in.into(), priority).map_err(|_| anyhow!("Failed to send task"))
	}
}
