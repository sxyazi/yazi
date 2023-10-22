use std::{path::Path, process::Stdio};

use anyhow::{bail, Result};
use tokio::{io::AsyncReadExt, process::Command};

pub async fn unar_head(path: &Path, target: &Path) -> Result<Vec<u8>> {
	let mut child = Command::new("unar")
		.args([path, target])
		.args(["-o", "-"])
		.stdout(Stdio::piped())
		.stderr(Stdio::null())
		.kill_on_drop(true)
		.spawn()?;

	let mut buf = vec![0; 1024];
	child.stdout.take().unwrap().read(&mut buf).await.ok();
	child.start_kill().ok();

	if buf.is_empty() {
		bail!("failed to get head of unar");
	}
	Ok(buf)
}
