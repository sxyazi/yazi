use std::path::Path;
use std::process::Stdio;

use anyhow::Result;
use tokio::{
	io::{AsyncBufReadExt, BufReader},
	process::{Child, Command},
	sync::mpsc::{self, UnboundedReceiver},
};
use yazi_fs::File;
use yazi_shared::url::Url;

pub struct EsOpt {
	pub cwd: Url,
	pub hidden: bool,
	pub subject: String,
	pub args: Vec<String>,
}

pub fn es(opt: EsOpt) -> Result<UnboundedReceiver<File>> {
	let mut child = spawn("es", &opt)?;

	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();
	let (tx, rx) = mpsc::unbounded_channel();

	tokio::spawn(async move {
		while let Ok(Some(line)) = it.next_line().await {
			if let Ok(file) = File::new(Path::new(&line).into()).await {
				tx.send(file).ok();
			}
		}
		child.wait().await.ok();
	});
	Ok(rx)
}

fn spawn(program: &str, opt: &EsOpt) -> std::io::Result<Child> {
	if !cfg!(windows) {
		return Err(std::io::Error::new(
			std::io::ErrorKind::Unsupported,
			"Sorry, EveryThing Search is only avaiable for Windows Users!",
		));
	}
	let Some(path) = opt.cwd.as_path() else {
		return Err(std::io::Error::new(
			std::io::ErrorKind::InvalidInput,
			"Check if you have EveryThing CLI installed!",
		));
	};

	Command::new(program)
		.arg("-path")
		.arg(path)
		// .arg(if opt.hidden { "/ah" } else { "/a-h" })
		// .args(&opt.args)
		.arg("-regex")
		.arg(&opt.subject)
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()
}
