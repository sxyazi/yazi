use std::{path::PathBuf, process::Stdio};

use anyhow::Result;
use tokio::{process::Command, sync::oneshot::{self, Receiver}};

pub struct FzfOpt {}

pub fn fzf(opt: FzfOpt) -> Result<Receiver<Result<PathBuf>>> {
	let child = Command::new("fzf").kill_on_drop(true).stdout(Stdio::piped()).spawn()?;

	let (tx, rx) = oneshot::channel();
	tokio::spawn(async move {
		if let Ok(output) = child.wait_with_output().await {
			let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
			if !selected.is_empty() {
				tx.send(Ok(selected.into())).ok();
				return;
			}
		}
		tx.send(Err(anyhow::anyhow!("No match"))).ok();
	});
	Ok(rx)
}
