use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use yazi_core::tasks::TaskOpt;
use yazi_macro::{emit, relay};
use yazi_scheduler::{TaskHandle, custom::CustomOut, process::ShellOpt};
use yazi_shared::id::Id;

pub struct TasksProxy;

impl TasksProxy {
	pub async fn spawn(opt: TaskOpt) -> Result<TaskHandle> {
		let (tx, mut rx) = mpsc::unbounded_channel();
		emit!(Call(relay!(tasks:spawn).with_any("opt", opt).with_replier(tx)));

		rx.recv().await.ok_or_else(|| anyhow!("channel closed"))??.into_any2()
	}

	pub fn output(id: Id, out: CustomOut) {
		emit!(Call(relay!(tasks:output).with("id", id).with_any("out", out)));
	}

	pub fn process_open(opt: ShellOpt) {
		emit!(Call(relay!(tasks:process_open).with_any("opt", opt)));
	}

	pub async fn process_exec(opt: ShellOpt) {
		let (tx, mut rx) = mpsc::unbounded_channel();
		emit!(Call(relay!(tasks:process_open).with_any("opt", opt).with_replier(tx)));
		rx.recv().await;
	}
}
