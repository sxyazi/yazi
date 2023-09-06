use std::process::Stdio;

use anyhow::Result;
use shared::Url;
use tokio::{process::Command, sync::oneshot::{self, Receiver}};

pub struct ZoxideOpt {
	pub cwd: Url,
}

pub fn zoxide(opt: ZoxideOpt) -> Result<Receiver<Result<Url>>> {
	let child = Command::new("zoxide")
		.args(["query", "-i", "--exclude"])
		.arg(&opt.cwd)
		.kill_on_drop(true)
		.stdout(Stdio::piped())
		.spawn()?;

	let (tx, rx) = oneshot::channel();
	tokio::spawn(async move {
		if let Ok(output) = child.wait_with_output().await {
			let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
			if !selected.is_empty() {
				tx.send(Ok(Url::from(selected))).ok();
				return;
			}
		}
		tx.send(Err(anyhow::anyhow!("No match"))).ok();
	});
	Ok(rx)
}
