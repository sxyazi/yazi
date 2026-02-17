use std::process::Stdio;

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command, sync::mpsc::{self, UnboundedReceiver}};
use yazi_fs::{File};
use yazi_shared::url::{UrlBuf, UrlLike};
use yazi_vfs::VfsFile;

pub struct RgOpt {
	pub cwd:     UrlBuf,
	pub hidden:  bool,
	pub subject: String,
	pub args:    Vec<String>,
}

fn parse_rg_line_column(line: &str) -> Option<(&str, usize, usize)> {
    let mut parts = line.split(':');
    
    let (f, l, c) = (parts.next()?, parts.next()?, parts.next()?);
    if f.is_empty() { return None; }
    
    Some((f, l.parse().ok()?, c.parse().ok()?))
}

pub fn rg(opt: RgOpt) -> Result<UnboundedReceiver<File>> {
	let subject = opt.subject.clone();
	let cwd = opt.cwd.clone();

	let mut child = Command::new("rg")
		.args(["--color=never", "--line-number", "--no-heading", "--smart-case", "--column"])
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
			let Some((fp, l, c)) = parse_rg_line_column(&line) else { continue };

			let url = cwd.try_join(fp)
				.ok()
				.and_then(|u| u.to_search(&format!("{}#{}:{}", subject, l, c)).ok());

			let Some(url) = url else { continue };

			if let Ok(file) = File::new(url).await {
				tx.send(file).ok();
			}
		}
		child.wait().await.ok();
	});
	Ok(rx)
}
