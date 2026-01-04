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
		let done = CompletionToken::default();
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

	pub fn update_succeed<I>(url: I)
	where
		I: IntoIterator,
		I::Item: Into<UrlBuf>,
	{
		let urls: Vec<_> = url.into_iter().map(Into::into).collect();
		emit!(Call(relay!(tasks:update_succeed).with_any("urls", urls)));
	}
}
