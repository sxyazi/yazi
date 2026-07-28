use std::{borrow::Cow, path::Path, process::Stdio};

use anyhow::Result;
use tokio::{io::{AsyncBufReadExt, BufReader}, process::{Child, Command}, sync::mpsc::{self, UnboundedReceiver}};
use yazi_fs::{FsUrl, Normalizer, file::File};
use yazi_shared::url::{AsUrl, UrlBuf, UrlLike};
use yazi_vfs::engine;

pub struct FdOpt {
	pub cwd:     UrlBuf,
	pub hidden:  bool,
	pub subject: String,
	pub args:    Vec<String>,
}

pub fn fd(mut opt: FdOpt) -> Result<UnboundedReceiver<File>> {
	if !regex_disabled(&opt.args)
		&& let Ok(Cow::Owned(normalized)) = Normalizer::normalize(&opt.subject)
	{
		opt.subject = normalized;
	}

	let mut child = spawn("fd", &opt).or_else(|_| spawn("fdfind", &opt))?;
	let mut it = BufReader::new(child.stdout.take().unwrap()).lines();

	let (tx, rx) = mpsc::unbounded_channel();
	tokio::spawn(async move {
		while let Ok(Some(line)) = it.next_line().await {
			if Path::new(&line).is_absolute() {
				continue;
			}
			let Ok(url) = opt.cwd.try_join(line) else {
				continue;
			};
			if let Ok(file) = engine::file(url).await {
				tx.send(file).ok();
			}
		}
		child.wait().await.ok();
	});
	Ok(rx)
}

fn spawn(program: &str, opt: &FdOpt) -> std::io::Result<Child> {
	Command::new(program)
		.arg("--base-directory")
		.arg(&*opt.cwd.as_url().working_path())
		.arg("--regex")
		.arg(if opt.hidden { "--hidden" } else { "--no-hidden" })
		.args(&opt.args)
		.arg(&opt.subject)
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.spawn()
}

fn regex_disabled(args: &[String]) -> bool {
	let mut glob = false;
	for arg in args {
		match arg.as_str() {
			"--fixed-strings" => return true,
			"--glob" => glob = true,
			"--regex" => glob = false,
			_ if arg.starts_with("--") => {}
			_ if let Some(flags) = arg.strip_prefix('-') => {
				if flags.contains('F') {
					return true;
				}
				glob |= flags.contains('g');
			}
			_ => {}
		}
	}
	glob
}
