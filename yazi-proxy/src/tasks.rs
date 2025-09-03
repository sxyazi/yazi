use std::{borrow::Cow, ffi::OsString};

use tokio::sync::oneshot;
use yazi_config::opener::OpenerRule;
use yazi_macro::{emit, relay};
use yazi_parser::{mgr::OpenWithOpt, tasks::ProcessExecOpt};
use yazi_shared::url::UrlBuf;

pub struct TasksProxy;

impl TasksProxy {
	pub fn open_with(opener: Cow<'static, OpenerRule>, cwd: UrlBuf, targets: Vec<UrlBuf>) {
		emit!(Call(relay!(tasks:open_with).with_any("option", OpenWithOpt { opener, cwd, targets })));
	}

	pub async fn process_exec(opener: Cow<'static, OpenerRule>, cwd: UrlBuf, args: Vec<OsString>) {
		let (tx, rx) = oneshot::channel();
		emit!(Call(relay!(tasks:process_exec).with_any("option", ProcessExecOpt {
			cwd,
			opener,
			args,
			done: Some(tx)
		})));
		rx.await.ok();
	}

	pub fn update_succeed(url: impl Into<UrlBuf>) {
		emit!(Call(relay!(tasks:update_succeed).with_any("urls", vec![url.into()])));
	}
}
