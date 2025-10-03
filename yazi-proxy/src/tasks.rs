use std::{borrow::Cow, ffi::OsStr};

use tokio::sync::oneshot;
use yazi_config::opener::OpenerRule;
use yazi_macro::{emit, relay};
use yazi_parser::{mgr::OpenWithOpt, tasks::ProcessOpenOpt};
use yazi_shared::url::{UrlBuf, UrlCow};

pub struct TasksProxy;

impl TasksProxy {
	pub fn file_open(opener: Cow<'static, OpenerRule>, cwd: UrlBuf, targets: Vec<UrlCow<'static>>) {
		emit!(Call(relay!(tasks:file_open).with_any("option", OpenWithOpt { opener, cwd, targets })));
	}

	pub async fn process_exec(
		opener: Cow<'static, OpenerRule>,
		cwd: UrlBuf,
		args: Vec<Cow<'static, OsStr>>,
	) {
		let (tx, rx) = oneshot::channel();
		emit!(Call(relay!(tasks:process_open).with_any("option", ProcessOpenOpt {
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
