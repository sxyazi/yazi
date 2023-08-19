use std::{ffi::OsStr, os::unix::prelude::OsStrExt, process::Stdio};

use anyhow::{bail, Result};
use tokio::{io::AsyncWriteExt, process::Command};
use tracing::info;

pub async fn clipboard_get() -> Result<String> {
	let all = [
		("pbpaste", vec![]),
		("wl-paste", vec![]),
		("xclip", vec!["-o", "-selection", "clipboard"]),
		("xsel", vec!["-ob"]),
	];

	for (cmd, args) in all {
		let output = Command::new(cmd).args(args).kill_on_drop(true).output().await?;
		if output.status.success() {
			return Ok(String::from_utf8_lossy(&output.stdout).to_string());
		}
	}

	bail!("failed to get clipboard")
}

pub async fn clipboard_set(s: impl AsRef<OsStr>) -> Result<()> {
	info!("clipboard_set: {:?}", s.as_ref());

	let all = [
		("pbcopy", vec![]),
		("wl-copy", vec![]),
		("xclip", vec!["-selection", "clipboard"]),
		("xsel", vec!["-ib"]),
	];

	for (cmd, args) in all {
		info!("clipboard_set: trying {:?} {:?}", cmd, args);
		let child = Command::new(cmd)
			.args(args)
			.stdin(Stdio::piped())
			.stdout(Stdio::null())
			.kill_on_drop(true)
			.spawn();

		info!("clipboard_set: spawned {:?}", child);
		let mut child = child?;

		if let Some(mut stdin) = child.stdin.take() {
			stdin.write_all(s.as_ref().as_bytes()).await?;
		}
		if child.wait().await?.success() {
			return Ok(());
		}
	}

	bail!("failed to set clipboard")
}
