use anyhow::Result;
use tokio::sync::mpsc;

use crate::TaskOp;

pub struct Plugin {
	tx: async_channel::Sender<PluginOp>,
	rx: async_channel::Receiver<PluginOp>,

	sch: mpsc::UnboundedSender<TaskOp>,
}

#[derive(Debug)]
pub enum PluginOp {
	Entry(PluginOpEntry),
}

#[derive(Clone, Debug)]
pub struct PluginOpEntry {
	pub id:   usize,
	pub name: String,
}

impl Plugin {
	pub fn new(sch: mpsc::UnboundedSender<TaskOp>) -> Self {
		let (tx, rx) = async_channel::unbounded();
		Self { tx, rx, sch }
	}

	#[inline]
	pub async fn recv(&self) -> Result<(usize, PluginOp)> {
		Ok(match self.rx.recv().await? {
			PluginOp::Entry(t) => (t.id, PluginOp::Entry(t)),
		})
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
		self.sch.send(TaskOp::New(task.id, 0))?;

		if let Err(e) = yazi_plugin::isolate::entry(&task.name).await {
			self.fail(task.id, format!("Micro plugin failed:\n{e}"))?;
			return Err(e.into());
		}

		self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
		self.succ(task.id)
	}

	pub fn macro_(&self, task: PluginOpEntry) -> Result<()> {
		let id = task.id;

		self.sch.send(TaskOp::New(id, 0))?;
		self.tx.send_blocking(PluginOp::Entry(task))?;
		self.succ(id)
	}
}

impl Plugin {
	#[inline]
	fn succ(&self, id: usize) -> Result<()> { Ok(self.sch.send(TaskOp::Succ(id))?) }

	#[inline]
	fn fail(&self, id: usize, reason: String) -> Result<()> {
		Ok(self.sch.send(TaskOp::Fail(id, reason))?)
	}
}
