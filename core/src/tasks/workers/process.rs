use std::{ffi::OsString, process::Stdio};

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command, select, sync::{mpsc, oneshot}};
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
	fn log(&self, id: usize, line: String) -> Result<()> { Ok(self.sch.send(TaskOp::Log(id, line))?) }

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

		self.sch.send(TaskOp::New(task.id, 0))?;
		let mut child = Command::new(&task.cmd)
			.args(&task.args)
			.stdout(Stdio::piped())
			.stderr(Stdio::piped())
			.kill_on_drop(true)
			.spawn()?;

		let mut stdout = BufReader::new(child.stdout.take().unwrap()).lines();
		let mut stderr = BufReader::new(child.stderr.take().unwrap()).lines();
		loop {
			select! {
				_ = task.cancel.closed() => break,
				Ok(Some(line)) = stdout.next_line() => {
					self.log(task.id, line)?;
				}
				Ok(Some(line)) = stderr.next_line() => {
					self.log(task.id, line)?;
				}
				Ok(status) = child.wait() => {
					self.log(task.id, format!("Exited with {:?}", status))?;
					break;
				}
			}
		}
		self.done(task.id)
	}
}
