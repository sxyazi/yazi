use std::process::Stdio;

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command, sync::mpsc::{self, UnboundedReceiver}};
use yazi_shared::fs::{File, Url};

pub struct FdOpt {
	pub cwd:     Url,
	pub hidden:  bool,
	pub glob:    bool,
	pub subject: String,
}

pub fn fd(opt: FdOpt) -> Result<UnboundedReceiver<File>> {
	let mut child = Command::new("fd")
		.arg("--base-directory")
		.arg(&opt.cwd)
		.args(if opt.hidden { ["--hidden", "--no-ignore"] } else { ["--no-hidden", "--ignore"] })
		.arg(if opt.glob { "--glob" } else { "--regex" })
		.arg(&opt.subject)
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()?;

	drop(child.stderr.take());

	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();
	let (tx, rx) = mpsc::unbounded_channel();

	tokio::spawn(async move {
		while let Ok(Some(line)) = it.next_line().await {
			if let Ok(file) = File::from(opt.cwd.join(line)).await {
				tx.send(file).ok();
			}
		}
		child.wait().await.ok();
	});
	Ok(rx)
}
