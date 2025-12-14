use std::ffi::OsString;

use yazi_macro::{emit, relay};
use yazi_parser::tasks::ProcessOpenOpt;
use yazi_shared::{CompletionToken, url::{UrlBuf, UrlCow}};

pub struct TasksProxy;

impl TasksProxy {
	// TODO: remove
	pub fn open_shell_compat(opt: ProcessOpenOpt) {
		emit!(Call(relay!(tasks:open_shell_compat).with_any("opt", opt)));
	}

	pub async fn process_exec(
		cwd: UrlCow<'static>,
		cmd: OsString,
		args: Vec<UrlCow<'static>>,
		block: bool,
		orphan: bool,
	) {
		let done = CompletionToken::new();
		emit!(Call(relay!(tasks:process_open).with_any("opt", ProcessOpenOpt {
			cwd,
			cmd,
			args,
			block,
			orphan,
			done: Some(done.clone()),
			spread: false
		})));
		done.future().await;
	}

	pub fn update_succeed(url: impl Into<UrlBuf>) {
		emit!(Call(relay!(tasks:update_succeed).with_any("urls", vec![url.into()])));
	}
}
