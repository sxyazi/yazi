use anyhow::Result;
use scopeguard::defer;
use tokio::{io::{AsyncBufReadExt, BufReader}, select, sync::mpsc};
use yazi_proxy::{AppProxy, HIDER};
use yazi_shared::Id;

use super::{ProcessInBg, ProcessInBlock, ProcessInOrphan, ShellOpt};
use crate::TaskProg;

pub struct Process {
	prog: mpsc::UnboundedSender<TaskProg>,
}

impl Process {
	pub fn new(prog: mpsc::UnboundedSender<TaskProg>) -> Self { Self { prog } }

	pub async fn block(&self, task: ProcessInBlock) -> Result<()> {
		let _permit = HIDER.acquire().await.unwrap();
		defer!(AppProxy::resume());
		AppProxy::stop().await;

		let (id, cmd) = (task.id, task.cmd.clone());
		let result = super::shell(task.into());
		if let Err(e) = result {
			AppProxy::notify_warn(&cmd.to_string_lossy(), format!("Failed to start process: {e}"));
			return self.succ(id);
		}

		let status = result.unwrap().wait().await?;
		if !status.success() {
			let content = match status.code() {
				Some(130) => return self.succ(id), // Ctrl-C pressed by user
				Some(code) => format!("Process exited with status code: {code}"),
				None => "Process terminated by signal".to_string(),
			};
			AppProxy::notify_warn(&cmd.to_string_lossy(), &content);
		}

		self.succ(id)
	}

	pub async fn orphan(&self, task: ProcessInOrphan) -> Result<()> {
		let id = task.id;
		match super::shell(task.into()) {
			Ok(_) => self.succ(id)?,
			Err(e) => {
				self.prog.send(TaskProg::New(id, 0))?;
				self.fail(id, format!("Failed to start process: {e}"))?;
			}
		}

		Ok(())
	}

	pub async fn bg(&self, task: ProcessInBg) -> Result<()> {
		self.prog.send(TaskProg::New(task.id, 0))?;
		let mut child = super::shell(ShellOpt {
			cwd:    task.cwd,
			cmd:    task.cmd,
			args:   task.args,
			piped:  true,
			orphan: false,
		})?;

		let mut stdout = BufReader::new(child.stdout.take().unwrap()).lines();
		let mut stderr = BufReader::new(child.stderr.take().unwrap()).lines();
		let mut cancel = task.cancel;
		loop {
			select! {
				_ = cancel.recv() => {
					child.start_kill().ok();
					cancel.close();
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
	fn succ(&self, id: Id) -> Result<()> { Ok(self.prog.send(TaskProg::Succ(id))?) }

	#[inline]
	fn fail(&self, id: Id, reason: String) -> Result<()> {
		Ok(self.prog.send(TaskProg::Fail(id, reason))?)
	}

	#[inline]
	fn log(&self, id: Id, line: String) -> Result<()> { Ok(self.prog.send(TaskProg::Log(id, line))?) }
}
