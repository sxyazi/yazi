use std::process::Stdio;

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::{Child, Command}, sync::mpsc::{self, UnboundedReceiver}};
use yazi_fs::File;
use yazi_shared::url::Url;

pub struct FdOpt {
	pub cwd:     Url,
	pub hidden:  bool,
	pub subject: String,
	pub args:    Vec<String>,
}

pub fn fd(opt: FdOpt) -> Result<UnboundedReceiver<File>> {
	let mut child = spawn("fd", &opt).or_else(|_| spawn("fdfind", &opt))?;

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

fn spawn(program: &str, opt: &FdOpt) -> std::io::Result<Child> {
	let Some(path) = opt.cwd.as_path() else {
		return Err(std::io::Error::new(
			std::io::ErrorKind::InvalidInput,
			"fd can only search local filesystem",
		));
	};

	Command::new(program)
		.arg("--base-directory")
		.arg(path)
		.arg("--regex")
		.arg(if opt.hidden { "--hidden" } else { "--no-hidden" })
		.args(&opt.args)
		.arg(&opt.subject)
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()
}
