use std::ffi::OsString;

use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use yazi_core::tasks::TaskOpt;
use yazi_macro::{emit, relay};
use yazi_scheduler::process::ProcessOpt;
use yazi_shared::{Id, url::{UrlBuf, UrlCow}};

pub struct TasksProxy;

impl TasksProxy {
	pub async fn spawn(opt: TaskOpt) -> Result<Id> {
		let (tx, mut rx) = mpsc::unbounded_channel();
		emit!(Call(relay!(tasks:spawn).with_any("opt", opt).with_replier(tx)));

		rx.recv().await.ok_or_else(|| anyhow!("channel closed"))??.try_into()
	}

	// TODO: remove
	pub fn open_shell_compat(opt: ProcessOpt) {
		emit!(Call(relay!(tasks:open_shell_compat).with_any("opt", opt)));
	}

	pub async fn process_exec(
		cwd: UrlBuf,
		cmd: OsString,
		args: Vec<UrlCow<'static>>,
		block: bool,
		orphan: bool,
	) {
		let (tx, mut rx) = mpsc::unbounded_channel();
		emit!(Call(
			relay!(tasks:process_open)
				.with_any("opt", ProcessOpt { cwd, cmd, args, block, orphan, spread: false })
				.with_replier(tx)
		));
		rx.recv().await;
	}
}
