use std::{path::PathBuf, process::Stdio, time::Duration};

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command, sync::mpsc::UnboundedReceiver, task::JoinHandle};

use crate::misc::DelayedBuffer;

pub struct FdOpt {
	pub cwd:     PathBuf,
	pub hidden:  bool,
	pub regex:   bool,
	pub subject: String,
}

pub fn fd(opt: FdOpt) -> Result<(JoinHandle<()>, UnboundedReceiver<Vec<PathBuf>>)> {
	let mut child = Command::new("fd")
		.arg("--base-directory")
		.arg(&opt.cwd)
		.arg(if opt.hidden { "--hidden" } else { "--no-hidden" })
		.arg(if opt.regex { "--regex" } else { "--glob" })
		.arg(&opt.subject)
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.spawn()?;

	drop(child.stderr.take());

	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();
	let (mut buf, rx) = DelayedBuffer::new(Duration::from_millis(100));

	let handle = tokio::spawn(async move {
		while let Ok(Some(line)) = it.next_line().await {
			let path = PathBuf::from(line);
			if path.components().count() > 1 {
				buf.push(path);
			}
		}
		child.wait().await.ok();
	});
	Ok((handle, rx))
}
