use std::process::Stdio;

use anyhow::Result;
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::Command,
	sync::mpsc::{self, UnboundedReceiver},
};
use yazi_shared::fs::{File, Url};

pub struct FdOpt {
	pub cwd: Url,
	pub hidden: bool,
	pub subject: String,
	pub args: Vec<String>,
}

pub async fn fd(opt: FdOpt) -> Result<UnboundedReceiver<File>> {
	let cmd_name = if Command::new("fd").arg("-V").status().await.is_ok() { "fd" } else { "fdfind" };

	let mut child = Command::new(cmd_name)
		.arg("--base-directory")
		.arg(&opt.cwd)
		.arg("--regex")
		.arg(if opt.hidden { "--hidden" } else { "--no-hidden" })
		.args(opt.args)
		.arg(opt.subject)
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()?;

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
