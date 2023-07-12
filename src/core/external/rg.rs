use std::{path::PathBuf, process::Stdio, time::Duration};

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command, sync::mpsc::UnboundedReceiver};

use crate::misc::DelayedBuffer;

pub struct RgOpt {
	pub cwd:     PathBuf,
	pub hidden:  bool,
	pub subject: String,
}

pub fn rg(opt: RgOpt) -> Result<UnboundedReceiver<Vec<PathBuf>>> {
	let mut child = Command::new("rg")
		.current_dir(&opt.cwd)
		.args(&["--color=never", "--files-with-matches", "--smart-case"])
		.arg(if opt.hidden { "--hidden" } else { "--no-hidden" })
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
