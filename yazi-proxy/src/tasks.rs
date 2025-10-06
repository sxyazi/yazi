use std::ffi::OsString;

use tokio::sync::oneshot;
use yazi_macro::{emit, relay};
use yazi_parser::tasks::ProcessOpenOpt;
use yazi_shared::url::{UrlBuf, UrlCow};

pub struct TasksProxy;

impl TasksProxy {
	// TODO: remove
	pub fn open_shell_compat(opt: ProcessOpenOpt) {
		emit!(Call(relay!(tasks:open_shell_compat).with_any("opt", opt)));
	}

	pub async fn process_exec(
		cwd: UrlBuf,
		cmd: OsString,
		args: Vec<UrlCow<'static>>,
		block: bool,
		orphan: bool,
	) {
		let (tx, rx) = oneshot::channel();
		emit!(Call(relay!(tasks:process_open).with_any("opt", ProcessOpenOpt {
			cwd,
			cmd,
			args,
			block,
			orphan,
			done: Some(tx),
			spread: false
		})));
		rx.await.ok();
	}

	pub fn update_succeed(url: impl Into<UrlBuf>) {
		emit!(Call(relay!(tasks:update_succeed).with_any("urls", vec![url.into()])));
	}
}
