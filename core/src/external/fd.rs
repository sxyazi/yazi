use std::{path::PathBuf, process::Stdio, time::Duration};

use anyhow::Result;
use shared::DelayedBuffer;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command, sync::mpsc::UnboundedReceiver};

pub struct FdOpt {
	pub cwd:     PathBuf,
	pub hidden:  bool,
	pub glob:    bool,
	pub subject: String,
}

pub fn fd(opt: FdOpt) -> Result<UnboundedReceiver<Vec<PathBuf>>> {
	let mut child = Command::new("fd")
		.arg("--base-directory")
		.arg(&opt.cwd)
		.arg(if opt.hidden { "--hidden" } else { "--no-hidden" })
		.arg(if opt.glob { "--glob" } else { "--regex" })
		.arg(&opt.subject)
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	drop(child.stderr.take());

	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();
	let (mut buf, rx) = DelayedBuffer::new(Duration::from_millis(100));

	tokio::spawn(async move {
		while let Ok(Some(line)) = it.next_line().await {
			buf.push(opt.cwd.join(line));
		}
		child.wait().await.ok();
	});
	Ok(rx)
}
