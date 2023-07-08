use anyhow::Result;
use tokio::{process::Command, select, sync::{mpsc, oneshot, Semaphore}};
use tracing::trace;

use super::TaskOp;
use crate::emit;

pub(super) struct Process {
	rx: async_channel::Receiver<ProcessOp>,
	tx: async_channel::Sender<ProcessOp>,

	sch: mpsc::UnboundedSender<TaskOp>,

	blocker: Semaphore,
}

#[derive(Debug)]
pub(super) enum ProcessOp {
	Open(ProcessOpOpen),
}

#[derive(Debug)]
pub(super) struct ProcessOpOpen {
	pub id:     usize,
	pub cmd:    String,
	pub args:   Vec<String>,
	pub block:  bool,
	pub cancel: oneshot::Sender<()>,
}

impl Process {
	pub(super) fn new(sch: mpsc::UnboundedSender<TaskOp>) -> Self {
		let (tx, rx) = async_channel::unbounded();
		Self { tx, rx, sch, blocker: Semaphore::new(1) }
	}

	#[inline]
	pub(super) async fn recv(&self) -> Result<(usize, ProcessOp)> {
		Ok(match self.rx.recv().await? {
			ProcessOp::Open(t) => (t.id, ProcessOp::Open(t)),
		})
	}

	pub(super) async fn work(&self, task: &mut ProcessOp) -> Result<()> {
		match task {
			ProcessOp::Open(task) => {
				trace!("Open task: {:?}", task);
				if !task.block {
					select! {
						_ = task.cancel.closed() => {},
						Ok(status) = Command::new(&task.cmd).args(&task.args).kill_on_drop(true).status() => {
							trace!("{} exited with {:?}", task.cmd, status);
						}
					}
					return Ok(self.sch.send(TaskOp::Adv(task.id, 1, 0))?);
				}

				let _guard = self.blocker.acquire().await.unwrap();
				emit!(Stop(true)).await;

				match Command::new(&task.cmd).args(&task.args).kill_on_drop(true).spawn() {
					Ok(mut child) => {
						child.wait().await.ok();
					}
					Err(e) => {
						trace!("Failed to spawn {}: {}", task.cmd, e);
					}
				}

				emit!(Stop(false)).await;
				self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
			}
		}
		Ok(())
	}

	fn done(&self, id: usize) -> Result<()> { Ok(self.sch.send(TaskOp::Done(id))?) }

	pub(super) async fn open(&self, task: ProcessOpOpen) -> Result<()> {
		let id = task.id;
		self.sch.send(TaskOp::New(id, 0))?;
		self.tx.send(ProcessOp::Open(task)).await?;
		self.done(id)
	}
}
