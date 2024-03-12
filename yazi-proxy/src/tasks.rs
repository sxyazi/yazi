use std::{borrow::Cow, ffi::OsString};

use tokio::sync::oneshot;
use yazi_config::open::Opener;
use yazi_shared::{emit, event::Cmd, fs::Url, Layer};

use crate::options::{OpenWithOpt, ProcessExecOpt};

pub struct TasksProxy;

impl TasksProxy {
	#[inline]
	pub fn open_with(targets: Vec<Url>, opener: Cow<'static, Opener>) {
		emit!(Call(Cmd::new("open_with").with_data(OpenWithOpt { targets, opener }), Layer::Tasks));
	}

	#[inline]
	pub async fn process_exec(args: Vec<OsString>, opener: Cow<'static, Opener>) {
		let (tx, rx) = oneshot::channel();
		emit!(Call(
			Cmd::new("process_exec").with_data(ProcessExecOpt { args, opener, done: tx }),
			Layer::Tasks
		));
		rx.await.ok();
	}
}
