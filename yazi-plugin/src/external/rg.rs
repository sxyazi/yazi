use std::process::Stdio;

use anyhow::{Result, bail};
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command, sync::mpsc::{self, UnboundedReceiver}};
use yazi_fs::File;
use yazi_shared::url::Url;

pub struct RgOpt {
	pub cwd:     Url,
	pub hidden:  bool,
	pub subject: String,
	pub args:    Vec<String>,
}

pub fn rg(opt: RgOpt) -> Result<UnboundedReceiver<File>> {
	let Some(path) = opt.cwd.as_path() else {
		bail!("rg can only search local filesystem");
	};

	let mut child = Command::new("rg")
		.args(["--color=never", "--files-with-matches", "--smart-case"])
		.arg(if opt.hidden { "--hidden" } else { "--no-hidden" })
		.args(opt.args)
		.arg(opt.subject)
		.arg(path)
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()?;

	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();
	let (tx, rx) = mpsc::unbounded_channel();

	tokio::spawn(async move {
		while let Ok(Some(line)) = it.next_line().await {
			if let Ok(file) = File::new(opt.cwd.join(line)).await {
				tx.send(file).ok();
			}
		}
		child.wait().await.ok();
	});
	Ok(rx)
}
