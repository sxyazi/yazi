use std::ffi::OsString;

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, select, sync::{mpsc, oneshot}};
use tracing::trace;

use crate::{emit, external::{self, ShellOpt}, tasks::TaskOp, BLOCKER};

pub(crate) struct Process {
	sch: mpsc::UnboundedSender<TaskOp>,
}

#[derive(Debug)]
pub(crate) struct ProcessOpOpen {
	pub id:     usize,
	pub cmd:    OsString,
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

			match external::shell(ShellOpt { cmd: task.cmd, args: task.args, piped: false }) {
				Ok(mut child) => {
					child.wait().await.ok();
				}
				Err(e) => {
					trace!("Failed to spawn process: {e}");
				}
			}
			emit!(Stop(false)).await;

			self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
			return self.done(task.id);
		}

		self.sch.send(TaskOp::New(task.id, 0))?;
		let mut child = external::shell(ShellOpt { cmd: task.cmd, args: task.args, piped: true })?;

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
					self.log(task.id, match status.code() {
						Some(code) => format!("Exited with status code: {code}"),
						None => "Process terminated by signal".to_string(),
					})?;
					if !status.success() {
						return Ok(());
					}
					break;
				}
			}
		}

		self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
		self.done(task.id)
	}
}
