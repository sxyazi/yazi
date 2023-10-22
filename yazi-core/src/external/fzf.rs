use std::process::Stdio;

use anyhow::Result;
use tokio::{process::Command, sync::oneshot::{self, Receiver}};
use yazi_shared::Url;

pub struct FzfOpt {
	pub cwd: Url,
}

pub fn fzf(opt: FzfOpt) -> Result<Receiver<Result<Url>>> {
	let child =
		Command::new("fzf").current_dir(&opt.cwd).kill_on_drop(true).stdout(Stdio::piped()).spawn()?;

	let (tx, rx) = oneshot::channel();
	tokio::spawn(async move {
		if let Ok(output) = child.wait_with_output().await {
			let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
			if !selected.is_empty() {
				tx.send(Ok(opt.cwd.join(selected))).ok();
				return;
			}
		}
		tx.send(Err(anyhow::anyhow!("No match"))).ok();
	});
	Ok(rx)
}
