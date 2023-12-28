use anyhow::Result;
use tokio::sync::mpsc;

use super::{PluginOp, PluginOpEntry};
use crate::{TaskOp, TaskProg};

pub struct Plugin {
	macro_: async_channel::Sender<TaskOp>,
	prog:   mpsc::UnboundedSender<TaskProg>,
}

impl Plugin {
	pub fn new(macro_: async_channel::Sender<TaskOp>, prog: mpsc::UnboundedSender<TaskProg>) -> Self {
		Self { macro_, prog }
	}

	pub async fn work(&self, op: &mut PluginOp) -> Result<()> {
		match op {
			PluginOp::Entry(task) => {
				yazi_plugin::isolate::entry(&task.name).await?;
			}
		}
		Ok(())
	}

	pub async fn micro(&self, task: PluginOpEntry) -> Result<()> {
		self.prog.send(TaskProg::New(task.id, 0))?;

		if let Err(e) = yazi_plugin::isolate::entry(&task.name).await {
			self.fail(task.id, format!("Micro plugin failed:\n{e}"))?;
			return Err(e.into());
		}

		self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
		self.succ(task.id)
	}

	pub fn macro_(&self, task: PluginOpEntry) -> Result<()> {
		let id = task.id;

		self.prog.send(TaskProg::New(id, 0))?;
		self.macro_.send_blocking(PluginOp::Entry(task).into())?;
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
}
