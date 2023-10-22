use std::ffi::OsString;

use anyhow::Result;

#[cfg(unix)]
pub async fn clipboard_get() -> Result<OsString> {
	use std::os::unix::prelude::OsStringExt;

	use anyhow::bail;
	use tokio::process::Command;

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

#[cfg(windows)]
pub async fn clipboard_get() -> Result<OsString> {
	use anyhow::anyhow;
	use clipboard_win::{formats, get_clipboard};

	let result = tokio::task::spawn_blocking(|| get_clipboard::<String, _>(formats::Unicode));
	Ok(result.await?.map_err(|_| anyhow!("failed to get clipboard"))?.into())
}

#[cfg(unix)]
pub async fn clipboard_set(s: impl AsRef<std::ffi::OsStr>) -> Result<()> {
	use std::{os::unix::prelude::OsStrExt, process::Stdio};

	use anyhow::bail;
	use tokio::{io::AsyncWriteExt, process::Command};

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
			.stderr(Stdio::null())
			.kill_on_drop(true)
			.spawn()
		else {
			continue;
		};

		let mut stdin = child.stdin.take().unwrap();
		if stdin.write_all(s.as_ref().as_bytes()).await.is_err() {
			continue;
		}
		drop(stdin);

		if child.wait().await.map(|s| s.success()).unwrap_or_default() {
			return Ok(());
		}
	}

	bail!("failed to set clipboard")
}

#[cfg(windows)]
pub async fn clipboard_set(s: impl AsRef<std::ffi::OsStr>) -> Result<()> {
	use anyhow::anyhow;
	use clipboard_win::{formats, set_clipboard};

	let s = s.as_ref().to_owned();
	let result =
		tokio::task::spawn_blocking(move || set_clipboard(formats::Unicode, s.to_string_lossy()));

	result.await?.map_err(|_| anyhow!("failed to set clipboard"))
}
