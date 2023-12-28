use std::{ffi::OsString, mem};

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, select, sync::{mpsc, oneshot}};
use yazi_plugin::external::{self, ShellOpt};

use crate::{Scheduler, TaskProg, BLOCKER};

pub struct Process {
	prog: mpsc::UnboundedSender<TaskProg>,
}

#[derive(Debug)]
pub enum ProcessOp {
	Open(ProcessOpOpen),
}

#[derive(Debug)]
pub struct ProcessOpOpen {
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
	pub fn new(prog: mpsc::UnboundedSender<TaskProg>) -> Self { Self { prog } }

	pub async fn open(&self, mut task: ProcessOpOpen) -> Result<()> {
		let opt = ShellOpt::from(&mut task);
		if task.block {
			let _guard = BLOCKER.acquire().await.unwrap();
			Scheduler::app_stop().await;

			match external::shell(opt) {
				Ok(mut child) => {
					child.wait().await.ok();
					self.succ(task.id)?;
				}
				Err(e) => {
					self.prog.send(TaskProg::New(task.id, 0))?;
					self.fail(task.id, format!("Failed to spawn process: {e}"))?;
				}
			}
			return Ok(Scheduler::app_resume());
		}

		if task.orphan {
			match external::shell(opt) {
				Ok(_) => self.succ(task.id)?,
				Err(e) => {
					self.prog.send(TaskProg::New(task.id, 0))?;
					self.fail(task.id, format!("Failed to spawn process: {e}"))?;
				}
			}
			return Ok(());
		}

		self.prog.send(TaskProg::New(task.id, 0))?;
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

		self.prog.send(TaskProg::Adv(task.id, 1, 0))?;
		self.succ(task.id)
	}
}

impl Process {
	#[inline]
	fn succ(&self, id: usize) -> Result<()> { Ok(self.prog.send(TaskProg::Succ(id))?) }

	#[inline]
	fn fail(&self, id: usize, reason: String) -> Result<()> {
		Ok(self.prog.send(TaskProg::Fail(id, reason))?)
	}

	#[inline]
	fn log(&self, id: usize, line: String) -> Result<()> {
		Ok(self.prog.send(TaskProg::Log(id, line))?)
	}
}
