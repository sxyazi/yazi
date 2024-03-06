use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, select, sync::mpsc};
use yazi_plugin::external::{self, ShellOpt};
use yazi_proxy::AppProxy;

use super::ProcessOpOpen;
use crate::{TaskProg, BLOCKER};

pub struct Process {
	prog: mpsc::UnboundedSender<TaskProg>,
}

impl Process {
	pub fn new(prog: mpsc::UnboundedSender<TaskProg>) -> Self { Self { prog } }

	pub async fn open(&self, mut task: ProcessOpOpen) -> Result<()> {
		let opt = ShellOpt::from(&mut task);
		if task.block {
			let _guard = BLOCKER.acquire().await.unwrap();
			AppProxy::stop().await;

			match external::shell(opt) {
				Ok(mut child) => {
					let status = child.wait().await?;
					if !status.success() {
						let message = match status.code() {
							Some(code) => format!("Exited with status code: {code}"),
							None => "Process terminated by signal".to_string(),
						};
						AppProxy::notify_warn("Process failed", message.as_str());
					}
					self.succ(task.id)?;
				}
				Err(e) => {
					AppProxy::notify_warn("Process failed", format!("Failed to spawn process: {e}").as_str());
					self.succ(task.id)?;
				}
			}
			return Ok(AppProxy::resume());
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
