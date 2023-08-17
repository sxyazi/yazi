use std::process::Stdio;

use anyhow::{bail, Result};
use tokio::{io::AsyncWriteExt, process::Command};

pub async fn clipboard_get() -> Result<String> {
	for cmd in &["pbpaste", "wl-paste"] {
		let output = Command::new(cmd).kill_on_drop(true).output().await?;
		if output.status.success() {
			return Ok(String::from_utf8_lossy(&output.stdout).to_string());
		}
	}

	bail!("failed to get clipboard")
}

pub async fn clipboard_set(s: &str) -> Result<()> {
	for cmd in &["pbcopy", "wl-copy"] {
		let mut child =
			Command::new(cmd).stdin(Stdio::piped()).stdout(Stdio::null()).kill_on_drop(true).spawn()?;
		if let Some(mut stdin) = child.stdin.take() {
			stdin.write_all(s.as_bytes()).await?;
		}
		if child.wait().await?.success() {
			return Ok(());
		}
	}

	bail!("failed to set clipboard")
}
