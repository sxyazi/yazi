use anyhow::{Result, anyhow};
use tokio::{io::{AsyncBufReadExt, BufReader}, select, sync::mpsc};
use yazi_binding::Permit;
use yazi_proxy::{AppProxy, HIDER};

use super::{ProcessInBg, ProcessInBlock, ProcessInOrphan, ShellOpt};
use crate::{TaskOp, TaskOps, process::{ProcessOutBg, ProcessOutBlock, ProcessOutOrphan}};

pub(crate) struct Process {
	ops: TaskOps,
}

impl Process {
	pub(crate) fn new(ops: &mpsc::UnboundedSender<TaskOp>) -> Self { Self { ops: ops.into() } }

	pub(crate) async fn block(&self, task: ProcessInBlock) -> Result<(), ProcessOutBlock> {
		let _permit = Permit::new(HIDER.acquire().await.unwrap(), AppProxy::resume());
		AppProxy::stop().await;

		let (id, cmd) = (task.id, task.cmd.clone());
		let result = super::shell(task.into()).await;
		if let Err(e) = result {
			AppProxy::notify_warn(cmd.to_string_lossy(), format!("Failed to start process: {e}"));
			return Ok(self.ops.out(id, ProcessOutBlock::Succ));
		}

		let status = result.unwrap().wait().await?;
		if !status.success() {
			let content = match status.code() {
				Some(130) => return Ok(self.ops.out(id, ProcessOutBlock::Succ)), // Ctrl-C pressed by user
				Some(code) => format!("Process exited with status code: {code}"),
				None => "Process terminated by signal".to_string(),
			};
			AppProxy::notify_warn(cmd.to_string_lossy(), content);
		}

		Ok(self.ops.out(id, ProcessOutBlock::Succ))
	}

	pub(crate) async fn orphan(&self, task: ProcessInOrphan) -> Result<(), ProcessOutOrphan> {
		let id = task.id;

		super::shell(task.into()).await?;
		Ok(self.ops.out(id, ProcessOutOrphan::Succ))
	}

	pub(crate) async fn bg(&self, task: ProcessInBg) -> Result<(), ProcessOutBg> {
		let mut child = super::shell(ShellOpt {
			cwd:    task.cwd,
			cmd:    task.cmd,
			args:   task.args,
			piped:  true,
			orphan: false,
		})
		.await?;

		let done = task.done;
		let mut stdout = BufReader::new(child.stdout.take().unwrap()).lines();
		let mut stderr = BufReader::new(child.stderr.take().unwrap()).lines();
		loop {
			select! {
				false = done.future() => {
					child.start_kill().ok();
					break;
				}
				Ok(Some(line)) = stdout.next_line() => {
					self.ops.out(task.id, ProcessOutBg::Log(line));
				}
				Ok(Some(line)) = stderr.next_line() => {
					self.ops.out(task.id, ProcessOutBg::Log(line));
				}
				Ok(status) = child.wait() => {
					self.ops.out(task.id, ProcessOutBg::Log(match status.code() {
						Some(code) => format!("Exited with status code: {code}"),
						None => "Process terminated by signal".to_string(),
					}));
					if !status.success() {
						Err(anyhow!("Process failed"))?;
					}
					break;
				}
			}
		}

		Ok(self.ops.out(task.id, ProcessOutBg::Succ))
	}
}
