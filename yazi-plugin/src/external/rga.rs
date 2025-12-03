use std::process::Stdio;

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command, sync::mpsc::{self, UnboundedReceiver}};
use yazi_fs::{File, FsUrl};
use yazi_shared::url::{AsUrl, UrlBuf, UrlLike};
use yazi_vfs::VfsFile;

pub struct RgaOpt {
	pub cwd:     UrlBuf,
	pub hidden:  bool,
	pub subject: String,
	pub args:    Vec<String>,
}

pub fn rga(opt: RgaOpt) -> Result<UnboundedReceiver<File>> {
	let mut child = Command::new("rga")
		.args(["--color=never", "--files-with-matches", "--smart-case"])
		.arg(if opt.hidden { "--hidden" } else { "--no-hidden" })
		.args(opt.args)
		.arg(opt.subject)
		.arg(&*opt.cwd.as_url().unified_path())
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()?;

	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();
	let (tx, rx) = mpsc::unbounded_channel();

	tokio::spawn(async move {
		while let Ok(Some(line)) = it.next_line().await {
			let Ok(url) = opt.cwd.try_join(line) else {
				continue;
			};
			if let Ok(file) = File::new(url).await {
				tx.send(file).ok();
			}
		}
		child.wait().await.ok();
	});
	Ok(rx)
}
