use std::{ffi::OsString, mem};

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
	pub orphan: bool,
	pub cancel: oneshot::Sender<()>,
}

impl From<&mut ProcessOpOpen> for ShellOpt {
	fn from(value: &mut ProcessOpOpen) -> Self {
		Self {
			cmd:    mem::take(&mut value.cmd),
			args:   mem::take(&mut value.args),
			piped:  false,
			orphan: value.orphan,
		}
	}
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

			match external::shell(ShellOpt::from(&mut task)) {
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
		let mut child = external::shell(ShellOpt::from(&mut task).with_piped())?;

		let mut stdout = BufReader::new(child.stdout.take().unwrap()).lines();
		let mut stderr = BufReader::new(child.stderr.take().unwrap()).lines();
		loop {
			select! {
				_ = task.cancel.closed() => {
					child.start_kill().ok();
					break;
				}
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
