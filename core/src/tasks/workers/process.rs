use std::{ffi::OsString, process::Stdio};

use anyhow::Result;
use tokio::{process::Command, select, sync::{mpsc, oneshot}};
use tracing::trace;

use crate::{emit, tasks::TaskOp, BLOCKER};

pub(crate) struct Process {
	sch: mpsc::UnboundedSender<TaskOp>,
}

#[derive(Debug)]
pub(crate) struct ProcessOpOpen {
	pub id:     usize,
	pub cmd:    String,
	pub args:   Vec<OsString>,
	pub block:  bool,
	pub cancel: oneshot::Sender<()>,
}

impl Process {
	pub(crate) fn new(sch: mpsc::UnboundedSender<TaskOp>) -> Self { Self { sch } }

	#[inline]
	fn done(&self, id: usize) -> Result<()> { Ok(self.sch.send(TaskOp::Done(id))?) }

	pub(crate) async fn open(&self, mut task: ProcessOpOpen) -> Result<()> {
		if task.block {
			let _guard = BLOCKER.acquire().await.unwrap();
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
			return Ok(());
		}

		let status = Command::new(&task.cmd)
			.args(&task.args)
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.kill_on_drop(true)
			.status();

		self.sch.send(TaskOp::New(task.id, 0))?;
		select! {
			_ = task.cancel.closed() => {},
			Ok(status) = status => {
				trace!("{} exited with {:?}", task.cmd, status);
			}
		}
		self.done(task.id)
	}
}
