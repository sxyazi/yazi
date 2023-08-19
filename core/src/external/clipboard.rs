use std::{ffi::{OsStr, OsString}, os::unix::prelude::{OsStrExt, OsStringExt}, process::Stdio};

use anyhow::{bail, Result};
use tokio::{io::AsyncWriteExt, process::Command};

pub async fn clipboard_get() -> Result<OsString> {
	let all = [
		("pbpaste", vec![]),
		("wl-paste", vec![]),
		("xclip", vec!["-o", "-selection", "clipboard"]),
		("xsel", vec!["-ob"]),
	];

	for (cmd, args) in all {
		let Ok(output) = Command::new(cmd).args(args).kill_on_drop(true).output().await else {
			continue;
		};
		if output.status.success() {
			return Ok(OsString::from_vec(output.stdout));
		}
	}

	bail!("failed to get clipboard")
}

pub async fn clipboard_set(s: impl AsRef<OsStr>) -> Result<()> {
	let all = [
		("pbcopy", vec![]),
		("wl-copy", vec![]),
		("xclip", vec!["-selection", "clipboard"]),
		("xsel", vec!["-ib"]),
	];

	for (cmd, args) in all {
		let Ok(mut child) = Command::new(cmd)
			.args(args)
			.stdin(Stdio::piped())
			.stdout(Stdio::null())
			.kill_on_drop(true)
			.spawn()
		else {
			continue;
		};

		let mut stdin = child.stdin.take().unwrap();
		if stdin.write_all(s.as_ref().as_bytes()).await.is_err() {
			continue;
		}
		if child.wait().await.map(|s| s.success()).unwrap_or_default() {
			return Ok(());
		}
	}

	bail!("failed to set clipboard")
}
