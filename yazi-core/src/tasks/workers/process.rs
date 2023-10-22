use std::{ffi::OsString, mem};

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, select, sync::{mpsc, oneshot}};

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

	pub(crate) async fn open(&self, mut task: ProcessOpOpen) -> Result<()> {
		let opt = ShellOpt::from(&mut task);
		if task.block {
			let _guard = BLOCKER.acquire().await.unwrap();
			emit!(Stop(true)).await;

			match external::shell(opt) {
				Ok(mut child) => {
					child.wait().await.ok();
					self.succ(task.id)?;
				}
				Err(e) => {
					self.sch.send(TaskOp::New(task.id, 0))?;
					self.fail(task.id, format!("Failed to spawn process: {e}"))?;
				}
			}
			return Ok(emit!(Stop(false)).await);
		}

		if task.orphan {
			match external::shell(opt) {
				Ok(_) => self.succ(task.id)?,
				Err(e) => {
					self.sch.send(TaskOp::New(task.id, 0))?;
					self.fail(task.id, format!("Failed to spawn process: {e}"))?;
				}
			}
			return Ok(());
		}

		self.sch.send(TaskOp::New(task.id, 0))?;
		let mut child = external::shell(opt.with_piped())?;

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
						return self.fail(task.id, "Process failed".to_string());
					}
					break;
				}
			}
		}

		self.sch.send(TaskOp::Adv(task.id, 1, 0))?;
		self.succ(task.id)
	}
}

impl Process {
	#[inline]
	fn succ(&self, id: usize) -> Result<()> { Ok(self.sch.send(TaskOp::Succ(id))?) }

	#[inline]
	fn fail(&self, id: usize, reason: String) -> Result<()> {
		Ok(self.sch.send(TaskOp::Fail(id, reason))?)
	}

	#[inline]
	fn log(&self, id: usize, line: String) -> Result<()> { Ok(self.sch.send(TaskOp::Log(id, line))?) }
}
